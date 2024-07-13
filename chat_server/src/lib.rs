pub mod config;
pub mod error;
pub mod handlers;
pub mod models;
pub mod utils;
use error::AppError;
use models::*;
use std::{fmt::Debug, ops::Deref, sync::Arc};
use utils::jwt::{DecodingKey, EncodingKey};

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
pub struct AppStateInner {
    pub(crate) config: AppConfig,
    pub(crate) ek: EncodingKey,
    pub(crate) dk: DecodingKey,
    pub(crate) pool: sqlx::PgPool,
}

impl Debug for AppStateInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .finish()
    }
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        let ek = EncodingKey::load(&config.auth.sk).expect("load encoding key failed");
        let dk = DecodingKey::load(&config.auth.pk).expect("load decoding key failed");
        let pool = sqlx::PgPool::connect(&config.server.db_url).await?;
        Ok(Self {
            inner: Arc::new(AppStateInner {
                config,
                ek,
                dk,
                pool,
            }),
        })
    }
}

pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
    let state = AppState::try_new(config).await?;
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

    Ok(Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state))
}

#[cfg(test)]
impl AppState {
    #[cfg(test)]
    pub async fn new_for_test(
        config: AppConfig,
    ) -> Result<(sqlx_db_tester::TestPg, Self), AppError> {
        use sqlx_db_tester::TestPg;

        let ek = EncodingKey::load(&config.auth.sk).expect("load encoding key failed");
        let dk = DecodingKey::load(&config.auth.pk).expect("load decoding key failed");
        let server_url = config.server.db_url.split('/').next().unwrap().into();
        // 当缺省的时候它应该自己会补齐这个 postgre://zackjchen:postgres@localhost:5432
        assert_eq!(server_url, "postgre:");
        let tgp: TestPg = TestPg::new(server_url, std::path::Path::new("../migrations"));
        let pool = tgp.get_pool().await;
        let state = Self {
            inner: Arc::new(AppStateInner {
                config,
                ek,
                dk,
                pool,
            }),
        };
        Ok((tgp, state))
    }
}
