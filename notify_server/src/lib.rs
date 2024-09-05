mod sse;
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use chat_core::{Chat, Message};
use sqlx::postgres::PgListener;
use sse::sse_handler;
use tokio_stream::StreamExt;
use tracing::info;

const INDEX_HTML: &str = include_str!("../index.html");

pub enum Event {
    NewChat(Chat),
    AddToChat(Chat),
    RemoveFromChat(Chat),
    NewMessage(Message),
}

pub fn get_router() -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/events", get(sse_handler))
}

async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}

pub async fn setup_pg_listener() -> anyhow::Result<()> {
    let mut listener =
        PgListener::connect("postgresql://zackjchen:postgres@localhost:5432/chat").await?;
    listener.listen("chat_updated").await?;
    listener.listen("message_added").await?;

    let mut stream = listener.into_stream();

    tokio::spawn(async move {
        while let Some(Ok(notify)) = stream.next().await {
            info!("Received notification: {:?}", notify);
        }
    });
    Ok(())
}
