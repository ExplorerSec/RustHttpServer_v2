mod server;
use server::SyncError;

mod router;

mod middleware;

mod protocol;

#[tokio::main]
async fn main() -> Result<(), Box<SyncError>> {
    let addr = "127.0.0.1:25823";
    let mut server = server::Server::new(addr).await?;
    let _ = server.run().await;

    Ok(())
}
