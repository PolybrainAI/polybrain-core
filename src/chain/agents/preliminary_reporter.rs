use std::{error::Error, pin::Pin};

use futures::Future;
use llm_chain::options;
use llm_chain::prompt;
use llm_chain::{executor, parameters};
use llm_chain_openai;
use llm_chain_openai::chatgpt::Model;

use crate::server::types::ServerResponseType;
use crate::{chain::util::trim_assistant_prefix, server::types::ServerResponse};

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
    openai_key: &'b String,
}

impl<'b> PreliminaryReporter<'b> {
    pub fn new(openai_key: &'b String, report: String) -> PreliminaryReporter {
        PreliminaryReporter { openai_key, report }
    }

    pub async fn run<'a, O>(&mut self, send_output: &O) -> Result<(), Box<dyn std::error::Error>>
    where
        O: Fn(
                ServerResponse,
            ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'a>>
            + Send
            + 'a,
    {
        let opts = options! {
            Model: Model::Other("gpt-4o".to_string()),
            // Model: Model::Gpt35Turbo,
            ApiKey: self.openai_key.clone()
        };
        let exec = executor!(chatgpt, opts)?;

        let report = prompt!(PRELIMINARY_REPORTER_PROMPT)
            .run(&parameters!("report" => &self.report), &exec)
            .await?
            .to_immediate()
            .await?
            .primary_textual_output()
            .expect("No LLM output");

        let report = trim_assistant_prefix(&report).replace("OnPy", "OnShape");
        println!("Summarized prompt as: {}", report);

        send_output(ServerResponse {
            response_type: ServerResponseType::Info,
            content: report,
        })
        .await?;

        Ok(())
    }
}
