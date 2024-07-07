pub mod config;
pub mod handlers;
use std::{ops::Deref, sync::Arc};

use axum::{
    routing::{get, patch, post},
    Router,
};
pub use config::*;
use handlers::{
    auth::*,
    chat::*,
    index_handler,
    messages::{list_message_handler, send_message_handler},
};

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    inner: Arc<AppStateInner>,
}

#[allow(unused)]
#[derive(Debug)]
pub struct AppStateInner {
    pub(crate) config: AppConfig,
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            inner: Arc::new(AppStateInner { config }),
        }
    }
}

pub fn get_router(config: AppConfig) -> Router {
    let state = AppState::new(config);
    let api = Router::new()
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler))
        .route(
            "/chat",
            get(list_chat_handler)
                .post(create_chat_handler)
                .patch(update_chat_handler),
        )
        .route(
            "/chat/:id",
            patch(update_chat_handler)
                .delete(delete_chat_handler)
                .post(send_message_handler),
        )
        .route("/chat/:id/messages", get(list_message_handler));

    Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state)
}
