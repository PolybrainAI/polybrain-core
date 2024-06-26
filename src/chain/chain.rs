use std::error::Error;
use std::io;
use std::{future::Future, pin::Pin};

use crate::chain::agents::mathematician::{self, MathematicianAgent};
use crate::chain::agents::pessimist::PessimistAgent;
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

    let parsed_prompt = pessimist.run(
        initial_input,
        &query_input,
        &send_output,
    ).await.unwrap();

    // Mathematician Chain
    let mut mathematician = MathematicianAgent::new(&credentials.openai_token);
    let math_notes = mathematician.run().await;

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    send_output(ServerResponse {
        response_type: ServerResponseType::Final,
        content: format!("LLM Chain has finished! The prompt is: '{}'", parsed_prompt),
    })
    .await
    .unwrap();

    Ok(())
}
