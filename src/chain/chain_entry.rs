use uuid::Uuid;

use crate::chain::agents::executive_planner::ExecutivePlanner;
use crate::chain::agents::mathematician::Mathematician;
use crate::chain::agents::onpy_agent::OnPyAgent;
use crate::chain::agents::pessimist::PessimistAgent;
use crate::chain::agents::preliminary_reporter::PreliminaryReporter;
use crate::chain::agents::Agent;
use crate::server::auth::fetch_user_credentials;
use crate::server::background::{BackgroundClient, BackgroundRequest, BackgroundResponse};
use crate::server::error::PolybrainError;
use crate::server::types::{
    ApiCredentials, ServerResponse, ServerResponseType, SessionStartResponse, UserPromptInitial,
};

async fn handshake(
    client: &mut BackgroundClient,
) -> Result<(ApiCredentials, UserPromptInitial, String), PolybrainError> {
    println!("Spawned new task for socket");

    let session_start_request = match client.send(BackgroundRequest::GetSessionStart).await? {
        BackgroundResponse::SessionStart(s) => s,
        _ => return Err(PolybrainError::Unreachable),
    };

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
    let initial_input = match client.send(BackgroundRequest::GetInitialInput).await? {
        BackgroundResponse::InitialInput(inp) => inp,
        _ => {
            return Err(PolybrainError::Unreachable);
        }
    };

    Ok((
        credentials,
        initial_input,
        session_start_request.onshape_document_id,
    ))
}

pub async fn enter_chain(mut client: BackgroundClient) -> Result<(), PolybrainError> {
    let (credentials, initial_input, onshape_document) = handshake(&mut client).await?;
    println!(
        "Entering chain with initial input: {}",
        initial_input.contents
    );

    // Pessimist Chain
    let mut pessimist =
        PessimistAgent::new(&credentials, &mut client, initial_input.contents.clone());
    let parsed_prompt = pessimist.invoke().await?;
    // Mathematician Chain
    let mut mathematician = Mathematician::new(&credentials, &mut client);
    let math_notes = mathematician.invoke().await?;

    // Executive Planner Chain
    let mut executive_planner =
        ExecutivePlanner::new(&credentials, &mut client, parsed_prompt, math_notes);
    let executive_report = executive_planner.invoke().await?;
    println!("The modeler outline is:\n{}", executive_report);

    // Preliminary Reporter Chain
    let mut preliminary_reporter =
        PreliminaryReporter::new(&credentials, executive_report.clone(), &mut client);
    preliminary_reporter.invoke().await?;

    // OnPy Agent Chain
    let mut onpy_agent = OnPyAgent::new(
        &credentials,
        &mut client,
        executive_report,
        initial_input.contents,
        onshape_document,
    );
    onpy_agent.invoke().await?;

    client
        .send(BackgroundRequest::End(ServerResponse {
            response_type: ServerResponseType::Final,
            content: "Your model has been created!".to_owned(),
        }))
        .await
        .unwrap();

    Ok(())
}
