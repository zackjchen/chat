use anyhow::Result;
use chat_core::{Chat, Message};
use futures::StreamExt;
use reqwest::{
    multipart::{Form, Part},
    StatusCode,
};
use reqwest_eventsource::Event;
use reqwest_eventsource::EventSource;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{net::SocketAddr, time::Duration};
use tokio::time::sleep;
const WILD_ADDRESS: &str = "0.0.0.0:0";

struct ChatServer {
    addr: SocketAddr,
    token: String,
    client: reqwest::Client,
}
struct NotifyServer;

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

    async fn create_chat(&self) -> Result<Chat> {
        let url = format!("http://{}/api/chat", self.addr);
        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.token))
            .body(r#"{"name": "zack group","members": [2,3,4,5,6],"public": false}"#)
            .send()
            .await?;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let chat: Chat = resp.json().await?;
        Ok(chat)
    }

    async fn create_message(&self, chat_id: u64) -> Result<Message> {
        let data = include_str!("../Cargo.toml");
        let files = Part::text(data)
            .file_name("Cargo.toml")
            .mime_str("text/plain")?;
        let form = Form::new().part("file", files);
        let rep = self
            .client
            .post(&format!("http://{}/api/upload", self.addr))
            .header("Authorization", format!("Bearer {}", self.token))
            .multipart(form)
            .send()
            .await?;
        assert_eq!(rep.status(), StatusCode::OK);
        let file_urls: Vec<String> = rep.json().await?;

        let body = json!(
            {
                "content": "hello",
                "files": file_urls
            }
        )
        .to_string();
        let res = self
            .client
            .post(&format!(
                "http://{}/api/chat/{}/messages",
                self.addr, chat_id
            ))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.token))
            .body(body)
            .send()
            .await?;
        assert_eq!(res.status(), StatusCode::OK);
        let message: Message = res.json().await?;
        assert_eq!(message.content, "hello");
        assert_eq!(message.files, file_urls);
        Ok(message)
    }
}

impl NotifyServer {
    async fn new(db_url: &str, token: &str) -> anyhow::Result<Self> {
        let listener = tokio::net::TcpListener::bind(WILD_ADDRESS).await?;
        let addr = listener.local_addr()?;
        let mut config = notify_server::AppConfig::load().expect("failed to load config");
        config.server.db_url = db_url.to_string();
        println!("config db_url: {:?}", config.server.db_url);
        let app = notify_server::get_router(config).await?;
        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
        });
        let mut es = EventSource::get(format!("http://{}/events?access_token={}", addr, token));

        tokio::spawn(async move {
            while let Some(event) = es.next().await {
                println!("event: {:?}", event);
                match event {
                    Ok(Event::Open) => {
                        println!("Connection Open");
                    }
                    Ok(Event::Message(msg)) => {
                        println!("event: {:?}", msg);
                    }
                    Err(e) => {
                        println!("error: {:?}", e);
                        es.close();
                    }
                }
            }
        });
        Ok(Self)
    }
}

#[tokio::test]
async fn chat_server_should_work() -> anyhow::Result<()> {
    // _tdb is 需要整个生命周期都存在，所以不能用放到new里面，不然会被drop
    let (tdb, state) = chat_server::AppState::new_for_test().await?;
    let server = ChatServer::new(state).await?;
    println!("server: {:?}", tdb.url());
    NotifyServer::new(&tdb.url(), &server.token).await?;
    let chat = server.create_chat().await?;
    // println!("chat: {:?}", chat);
    let _message = server.create_message(chat.id as u64).await?;
    sleep(Duration::from_secs(1)).await;

    Ok(())
}
