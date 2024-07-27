use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use tracing::info;

use crate::{chat::CreateChat, error::AppError, AppState, Chat, User};

pub(crate) async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    info!("user:{:?}", user);
    let chats = Chat::fetch_all(user.ws_id as u64, &state.pool).await?;

    Ok((StatusCode::OK, Json(chats)))
}

pub(crate) async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::create(input, user.ws_id as _, &state.pool).await?;
    Ok((StatusCode::CREATED, Json(chat)))
}

pub(crate) async fn update_chat_handler() -> impl IntoResponse {
    todo!()
}

pub(crate) async fn delete_chat_handler() -> impl IntoResponse {
    todo!()
}

pub(crate) async fn get_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::fetch_by_id(id as _, &state.pool).await?;
    match chat {
        Some(chat) => Ok((StatusCode::OK, Json(chat))),
        None => Err(AppError::NotFound(format!("chat with id {} not found", id))),
    }
}
