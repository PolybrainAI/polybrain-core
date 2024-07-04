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

use async_trait::async_trait;
use llm_chain::{
    multitool,
    tools::ToolDescription,
    tools::{Tool, ToolError},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

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

const EXECUTIVE_PLANNER_PROMPT: &str = "\
You are a professional mechanical engineer familiar with popular parametric CAD 
programs, such as SolidWorks and OnShape. You will provide an in depth report
on the steps to take in order to create the following model in a new modeling
software called OnPy, which is similar to SolidWorks and OnShape. 

The description of the model to create is:
```txt
{{model_description}}
```

A coworker has provided the following mathematical notes:
```txt
{{math_notes}}
```

OnPy is a limited tool, so your instructions MUST conform to the following 
constraints:

Sketches can only be created on 2D flat planes. Within these sketches,
users can ONLY draw:
- Straight lines between two points
- Circles at a specified origin
- Fillets between two lines
- Centerpoint arcs

Users can copy, mirror, and pattern their designs.

After creating a sketch, the following features are available. If
a feature is not listed here, then it cannot be used in OnPy:
- Extrusions
- Offset Planes
- Lofts

Final Considerations:
- There are no sketch constraints in OnPy; do not mention them.
- OnPy cannot control color, or surface finish.
- Your report is going to another employee, so now is the chance to ask any
questions to the user about desired measurements.
- All OnPy units are in Inches. Only include units of Inches in your response

================

{{tools}}

You are encouraged to explain your thoughts as much as possible. Prefix
all thoughts with a YAML comment (i.e., a line that begins with #)

===== PREVIOUS COMMANDS & THOUGHTS =====

```yaml
{{scratchpad}}
```

===== NEW COMMANDS & THOUGHTS =====

```yaml
";

const MAX_ITER: usize = 7;

pub struct ExecutivePlanner<'b> {
    openai_key: &'b String,
    model_description: &'b String,
    math_notes: &'b String,
}

impl<'b> ExecutivePlanner<'b> {
    pub fn new(
        openai_key: &'b String,
        model_description: &'b String,
        math_notes: &'b String,
    ) -> Result<ExecutivePlanner<'b>, Box<dyn Error>> {
        Ok(ExecutivePlanner {
            openai_key,
            model_description,
            math_notes,
        })
    }

    async fn process_user_input_tool<'a, I>(
        &mut self,
        output: &str,
        get_input: &I,
    ) -> Result<String, Box<dyn Error>>
    where
        I: Fn(String) -> Pin<Box<dyn Future<Output = Result<String, Box<dyn Error>>> + Send + 'a>>
            + Send
            + 'a,
    {
        let mut response = deserialize_output(&output)?;
        let input: UserQueryInput =
            serde_yaml::from_value(response.clone().input).inspect_err(|e| {
                println!(
                    "Unable to extract input from user input tool response: {}",
                    e
                )
            })?;
        let prompt = input.question.replace("\"", "");
        let real_user_input = get_input(prompt).await?;

        response.output = real_user_input;

        Ok(serde_yaml::to_string(&response)?)
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

    pub async fn run<'a, I>(&mut self, get_input: &I) -> Result<String, Box<dyn Error>>
    where
        I: Fn(String) -> Pin<Box<dyn Future<Output = Result<String, Box<dyn Error>>> + Send + 'a>>
            + Send
            + 'a,
    {
        let mut tool_collection: ToolCollection<Multitool> = ToolCollection::new();
        tool_collection.add_tool(UserQuery::new().into());
        tool_collection.add_tool(Report::new().into());

        let tool_prompt = tool_collection.to_prompt_template()?;
        let mut scratchpad = String::new();

        let opts = options! {
            Model: Model::Other("gpt-4o".to_string()),
            // Model: Model::Gpt35Turbo,
            ApiKey: self.openai_key.clone()
        };
        let exec = executor!(chatgpt, opts)?;

        for _ in 0..MAX_ITER {
            let parameters = parameters!(
                "model_description" => self.model_description,
                "math_notes" => self.math_notes,
                "tools" => tool_prompt.to_string(),
                "scratchpad" => scratchpad.clone(),
            );
            let res = prompt!(EXECUTIVE_PLANNER_PROMPT)
                .run(&parameters, &exec)
                .await?
                .to_immediate()
                .await?
                .primary_textual_output()
                .expect("No LLM output")
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
                        addition = self.process_user_input_tool(&addition, get_input).await?;
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

        return Ok("temp".to_owned());
    }
}
