
use llm_chain::{
    parameters,
    prompt::{ChatMessage, Conversation},
};
use llm_chain_openai;
use llm_chain_openai::chatgpt::Model;

use crate::server::background::BackgroundClient;
use crate::server::types::ApiCredentials;
use crate::server::error::PolybrainError;
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

}

impl<'b> Agent for PessimistAgent<'b> {
    type InvocationResponse = String;

    async fn client(&mut self) -> &mut BackgroundClient {
        self.client
    }

    fn credentials(&self) -> &ApiCredentials {
        self.credentials
    }

    fn name(&self) -> &str {
        "Pessimist"
    }

    fn model(&self) -> Model {
        Model::Other("gpt-4o-mini".to_owned())
    }

    async fn invoke(&mut self) -> Result<String, PolybrainError> {
        let mut agent_response: String = "".to_owned();

        while !agent_response.contains("Begin!") {
            let parameters = parameters!(
                "conversation_history" => self.build_conversation_history()
            );

            let response = self.call_llm(PESSIMIST_PROMPT, parameters).await?;

            agent_response = trim_assistant_prefix(&response).trim().to_string();

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

        let parameters = parameters!("conversation_history" => self.build_conversation_history());
        let mut summary = self.call_llm(SUMMARIZER_PROMPT, parameters).await?;
        summary = trim_assistant_prefix(&summary).to_owned();

        println!("Summarized prompt as: {}", summary);

        Ok(summary)
    }
}
