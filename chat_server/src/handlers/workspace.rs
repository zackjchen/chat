use axum::{extract::State, response::IntoResponse, Extension, Json};

use crate::{error::AppError, AppState, User, WorkSpace};

pub(crate) async fn list_chat_users_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let users = WorkSpace::fetch_all_users(user.id as u64, &state.pool).await?;

    Ok(Json(users))
}
