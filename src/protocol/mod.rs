use std::net::SocketAddr;

use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};

use crate::{protocol::resp::RespValue, server::SyncError};

pub mod resp;

pub struct Redis {
    addr:SocketAddr
}

impl Redis {
    pub async fn redis_cmd(&self, cmd:String) -> Result<Option<RespValue>, Box<SyncError>> {
        let mut stream = TcpStream::connect(self.addr).await?;
        let cmd = resp::RespParser::serializer(resp::RespValue::BulkString(Some(cmd)));
        stream.write_all(&cmd).await?;
        stream.flush().await?;
        let mut buf = [0u8;128];
        let n = stream.read(&mut buf).await?;
        // 假定 Redis 返回数据包大小不超过 128
        if n<=128{
            let mut bytes = bytes::BytesMut::from(&buf[..n]);
            return resp::RespParser::parse(&mut bytes); 
        }

        Err("Redis 返回数据包过大".into()) 
    }
}
