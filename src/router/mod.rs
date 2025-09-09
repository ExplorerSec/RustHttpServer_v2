mod handler;
use handler::Handler;
mod prelude;
use crate::server::SyncError;
use bytes::BytesMut;
use httparse::Request;
use tokio::net::TcpStream;

pub async fn route(
    stream: &mut TcpStream,
    req_headers: &Request<'_, '_>,
    body: BytesMut,
) -> Result<(), Box<SyncError>> {
    match &req_headers.path {
        Some("/") => Handler::f1(stream, req_headers).await?,
        Some("/method") => Handler::echo_method(stream, req_headers).await?,
        Some("/ip") => Handler::echo_ip(stream, req_headers).await?,
        Some("/404") => Handler::f_404(stream, req_headers).await?,
        Some("/srs/login") => Handler::login(stream, req_headers, body).await?,
        _ => Handler::file(stream, req_headers).await?,
    };

    Ok(())
}
