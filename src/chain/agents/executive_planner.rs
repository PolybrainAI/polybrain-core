use futures::Future;
use llm_chain::prompt;
use llm_chain::tools::ToolUseError;
use llm_chain::{executor, options, parameters, tools::ToolCollection};
use llm_chain_openai::chatgpt::Model;
use std::{error::Error, pin::Pin};

use crate::chain::tools::misc::deserialize_output;
use crate::chain::tools::report_tool::{Report, ReportError, ReportInput, ReportOutput};
use crate::chain::tools::user_input_tool::{
    UserQuery, UserQueryError, UserQueryInput, UserQueryOutput,
};
use crate::server::background::{BackgroundClient, BackgroundRequest};
use crate::server::types::ApiCredentials;
use crate::util::PolybrainError;

use async_trait::async_trait;
use llm_chain::{
    multitool,
    tools::ToolDescription,
    tools::{Tool, ToolError},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::Agent;

multitool!(
    Multitool,
    MultiToolInput,
    MultiToolOutput,
    MultitoolError,
    UserQuery,
    UserQueryInput,
    UserQueryOutput,
    UserQueryError,
    Report,
    ReportInput,
    ReportOutput,
    ReportError
);

const EXECUTIVE_PLANNER_PROMPT: &str = include_str!("prompts/executive_planner_main.md");

const MAX_ITER: usize = 7;

pub struct ExecutivePlanner<'b> {
    credentials: &'b ApiCredentials,
    client: &'b mut BackgroundClient,
    model_description: String,
    math_notes: String,
}

impl<'b> ExecutivePlanner<'b> {
    pub fn new(
        credentials: &'b ApiCredentials,
        client: &'b mut BackgroundClient,
        model_description: String,
        math_notes: String,
    ) -> Self {
        Self {
            credentials,
            client,
            model_description,
            math_notes,
        }
    }

    async fn process_user_input_tool(&mut self, output: &str) -> Result<String, PolybrainError> {
        let mut response = deserialize_output(output)
            .map_err(|err| PolybrainError::InternalError(err.to_string()))?;
        let input: UserQueryInput = serde_yaml::from_value(response.clone().input)
            .inspect_err(|e| {
                println!(
                    "Unable to extract input from user input tool response: {}",
                    e
                )
            })
            .map_err(|err| PolybrainError::InternalError(err.to_string()))?;
        let prompt = input.question.replace("\"", "");

        let user_input = self.query_input(prompt).await?;

        response.output = user_input;

        serde_yaml::to_string(&response).map_err(|_| {
            PolybrainError::InternalError("Failed to convert user input into yaml".to_owned())
        })
    }

    fn process_report_tool(&self, output: &str) -> String {
        let output = *output
            .split("content:")
            .collect::<Vec<&str>>()
            .last()
            .expect("No report content?");
        let mut output = output.replace("output: none: null", "").trim().to_owned();

        if output.starts_with("|") {
            output = output.replacen("|", "", 1);
        }

        output
    }
}

impl<'b> Agent for ExecutivePlanner<'b> {
    type InvocationResponse = String;

    async fn client<'a>(&'a mut self) -> &'a mut BackgroundClient {
        self.client
    }

    fn credentials<'a>(&'a self) -> &'a ApiCredentials {
        &self.credentials
    }

    fn model(&self) -> Model {
        Model::Other("gpt-4o".to_owned())
    }

    fn name<'a>(&'a self) -> &'a str {
        "Executive Planner"
    }

    async fn invoke(&mut self) -> Result<String, PolybrainError> {
        let mut tool_collection: ToolCollection<Multitool> = ToolCollection::new();
        tool_collection.add_tool(UserQuery::new().into());
        tool_collection.add_tool(Report::new().into());

        let tool_prompt = tool_collection.to_prompt_template().map_err(|err| {
            PolybrainError::InternalError(format!("Error getting tool prompt: {}", err))
        })?;
        let mut scratchpad = String::new();

        for _ in 0..MAX_ITER {
            let parameters = parameters!(
                "model_description" => &self.model_description,
                "math_notes" => &self.math_notes,
                "tools" => tool_prompt.to_string(),
                "scratchpad" => scratchpad.clone(),
            );
            let res = self
                .call_llm(EXECUTIVE_PLANNER_PROMPT, parameters)
                .await?
                .replace("```yaml", "")
                .replace("```", "");

            match tool_collection.process_chat_input(&res).await {
                Ok(new) => {
                    let new = new.replace("result:", "");
                    let mut addition = format!("\n{}\noutput: {}\n", res.trim(), new.trim())
                        .replace("```yaml", "")
                        .replace("```", "");

                    if addition.contains("command: Report") {
                        return Ok(self.process_report_tool(&addition));
                    }

                    if addition.contains("command: User Query") {
                        addition = self.process_user_input_tool(&addition).await?;
                    }

                    println!(
                        concat!(
                            "====SCRATCHPAD NEW====\n",
                            "{}\n",
                            "======================\n",
                        ),
                        addition
                    );

                    scratchpad.push_str(&addition);
                }
                Err(ToolUseError::NoToolInvocation) => {
                    // Assume there is a comment when there's no invocation
                    println!(
                        concat!(
                            "====SCRATCHPAD NEW====\n",
                            "{}\n",
                            "======================\n",
                        ),
                        res
                    );
                    scratchpad.push_str(&format!("\n{}", res));
                }
                Err(e) => {
                    eprintln!(
                        concat!(
                            "====LLM YAML ERROR====\n",
                            "Response: \n{}\n",
                            "Error: \n{}\n",
                            "======================\n",
                        ),
                        res, e
                    );

                    scratchpad.push_str(&format!(concat!(
                        "YAML ERROR: Rephrase the following. Remove any mappings and respond as a string with \"|\"\n",
                        "Input\n```\n{}\n```\n",
                        "Output\n```\n{}\n```"
                    ), res, e ));
                }
            };
        }

        Ok("temp".to_owned())
    }
}
