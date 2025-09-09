mod server;
use server::SyncError;

mod router;

mod middleware;

mod protocol;

#[tokio::main]
async fn main() -> Result<(), Box<SyncError>> {
    let listen_addr = "127.0.0.1:25823";
    let db_addr = "127.0.0.1:6379";
    let mut server = server::Server::new(listen_addr, db_addr).await?;
    server.run().await?;

    Ok(())
}
