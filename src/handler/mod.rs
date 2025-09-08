mod prelude;
use std::ffi::OsStr;

use prelude::*;

pub use prelude::Handler;

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
        let method = req.method.unwrap_or_default();
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

    pub async fn echo_ip(
        stream: &mut TcpStream,
        _req: &Request<'_, '_>,
    ) -> Result<(), Box<SyncError>> {
        let addr = stream.peer_addr()?.to_string();
        let response = format!(
            "HTTP/1.1 200 OK\r\n\
            Content-Type: text/plain\r\n\
            Content-Length: {}\r\n\
            Connection: close\r\n\r\n\
            {}",
            addr.len(),
            addr
        );
        stream.write(response.as_bytes()).await?;
        stream.flush().await?;
        stream.shutdown().await?;
        Ok(())
    }

    pub async fn auth(stream: &mut TcpStream, req: &Request<'_, '_>) -> Result<(), Box<SyncError>> {
        match req.method.unwrap_or_default() {
            "GET" => {}
            "POST" => {}
            _ => {
                return Self::f_404(stream, req).await;
            }
        }
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
        let file_path = std::path::Path::new(&file_path);
        println!("----> file:{}", file_path.display());
        if !file_path.exists() {
            return Self::f_404(stream, req).await;
        }
        let file_type = guess_file_mime(file_path);

        let mut file = File::open(file_path).await?;
        let file_len = file.metadata().await?.len();
        let header = format!(
            "HTTP/1.1 200 OK\r\n\
            Content-Length: {}\r\n\
            Content-Type: {}\r\n\
            Connection: close\r\n\r\n",
            file_len, file_type
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
            Content-Type: text/html\r\n\
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

fn guess_file_mime(file_path: &std::path::Path) -> &str {
    match file_path
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        // text
        "html" => "text/html",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "css" => "text/css",
        // font
        "ttf" => "font/ttf",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        // image
        "gif" => "image/gif",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "bmp" => "image/bmp",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        // audio
        "wav" => "audio/x-wav",
        "mp3" | "mpa" => "audio/mpeg",
        "m4a" => "audio/m4a",
        "ogg" => "audio/ogg",
        "aac" => "audio/aac",
        // video
        "mp4" => "video/mp4",
        "flv" => "video/x-flv",
        "avi" => "video/x-msvideo",
        // application
        "js" => "application/javascript",
        "pdf" => "application/pdf",
        "json" => "application/json",
        "xml" => "application/xml",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn f() {}
}
