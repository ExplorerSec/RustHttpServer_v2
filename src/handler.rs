use httparse::Request;
use std::path;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::server::SyncError;

pub struct Handler;

impl Handler {
    pub async fn f1(stream: &mut TcpStream, _req: &Request<'_, '_>) -> Result<(), Box<SyncError>> {
        let body = "Hello, World!";
        let response = format!(
            "HTTP/1.1 200 OK\r\n\
            Content-Type: text/plain\r\n\
            Content-Length: {}\r\n\
            Connection: close\r\n\r\n\
            {}",
            body.len(),
            body
        );
        stream.write(response.as_bytes()).await?;
        stream.flush().await?;
        stream.shutdown().await?;
        Ok(())
    }

    pub async fn echo_method(
        stream: &mut TcpStream,
        req: &Request<'_, '_>,
    ) -> Result<(), Box<SyncError>> {
        let method = req.method.unwrap();
        let response = format!(
            "HTTP/1.1 200 OK\r\n\
            Content-Type: text/plain\r\n\
            Content-Length: {}\r\n\
            Connection: close\r\n\r\n\
            {}",
            method.len(),
            method
        );
        stream.write(response.as_bytes()).await?;
        stream.flush().await?;
        stream.shutdown().await?;
        Ok(())
    }

    pub async fn file(stream: &mut TcpStream, req: &Request<'_, '_>) -> Result<(), Box<SyncError>> {
        let http_path = req.path.unwrap_or_default();
        // 过滤掉可能越权访问上级目录的情况
        let decoded_path = percent_encoding::percent_decode(http_path.as_bytes())
            .decode_utf8()
            .unwrap_or_default();

        if !decoded_path.starts_with('/') || decoded_path.contains("..") {
            return Self::f_404(stream, req).await;
        }
        let file_path = format!("static{}", decoded_path);
        let file_path = path::Path::new(&file_path);
        println!("----> file:{}", file_path.display());
        if !file_path.exists() {
            return Self::f_404(stream, req).await;
        }
        let mut file = File::open(file_path).await?;
        let file_len = file.metadata().await?.len();
        let header = format!(
            "HTTP/1.1 200 OK\r\n\
            Content-Length: {}\r\n\
            Content-Type: application/octet-stream\r\n\
            Connection: close\r\n\r\n",
            file_len
        );
        stream.write(header.as_bytes()).await?;
        tokio::io::copy(&mut file, stream).await?;
        stream.flush().await?;
        stream.shutdown().await?;
        Ok(())
    }

    pub async fn f_404(
        stream: &mut TcpStream,
        req: &Request<'_, '_>,
    ) -> Result<(), Box<SyncError>> {
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
        stream.write(response.as_bytes()).await?;
        stream.flush().await?;
        stream.shutdown().await?;
        Ok(())
    }
}
