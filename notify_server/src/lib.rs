mod config;
mod error;
mod notify;
mod sse;
pub use notify::{setup_pg_listener, AppEvent};
use std::{ops::Deref, sync::Arc};

use axum::{
    middleware::from_fn_with_state,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use chat_core::{
    middleware::{auth::verify_token, TokenVerify},
    utils::jwt::DecodingKey,
    User,
};
use config::AppConfig;
use dashmap::DashMap;
use error::AppError;
use sse::sse_handler;
use tokio::sync::broadcast;

pub type UserMap = Arc<DashMap<u64, broadcast::Sender<Arc<AppEvent>>>>;
const INDEX_HTML: &str = include_str!("../index.html");

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);
pub struct AppStateInner {
    pub config: AppConfig,
    pub dk: DecodingKey,
    pub users: UserMap,
}
impl Deref for AppState {
    type Target = AppStateInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl TokenVerify for AppState {
    type Error = AppError;
    fn verify(&self, token: &str) -> Result<User, Self::Error> {
        let user = self.dk.verify(token);
        Ok(user?)
    }
}

impl AppState {
    pub fn new(config: AppConfig) -> Result<Self, AppError> {
        let decoding_pem = include_str!("../fixtures/decoding_key.pem");
        let dk = DecodingKey::load(decoding_pem)?;
        let users = Arc::new(DashMap::new());
        Ok(Self(Arc::new(AppStateInner { dk, config, users })))
    }
}

pub fn get_router() -> (Router, AppState) {
    let config = AppConfig::load().expect("failed to load config");
    let state = AppState::new(config).expect("failed to create app state");
    let router = Router::new()
        .route("/events", get(sse_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .route("/", get(index_handler))
        .with_state(state.clone());
    (router, state)
}

async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}
