use axum::response::IntoResponse;
pub mod auth;
pub mod chat;
pub mod messages;
pub mod workspace;

pub(crate) async fn index_handler() -> impl IntoResponse {
    todo!()
}
