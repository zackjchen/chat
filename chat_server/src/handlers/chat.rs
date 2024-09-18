use crate::{chat::CreateChat, error::AppError, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use chat_core::User;
use tracing::info;

/// list all chat
#[utoipa::path(
    get,
    path = "/api/chat",
    responses(
        (status=200, description="List of chats", body=[Chat]),
    ),
    security(("token" = [])),
    tag = "chat"
)]
pub(crate) async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    info!("user:{:?}", user);
    let chats = state.fetch_chats_all(user.ws_id as u64).await?;

    Ok((StatusCode::OK, Json(chats)))
}

/// create a new chat
#[utoipa::path(
    post,
    path = "/api/chat",
    request_body = CreateChat,
    responses(
        (status=200, description="List of chats", body=[Chat]),
    ),
    security(("token" = [])),
    tag = "chat"
)]
pub(crate) async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.create_chat(input, user.ws_id as _).await?;
    Ok((StatusCode::CREATED, Json(chat)))
}

// TODO: update chat
pub(crate) async fn update_chat_handler() -> impl IntoResponse {
    ""
}

// TODO: delete chat
pub(crate) async fn delete_chat_handler() -> impl IntoResponse {
    ""
}

/// get a chat
#[utoipa::path(
    get,
    path = "/api/chat/{id}",
    params(
        ("id"=i64, Path, description="chat id"),
        ("access_token"=inline(Option<String>),Query ,
            description="access_token, if passed, will be used to authenticate the request, alternatively, you can use the Authorization header")
    ),
    responses(
        (status=200, description="chat details", body=Chat),
    ),
    security(("token" = [])),
    tag = "chat"
)]
pub(crate) async fn get_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.fetch_chat_by_id(id as _).await?;
    match chat {
        Some(chat) => Ok((StatusCode::OK, Json(chat))),
        None => Err(AppError::NotFound(format!("chat with id {} not found", id))),
    }
}
