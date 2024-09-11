pub mod config;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;
use axum::{
    extract::DefaultBodyLimit,
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use chat_core::{
    middleware::{auth::verify_token, set_layer, TokenVerify},
    utils::jwt::{DecodingKey, EncodingKey},
    User,
};
pub use config::*;
use error::AppError;
use handlers::{
    auth::*,
    chat::*,
    index_handler,
    messages::{download_file_handler, list_message_handler, send_message_handler, upload_handler},
    workspace::list_chat_users_handler,
};
use middleware::chat::verify_chat;
use models::*;
use std::{fmt::Debug, ops::Deref, sync::Arc};
use tokio::fs;

#[derive(Debug, Clone)]
pub struct AppState {
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
impl TokenVerify for AppState {
    type Error = AppError;

    fn verify(&self, token: &str) -> Result<User, Self::Error> {
        let user = self.dk.verify(token)?;
        Ok(user)
    }
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        fs::create_dir_all(&config.server.base_dir).await?;
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

pub async fn get_router(state: AppState) -> Result<Router, AppError> {
    let chat_router = Router::new()
        .route(
            "/:id",
            get(get_chat_handler)
                .patch(update_chat_handler)
                .delete(delete_chat_handler)
                .post(send_message_handler),
        )
        .route(
            "/:id/messages",
            get(list_message_handler).post(send_message_handler),
        )
        .layer(from_fn_with_state(state.clone(), verify_chat))
        .route(
            "/",
            get(list_chat_handler)
                .post(create_chat_handler)
                .patch(update_chat_handler),
        );

    let api = Router::new()
        .route("/users", get(list_chat_users_handler))
        .nest("/chat", chat_router)
        .route(
            "/upload",
            post(upload_handler).layer(DefaultBodyLimit::max(50 * 1024 * 1024)),
        )
        .route("/files/:ws_id/*path", get(download_file_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        // 这里是因为登陆和注册还没有token，所以不需要验证token
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler));

    let app = Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state);

    Ok(set_layer(app))
}

#[cfg(feature = "test-util")]
pub mod test_utils {
    use super::*;
    use sqlx::{Executor, PgPool};
    use sqlx_db_tester::TestPg;
    impl AppState {
        // #[cfg(test)]
        pub async fn new_for_test() -> Result<(TestPg, Self), AppError> {
            let config = AppConfig::load()?;
            let ek = EncodingKey::load(&config.auth.sk).expect("load encoding key failed");
            let dk = DecodingKey::load(&config.auth.pk).expect("load decoding key failed");
            let index = config.server.db_url.rfind('/').expect("invalid db_url");
            let server_url = &config.server.db_url[..index];
            println!("server_url: {:?}", server_url);
            // server_url postgre://zackjchen:postgres@localhost:5432
            let (tdb, pool) = get_test_pool(Some(server_url)).await;
            let state = Self {
                inner: Arc::new(AppStateInner {
                    config,
                    ek,
                    dk,
                    pool,
                }),
            };
            Ok((tdb, state))
        }
    }
    async fn get_test_pool(url: Option<&str>) -> (TestPg, PgPool) {
        let url = if let Some(url) = url {
            url
        } else {
            "postgre://zackjchen:postgres@localhost:5432"
        };
        let tdb: TestPg = TestPg::new(url.into(), std::path::Path::new("../migrations"));
        let pool = tdb.get_pool().await;

        let sqls = include_str!("../fixtures/test.sql").split(';');
        let mut ts = pool.begin().await.expect("Begin transaction failed");
        for sql in sqls {
            if sql.trim().is_empty() {
                continue;
            }
            ts.execute(sql).await.expect("Execute sql failed");
        }
        ts.commit().await.expect("Commit transaction failed");
        (tdb, pool)
    }
}
