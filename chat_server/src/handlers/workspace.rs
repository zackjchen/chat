use axum::{extract::State, response::IntoResponse, Extension, Json};

use crate::{error::AppError, AppState, User};

pub(crate) async fn list_chat_users_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let users = state.fetch_workspace_all_users(user.id as u64).await?;

    Ok(Json(users))
}
