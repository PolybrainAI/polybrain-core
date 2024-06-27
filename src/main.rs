use chain::agents::executive_planner::ExecutivePlanner;
use dotenv::dotenv;
use server::dispatch::dispatch_incoming;
use std::io::Result;
use tokio::net::TcpListener;

use util::get_dotenv;

mod chain;
mod server;
mod util;

#[tokio::main]

async fn main() -> Result<()> {
    dotenv().ok();
    let address = format!("{}:{}", get_dotenv("HOST_NAME"), get_dotenv("HOST_PORT"));

    println!("connecting to address '{}'...", address);
    let listener = TcpListener::bind(address).await?;

    loop {
        println!("waiting for incoming connection...");
        let (socket, _) = listener.accept().await?;

        dispatch_incoming(socket).await;
    }

    // let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    // let model_description = "A big chair".to_owned();
    // let math_notes = "No comments".to_owned();

    // let mut planner = ExecutivePlanner::new(&api_key, &model_description, &math_notes).unwrap();

    // planner.run().await.unwrap();

    Ok(())
}
