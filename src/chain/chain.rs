use std::error::Error;
use std::io;
use std::{future::Future, pin::Pin};

use crate::server::types::{ServerResponse, ServerResponseType};

pub async fn enter_chain<'a, I, O>(
    initial_input: &str,
    query_input: I,
    send_output: O,
) -> io::Result<()>
where
    I: Fn(&'a str) -> Pin<Box<dyn Future<Output = Result<String, Box::<dyn Error>>> + Send + 'a>> + Send + 'a,
    O: Fn(ServerResponse) -> Pin<Box<dyn Future<Output = Result<(), Box::<dyn Error>>> + Send + 'a>> + Send + 'a,
{
    println!("Entering chain with initial input: {}", initial_input);

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    let user_input = query_input("Enter a value please").await.unwrap();
    println!("got user input: {}", user_input);

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    send_output(ServerResponse {
        response_type: ServerResponseType::Info,
        content: "Thinking".to_owned(),
    })
    .await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    send_output(ServerResponse {
        response_type: ServerResponseType::Final,
        content: "here is your model".to_owned(),
    })
    .await.unwrap();

    Ok(())
}
