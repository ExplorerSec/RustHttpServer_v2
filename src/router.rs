use crate::handler::Handler;
use crate::server::SyncError;
use httparse::Request;
use tokio::net::TcpStream;

pub async fn route(stream: &mut TcpStream, req: &Request<'_, '_>) -> Result<(), Box<SyncError>> {
    match &req.path {
        Some("/") => Handler::f1(stream, req).await?,
        Some("/method") => Handler::echo_method(stream, req).await?,
        Some("/ip") =>Handler::echo_ip(stream, req).await?,
        Some("/404") => Handler::f_404(stream, req).await?,
        _ => Handler::file(stream, req).await?,
    };

    Ok(())
}
