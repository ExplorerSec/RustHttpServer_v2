use std::sync::Arc;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

pub(crate) type SyncError = dyn std::error::Error + Send + Sync;

use crate::{middleware::auth::auth, router::route};

struct DataBase;

impl DataBase {
    pub fn new() -> DataBase {
        DataBase
    }
}

pub struct Server {
    listener: TcpListener,
    inner_db: Arc<Mutex<DataBase>>,
}

impl Server {
    pub async fn new(addr: &str) -> Result<Self, Box<SyncError>> {
        let listener = TcpListener::bind(addr).await?;
        let server = Server {
            listener: listener,
            inner_db: Arc::new(Mutex::new(DataBase::new())),
        };
        Ok(server)
    }

    pub async fn run(&mut self) -> Result<(), Box<SyncError>> {
        let env_path = std::env::current_dir().expect("无法获取程序运行环境路径");
        println!("Server running Env: {}", env_path.display());
        println!("Server running on http://{}", self.listener.local_addr()?);
        loop {
            let (stream, client_addr) = self.listener.accept().await?;
            let db = self.inner_db.clone();
            let _ = tokio::spawn(async move {
                if let Err(err) = Self::handle_connection(stream, db).await {
                    eprintln!("Error to handle connection from {}:{}", client_addr, err);
                }
            });
        }

        // Ok(())
    }

    async fn handle_connection(
        mut stream: TcpStream,
        db: Arc<Mutex<DataBase>>,
    ) -> Result<(), Box<SyncError>> {
        let mut tmpbuff = [0u8; 2048];
        let n = stream.read(&mut tmpbuff).await?;
        #[cfg(debug_assertions)]
        println!("http package size:{}", n);
        // 解析 HTTP 请求
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);
        let _res = req.parse(&mut tmpbuff)?;

        // 接下来是 中间件（权限认证） 和 业务逻辑
        // 中间件
        if auth(&mut stream, &req).await? {
            // 业务逻辑-路由
            route(&mut stream, &req).await?;
        }
        stream.shutdown().await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn httparse_use() {
        let buf = b"GET /404 HTTP/1.1\r\nHost:";
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);
        let res = req.parse(buf).unwrap();
        println!("{:?}", res);
        println!("{:?}", req);
    }
}
