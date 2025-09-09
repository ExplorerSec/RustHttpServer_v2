use std::net::{SocketAddr, ToSocketAddrs};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{protocol::resp::RespValue, server::SyncError};

pub mod resp;

pub struct Redis {
    addr: SocketAddr,
}

impl Redis {
    pub fn new(db_addr: &str) -> Result<Self, Box<SyncError>> {
        let mut addr = db_addr.to_socket_addrs()?;
        match addr.next() {
            Some(addr) => Ok(Self { addr }),
            None => Err("Err: Redis Addr Parse Error".into()),
        }
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub async fn redis_cmd(&self, cmd: Vec<String>) -> Result<Option<RespValue>, Box<SyncError>> {
        let mut stream = TcpStream::connect(self.addr).await?;
        let resp_cmd = RespValue::Array(
            cmd.into_iter()
                .map(|s| RespValue::BulkString(Some(s)))
                .collect::<Vec<_>>(),
        );
        let resp_raw = resp::RespParser::serializer(resp_cmd);
        stream.write_all(&resp_raw).await?;
        stream.flush().await?;
        let mut buf = [0u8; 128];
        let n = stream.read(&mut buf).await?;
        // 假定 Redis 返回数据包大小不超过 128
        if n <= 128 {
            let mut bytes = bytes::BytesMut::from(&buf[..n]);
            return resp::RespParser::parse(&mut bytes);
        }
        // 最可能的错误是 Redis 返回数据包过大，
        // 也可能是与 Redis 通信失败
        Err("Err:Redis_Cmd".into())
    }

    pub async fn test_user_password(&self, usr: &str, pwd: &str) -> Result<bool, Box<SyncError>> {
        if let Some(RespValue::BulkString(Some(password))) = self
            .redis_cmd(vec!["HGET".into(), "usr-pwd".into(), usr.into()])
            .await?
        {
            Ok(password.eq(pwd))
        } else {
            Ok(false)
        }
    }

    pub async fn unique_key(&self, key: &str) -> Result<bool, Box<SyncError>> {
        if let Some(RespValue::Integer(0)) =
            self.redis_cmd(vec!["EXISTS".into(), key.into()]).await?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn set_session_key(
        &self,
        key: &str,
        bind_ip: String,
        live_seconds: usize,
    ) -> Result<bool, Box<SyncError>> {
        if let Some(RespValue::SimpleString(_)) = self
            .redis_cmd(vec![
                "SET".into(),
                format!("Session-{}", key),
                bind_ip,
                "EX".into(),
                live_seconds.to_string(),
            ])
            .await?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn judge_session_key(
        &self,
        key: &str,
        bind_ip: String,
    ) -> Result<bool, Box<SyncError>> {
        
        let session_key = format!("Session-{}", key);
        if let Some(RespValue::BulkString(Some(trust_ip))) = self.redis_cmd(vec!["GET".into(), session_key]).await?{
            Ok(trust_ip==bind_ip)
        }else {
            Ok(false)
        }
        
    }
}
