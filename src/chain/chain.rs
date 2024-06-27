use std::error::Error;
use std::io;
use std::{future::Future, pin::Pin};

use crate::chain::agents::executive_planner::ExecutivePlanner;
use crate::chain::agents::mathematician::MathematicianAgent;
use crate::chain::agents::pessimist::PessimistAgent;
use crate::chain::agents::preliminary_reporter::{self, PreliminaryReporter};
use crate::server::types::{ApiCredentials, ServerResponse, ServerResponseType};

pub async fn enter_chain<'a, I, O>(
    initial_input: &str,
    credentials: ApiCredentials,
    query_input: I,
    send_output: O,
) -> io::Result<()>
where
    I: Fn(String) -> Pin<Box<dyn Future<Output = Result<String, Box<dyn Error>>> + Send + 'a>>
        + Send
        + 'a,
    O: Fn(ServerResponse) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'a>>
        + Send
        + 'a,
{
    println!("Entering chain with initial input: {}", initial_input);

    // Pessimist Chain
    let mut pessimist = PessimistAgent::new(&credentials.openai_token);
    let parsed_prompt = pessimist
        .run(initial_input, &query_input, &send_output)
        .await
        .map_err(|err| eprintln!("Pessimist errored: {}", err))
        .unwrap();

    // Mathematician Chain
    let mathematician = MathematicianAgent::new(&credentials.openai_token);
    let math_notes = mathematician.run().await;

    // Executive Planner Chain
    let mut executive_planner =
        ExecutivePlanner::new(&credentials.openai_token, &parsed_prompt, &math_notes).unwrap();
    let modeler_outline = executive_planner.run(&query_input).await.unwrap();

    // Preliminary Reporter Chain
    let mut preliminary_reporter =
        PreliminaryReporter::new(&credentials.openai_token, modeler_outline.clone());
    preliminary_reporter.run(&send_output).await.unwrap();

    println!("The modeler outline is:\n{}", modeler_outline);

    credentials.openai_token;

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    send_output(ServerResponse {
        response_type: ServerResponseType::Final,
        content: "LLM Chain has finished!".to_owned(),
    })
    .await
    .unwrap();

    Ok(())
}
