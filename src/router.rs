use crate::handler::Handler;
use crate::server::SyncError;
use httparse::Request;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use bytes::BytesMut;

pub async fn route(stream: &mut TcpStream, req: &Request<'_, '_>) -> Result<(), Box<SyncError>> {
    let mut buff = BytesMut::with_capacity(1024);

    match &req.path {
        Some("/") => Handler::f1(&req, &mut buff).await?,
        Some("/f2") => Handler::f2(&req, &mut buff).await?,
        _ => Handler::f_404(&req, &mut buff).await?,
    };

    stream.write(&buff).await?;
    stream.flush().await?;
    Ok(())
}
