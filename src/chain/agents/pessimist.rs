use std::{error::Error, pin::Pin};

use futures::Future;
use llm_chain::options;
use llm_chain::prompt;
use llm_chain::{
    executor, parameters,
    prompt::{ChatMessage, Conversation},
};
use llm_chain_openai;
use llm_chain_openai::chatgpt::Model;

use crate::server::background::BackgroundClient;
use crate::server::types::ApiCredentials;
use crate::util::PolybrainError;
use crate::{
    chain::util::trim_assistant_prefix,
    server::types::{ServerResponse, ServerResponseType},
};

use super::Agent;

const PESSIMIST_PROMPT: &str = include_str!("prompts/pessimist_main.md");
const SUMMARIZER_PROMPT: &str = include_str!("prompts/pessimist_summarizer.md");

pub struct PessimistAgent<'b> {
    messages: Conversation,
    credentials: &'b ApiCredentials,
    client: &'b mut BackgroundClient,
}

impl<'b> PessimistAgent<'b> {
    pub fn new(
        credentials: &'b ApiCredentials,
        client: &'b mut BackgroundClient,
        initial_message: String,
    ) -> Self {
        let messages = Conversation::new().with_user(initial_message);

        Self {
            messages,
            credentials,
            client,
        }
    }

    fn build_conversation_history(&self) -> String {
        let message_history = &self.messages.to_string();
        message_history.to_owned()
    }

    fn build_prompt(&self) -> String {
        PESSIMIST_PROMPT.replace(
            "{{conversation_history}}",
            &self.build_conversation_history(),
        )
    }
}

impl<'b> Agent for PessimistAgent<'b> {
    type InvocationResponse = String;

    async fn client<'a>(&'a mut self) -> &'a mut BackgroundClient {
        self.client
    }

    async fn invoke(&mut self) -> Result<String, PolybrainError> {
        let mut agent_response: String = "".to_owned();

        let opts = options! {
            Model: Model::Other("gpt-4o".to_string()),
            // Model: Model::Gpt35Turbo,
            ApiKey: self.credentials.openai_token.clone(),
            StopSequence: vec!["User:".to_string()]
        };
        let exec = executor!(chatgpt, opts).map_err(|_| {
            PolybrainError::InternalError("Error calling pessimist executor".to_owned())
        })?;

        while !agent_response.contains("Begin!") {
            let parameters = parameters! {};

            let res = prompt!(system: &self.build_prompt())
                .run(&parameters, &exec) // ...and run it
                .await
                .map_err(|_| {
                    PolybrainError::InternalError("Error calling Pessimist LLM".to_owned())
                })?
                .to_immediate()
                .await
                .map_err(|_| {
                    PolybrainError::InternalError(
                        "Error converting LLM response to immediate".to_owned(),
                    )
                })?
                .as_content()
                .to_text();

            agent_response = trim_assistant_prefix(&res).trim().to_string();

            println!("Pessimist: {}", agent_response);

            if agent_response.contains("Begin!") {
                self.send_message(ServerResponse {
                    response_type: ServerResponseType::Info,
                    content: agent_response.replace("Begin!", ""),
                })
                .await?;
            } else {
                self.messages
                    .add_message(ChatMessage::assistant(agent_response.replace("\n", " ")));
                let user_input = self.query_input(agent_response.clone()).await?;
                self.messages.add_message(ChatMessage::user(user_input))
            }
        }

        // Summarize what the user decided on
        let summary = prompt!(SUMMARIZER_PROMPT)
            .run(
                &parameters!("conversation_history" => self.build_conversation_history()),
                &exec,
            )
            .await
            .map_err(|_| PolybrainError::InternalError("Error calling Pessimist LLM".to_owned()))?
            .to_immediate()
            .await
            .map_err(|_| {
                PolybrainError::InternalError(
                    "Error converting LLM response to immediate".to_owned(),
                )
            })?
            .as_content()
            .to_text();

        let summary = trim_assistant_prefix(&summary).to_owned();

        println!("Summarized prompt as: {}", summary);

        Ok(summary)
    }
}
