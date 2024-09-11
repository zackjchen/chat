use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

const WILD_ADDRESS: &str = "0.0.0.0:0";

struct ChatServer {
    addr: SocketAddr,
    token: String,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize, Serialize)]
struct AuthToken {
    token: String,
}

impl ChatServer {
    async fn new(state: chat_server::AppState) -> anyhow::Result<Self> {
        let listener = tokio::net::TcpListener::bind(WILD_ADDRESS).await?;
        let addr = listener.local_addr()?;
        let app = chat_server::get_router(state).await?;
        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
        });

        let client = reqwest::Client::new();
        let mut ret = Self {
            addr,
            token: "".to_string(),
            client,
        };
        ret.signin().await?;
        Ok(ret)
    }

    async fn signin(&mut self) -> anyhow::Result<String> {
        let url = format!("http://{}/api/signin", self.addr);
        println!("url: {:?}", url);

        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(r#"{"email": "zack.j.chen@hkjc.org.hk","password": "Jiajia520"}"#)
            .send()
            .await?;
        assert_eq!(resp.status(), 200);
        let token: AuthToken = resp.json().await?;
        self.token = token.token;
        Ok(self.token.clone())
    }
}
#[tokio::test]
async fn chat_server_should_work() -> anyhow::Result<()> {
    // _tdb is 需要整个生命周期都存在，所以不能用放到new里面，不然会被drop
    let (_tdb, state) = chat_server::AppState::new_for_test().await?;
    let _server = ChatServer::new(state).await?;
    println!("server:");
    Ok(())
}
