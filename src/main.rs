use chain::agents::onpy_agent::OnPyAgent;
use dotenv::dotenv;
use server::dispatch::dispatch_incoming;
use std::io::Result;
use textwrap::dedent;
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

    // let output = OnPyAgent::execute_block(
    //     &dedent(
    //         r###"

    //         print("Hello world!")

    //         "###
    //     )
    //     , "123").await.unwrap();

    // println!("output: {output}");

    Ok(())
}
