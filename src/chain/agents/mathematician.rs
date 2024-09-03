use crate::{
    server::{background::BackgroundClient, types::ApiCredentials},
    util::PolybrainError,
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

impl<'a> Agent for Mathematician<'a> {
    type InvocationResponse = String;

    async fn client<'b>(&'b mut self) -> &'b mut BackgroundClient {
        self.client
    }

    async fn invoke(&mut self) -> Result<String, PolybrainError> {
        Ok("No math notes".to_owned())
    }
}
