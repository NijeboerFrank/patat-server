mod patat_connection;
mod patat_participant;
mod server;

use anyhow::Result;

fn main() -> Result<()> {
    println!("Starting the UDP server...");
    let patat_server = server::Server::new();
    patat_server
        .run_server()
        .expect("Something went wrong on the server");
    Ok(())
}
