use llm_chain_openai::chatgpt::Model;

use crate::{
    server::{background::BackgroundClient, types::ApiCredentials},
    server::error::PolybrainError,
};

use super::Agent;

pub struct Mathematician<'a> {
    credentials: &'a ApiCredentials,
    client: &'a mut BackgroundClient,
}
impl<'a> Mathematician<'a> {
    pub fn new(credentials: &'a ApiCredentials, client: &'a mut BackgroundClient) -> Self {
        Mathematician {
            credentials,
            client,
        }
    }
}

impl<'b> Agent for Mathematician<'b> {
    type InvocationResponse = String;

    fn name(&self) -> &str {
        "Mathematician"
    }

    fn credentials(&self) -> &ApiCredentials {
        self.credentials
    }

    fn model(&self) -> llm_chain_openai::chatgpt::Model {
        Model::Other("gpt-4o-mini".to_owned())
    }

    async fn client<'a>(&'a mut self) -> &'a mut BackgroundClient {
        self.client
    }

    async fn invoke(&mut self) -> Result<String, PolybrainError> {
        Ok("No math notes".to_owned())
    }
}
