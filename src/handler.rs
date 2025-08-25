use bytes::BytesMut;
use httparse::Request;

use crate::server::SyncError;

pub struct Handler;

impl Handler {
    pub async fn f1(_req: &Request<'_, '_>, buf: &mut BytesMut) -> Result<(), Box<SyncError>> {
        let body = "Hello, httparse!";
        let response = format!(
            "HTTP/1.1 200 OK\r\n\
            Content-Type: text/plain\r\n\
            Content-Length: {}\r\n\
            Connection: close\r\n\r\n\
            {}",
            body.len(),
            body
        );

        buf.extend_from_slice(response.as_bytes());
        Ok(())
    }

    pub async fn f2(req: &Request<'_, '_>, buf: &mut BytesMut) -> Result<(), Box<SyncError>> {
        let body = req.method.unwrap();
        let response = format!(
            "HTTP/1.1 200 OK\r\n\
            Content-Type: text/plain\r\n\
            Content-Length: {}\r\n\
            Connection: close\r\n\r\n\
            {}",
            body.len(),
            body
        );

        buf.extend_from_slice(response.as_bytes());
        Ok(())
    }

    pub async fn f_404(req: &Request<'_, '_>, buf: &mut BytesMut) -> Result<(), Box<SyncError>> {
        let body = format!("Not Found Path: {}", req.path.unwrap_or_default());
        let response = format!(
            "HTTP/1.1 404 NOT FOUND\r\n\
            Content-Type: text/plain\r\n\
            Content-Length: {}\r\n\
            Connection: close\r\n\r\n\
            {}",
            body.len(),
            body
        );

        buf.extend_from_slice(response.as_bytes());
        Ok(())
    }

}
