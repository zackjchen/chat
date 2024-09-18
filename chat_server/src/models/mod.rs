use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;

pub mod chat;
mod file;
pub mod messages;
pub mod user;
pub mod workspace;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ChatFile {
    pub ws_id: u64,
    pub ext: String,
    pub hash: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, PartialEq, Eq, ToSchema)]
pub struct ChatUser {
    pub id: i64,
    pub fullname: String,
    pub email: String,
}
