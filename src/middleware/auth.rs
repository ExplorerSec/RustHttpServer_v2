use httparse::Request;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::server::SyncError;

pub async fn auth(stream: &mut TcpStream, req: &Request<'_, '_>) -> Result<bool, Box<SyncError>> {
    Ok(true)
    /*
    println!("---> Auth: {}", req.path.unwrap());
    if req.path == Some("/") {
        Ok(true)
    } else {
        let body = "NOT Authoried";
        let response = format!(
            "HTTP/1.1 401 NOT Authoried\r\n\
            Content-Type: text/plain\r\n\
            Content-Length: {}\r\n\
            Connection: close\r\n\r\n\
            {}",
            body.len(),
            body
        );

        stream.write(response.as_bytes()).await?;
        stream.flush().await?;
        Ok(false)
    }*/
}
