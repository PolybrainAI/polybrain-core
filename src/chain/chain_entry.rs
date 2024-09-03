use std::error::Error;
use std::io;
use std::{future::Future, pin::Pin};

use uuid::Uuid;

use crate::chain::agents::executive_planner::ExecutivePlanner;
use crate::chain::agents::mathematician::Mathematician;
use crate::chain::agents::onpy_agent::OnPyAgent;
use crate::chain::agents::pessimist::PessimistAgent;
use crate::chain::agents::preliminary_reporter::PreliminaryReporter;
use crate::server::auth::fetch_user_credentials;
use crate::server::background::{BackgroundClient, BackgroundRequest, BackgroundResponse};
use crate::server::types::{
    ApiCredentials, ServerResponse, ServerResponseType, SessionStartResponse, UserPromptInitial,
};
use crate::util::PolybrainError;

async fn handshake(
    client: &mut BackgroundClient,
) -> Result<(ApiCredentials, UserPromptInitial), PolybrainError> {
    println!("Spawned new task for socket");

    let session_start_request;
    match client.send(BackgroundRequest::GetSessionStart).await? {
        BackgroundResponse::SessionStart(s) => {
            session_start_request = s;
        }
        _ => return Err(PolybrainError::Unreachable),
    }

    println!("Got session start request");
    let credentials: ApiCredentials =
        fetch_user_credentials(&session_start_request.user_token).await?;

    let session_id = Uuid::new_v4().to_string();
    println!("Starting session with id {session_id}");

    let session_start_response = SessionStartResponse { session_id };
    match client
        .send(BackgroundRequest::RespondSessionStart(
            session_start_response,
        ))
        .await?
    {
        BackgroundResponse::Ack => (),
        _ => return Err(PolybrainError::Unreachable),
    }

    println!("Getting initial prompt...");
    let inital_input = match client.send(BackgroundRequest::GetInitialInput).await? {
        BackgroundResponse::InitialInput(inp) => inp,
        _ => {
            return Err(PolybrainError::Unreachable);
        }
    };

    Ok((credentials, inital_input))
}

pub async fn enter_chain(client: BackgroundClient) -> Result<(), PolybrainError> {
    let (credentials, initial_input) = handshake(&mut client).await?;
    println!(
        "Entering chain with initial input: {}",
        initial_input.contents
    );

    // Pessimist Chain
    let mut pessimist = PessimistAgent::new(&credentials.openai_token);
    let parsed_prompt = pessimist
        .run(initial_input, &query_input, &send_output)
        .await
        .map_err(|err| eprintln!("Pessimist errored: {}", err))
        .unwrap();

    // Mathematician Chain
    let mathematician = Mathematician::new(&credentials.openai_token);
    let math_notes = mathematician.run().await;

    // Executive Planner Chain
    let mut executive_planner =
        ExecutivePlanner::new(&credentials.openai_token, &parsed_prompt, &math_notes).unwrap();
    let modeler_outline = executive_planner.run(&query_input).await.unwrap();
    println!("The modeler outline is:\n{}", modeler_outline);

    // Preliminary Reporter Chain
    let mut preliminary_reporter =
        PreliminaryReporter::new(&credentials.openai_token, modeler_outline.clone());
    preliminary_reporter.run(&send_output).await.unwrap();

    // OnPy Agent Chain
    let mut onpy_agent = OnPyAgent::new(
        &credentials.openai_token,
        modeler_outline,
        parsed_prompt,
        onshape_document_id,
    );
    onpy_agent.run(&query_input).await.unwrap();

    send_output(ServerResponse {
        response_type: ServerResponseType::Final,
        content: "Your model has been created!".to_owned(),
    })
    .await
    .unwrap();

    Ok(())
}
