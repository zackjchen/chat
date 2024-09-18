use crate::{error::AppError, AppState};
use axum::{extract::State, response::IntoResponse, Extension, Json};
use chat_core::User;

/// list all users under workspace
#[utoipa::path(
    get,
    path = "/api/users",
    responses(
        (status = 200, description = "List of chat users", body = [User]),
    ),
    security(("token" = [])),
    tag = "chat"
)]

pub(crate) async fn list_workspace_users_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let users = state.fetch_workspace_all_users(user.id as u64).await?;

    Ok(Json(users))
}
