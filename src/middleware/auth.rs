use std::sync::Arc;

use httparse::Request;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::{protocol::Redis, server::SyncError};

pub async fn auth(
    stream: &mut TcpStream,
    db: Arc<Redis>,
    req: &Request<'_, '_>,
) -> Result<bool, Box<SyncError>> {
    // SRS 目录不需要特殊权限就可以进入
    let path = req.path.unwrap();
    if path.to_uppercase().starts_with("/SRS") && !path.contains("..") {
        return Ok(true);
    }
    // 否则进行鉴权
    let cookie_header = req
        .headers
        .iter()
        .find(|h| h.name.eq_ignore_ascii_case("Cookie"))
        .and_then(|h| str::from_utf8(h.value).ok());

    if let Some(cookie_raw) = cookie_header {
        let cookies = cookie::Cookie::split_parse(cookie_raw);
        for cookie in cookies {
            let cookie = cookie?;
            if cookie.name() == "key"
                && db
                    .judge_session_key(cookie.value_trimmed(), stream.peer_addr()?.ip().to_string())
                    .await?
            {
                return Ok(true);
            }
        }
    }
    // 鉴权失败
    let body = "<!DOCTYPE html><html><head><meta http-equiv=\"refresh\" content=\"4; url=/srs/LoginInterface.html\"><title>NOT Authoried</title></head><body><h1>NOT Authoried</h1></body></html>";
    let response = format!(
        "HTTP/1.1 401 NOT Authoried\r\n\
            Content-Type: text/html\r\n\
            Content-Length: {}\r\n\
            Connection: close\r\n\r\n\
            {}",
        body.len(),
        body
    );

    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;
    Ok(false)
}
