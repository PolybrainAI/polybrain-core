use crate::{
    server::{
        background::{BackgroundClient, BackgroundRequest, BackgroundResponse},
        types::ServerResponse,
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

    /// The function to invoke the agent
    async fn invoke(&mut self) -> Result<Self::InvocationResponse, PolybrainError>;

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
