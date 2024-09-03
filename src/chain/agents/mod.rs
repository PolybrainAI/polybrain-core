use llm_chain::executor;
use llm_chain::options;
use llm_chain::prompt;
use llm_chain::Parameters;
use llm_chain_openai::chatgpt::{Executor, Model};

use crate::{
    server::{
        background::{BackgroundClient, BackgroundRequest, BackgroundResponse},
        types::{ApiCredentials, ServerResponse},
    },
    util::PolybrainError,
};

pub mod executive_planner;
pub mod mathematician;
pub mod onpy_agent;
pub mod pessimist;
pub mod preliminary_reporter;
pub mod types;

pub trait Agent {
    type InvocationResponse;

    /// A reference to the background client
    async fn client<'a>(&'a mut self) -> &'a mut BackgroundClient;

    /// A reference to the credentials
    fn credentials<'a>(&'a self) -> &'a ApiCredentials;

    /// The function to invoke the agent
    async fn invoke(&mut self) -> Result<Self::InvocationResponse, PolybrainError>;

    /// The name of the agent, used for debugging
    fn name<'a>(&'a self) -> &'a str;

    /// The type of model the agent runs on
    fn model(&self) -> Model;

    /// Constructs an exec object
    fn executor(&self) -> Result<Executor, PolybrainError> {
        let opts = options! {
            Model: self.model(),
            ApiKey: self.credentials().openai_token.clone()
        };
        executor!(chatgpt, opts).map_err(|err| {
            PolybrainError::InternalError(format!("Integral error when creating executor: {err}"))
        })
    }

    /// Calls the LLM
    ///
    /// Args:
    /// * `prompt` - The prompt to supply the LLM
    /// * `parameters` - A Parameters object to substitute into the prompt
    async fn call_llm(
        &self,
        llm_prompt: &str,
        parameters: Parameters,
    ) -> Result<String, PolybrainError> {
        let exec = self.executor()?;

        prompt!(llm_prompt)
            .run(&parameters, &exec)
            .await
            .map_err(|err| {
                PolybrainError::InternalError(format!(
                    "An error occurred when invoking LLM: {}",
                    err
                ))
            })?
            .to_immediate()
            .await
            .map_err(|_| {
                PolybrainError::InternalError(
                    "Error converting LLM response to immediate".to_owned(),
                )
            })?
            .primary_textual_output()
            .ok_or(PolybrainError::InternalError(
                "LLM responded with no output".to_owned(),
            ))
    }

    /// Ask a question to the user
    ///
    /// Args:
    /// * `input` - The question to ask the user
    ///
    /// Returns:
    /// * A `String` of the user's response
    async fn query_input(&mut self, input: String) -> Result<String, PolybrainError> {
        let client = self.client().await;
        let answer = match client.send(BackgroundRequest::UserQuery(input)).await? {
            BackgroundResponse::UserResponse(ans) => Ok(ans),
            _ => Err(PolybrainError::Unreachable),
        }?;

        println!("User responded with '{}'", answer);
        Ok(answer)
    }

    /// Send a message to the browser
    ///
    /// Args:
    /// * `message` - The message to send to the browser
    async fn send_message(&mut self, message: ServerResponse) -> Result<(), PolybrainError> {
        let client = self.client().await;

        let debug_copy = format!("{:#?}", message);

        match client.send(BackgroundRequest::SendOutput(message)).await? {
            BackgroundResponse::Ack => Ok(()),
            _ => Err(PolybrainError::Unreachable),
        }?;

        println!("Sent message to the server:\n {}", debug_copy);

        Ok(())
    }
}
