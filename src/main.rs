use std::io::Result;
use server::dispatch::dispatch_incoming;
use tokio::net::TcpListener;
use dotenv::dotenv;
use util::get_dotenv;

mod util;
mod server;

#[tokio::main]

async fn main() -> Result<()>{

    dotenv().ok();

    let address = format!("{}:{}", get_dotenv("HOST_NAME"), get_dotenv("HOST_PORT"));

    println!("connecting to address '{}'...", address);
    let listener = TcpListener::bind(address).await?;

    loop {
        println!("waiting for incoming connection...");
        let (socket, _) = listener.accept().await?;
        println!("dispatching incoming connection: {:?}", socket);
        dispatch_incoming(socket).await;
    }
}
