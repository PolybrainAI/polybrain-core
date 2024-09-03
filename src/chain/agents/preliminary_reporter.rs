use std::{error::Error, pin::Pin};

use futures::Future;
use llm_chain::options;
use llm_chain::prompt;
use llm_chain::{executor, parameters};
use llm_chain_openai;
use llm_chain_openai::chatgpt::Model;

use crate::server::background::BackgroundClient;
use crate::server::types::ApiCredentials;
use crate::server::types::ServerResponseType;
use crate::util::PolybrainError;
use crate::{chain::util::trim_assistant_prefix, server::types::ServerResponse};

use super::Agent;

const PRELIMINARY_REPORTER_PROMPT: &str = r###"
You are a reporter for Polybrain. The following outline was written by an 
executive to an engineer, detailing how he should build the model in OnPy.
Create a very short message to the client that gives a brief idea of how the
model will be created. This should only be about 1-2 sentence(s) long.

Respond in first person in friendly english. Your report will be announced as the
engineer is working; do not include an introduction, greeting, or goodbye. You
will act as if you are the person making the changes; don't reference anybody
other than yourself.

Do not mention OnPy, the engineer, nor the executive. Simply respond with the
general actions "you" will take.

The report is:
{{report}}
"###;

pub struct PreliminaryReporter<'b> {
    report: String,
    credentials: &'b ApiCredentials,
    client: &'b mut BackgroundClient,
}

impl<'b> PreliminaryReporter<'b> {
    pub fn new(
        credentials: &'b ApiCredentials,
        report: String,
        client: &'b mut BackgroundClient,
    ) -> Self {
        Self {
            report,
            credentials,
            client,
        }
    }
}

impl<'b> Agent for PreliminaryReporter<'b> {
    type InvocationResponse = ();

    async fn client<'a>(&'a mut self) -> &'a mut BackgroundClient {
        self.client
    }

    async fn invoke(&mut self) -> Result<(), PolybrainError> {
        let opts = options! {
            Model: Model::Other("gpt-4o".to_string()),
            // Model: Model::Gpt35Turbo,
            ApiKey: self.credentials.openai_token.clone()
        };
        let exec = executor!(chatgpt, opts)
            .map_err(|err| PolybrainError::InternalError(err.to_string()))?;

        let report = prompt!(PRELIMINARY_REPORTER_PROMPT)
            .run(&parameters!("report" => &self.report), &exec)
            .await
            .map_err(|_| {
                PolybrainError::InternalError("Error in PreliminaryReporter LLM".to_owned())
            })?
            .to_immediate()
            .await
            .map_err(|_| {
                PolybrainError::InternalError(
                    "Failed to convert LLM response to immediate".to_owned(),
                )
            })?
            .primary_textual_output()
            .expect("No LLM output");

        let report = trim_assistant_prefix(&report).replace("OnPy", "OnShape");
        println!("Summarized prompt as: {}", report);

        self.send_message(ServerResponse {
            response_type: ServerResponseType::Info,
            content: report,
        })
        .await?;

        Ok(())
    }
}
