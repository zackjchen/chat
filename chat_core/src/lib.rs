pub mod middleware;
pub mod utils;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct User {
    pub id: i64,
    pub ws_id: i64,
    pub fullname: String,
    pub email: String,
    #[sqlx(default)]
    #[serde(skip)]
    pub password_hash: Option<String>,
    #[schema(value_type=String)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, PartialEq, Eq, ToSchema)]
pub struct WorkSpace {
    pub id: i64,
    pub name: String,
    pub owner_id: i64,
    #[schema(value_type=String)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, PartialEq, Eq, ToSchema)]
pub struct Chat {
    pub id: i64,
    pub ws_id: i64,
    pub r#type: ChatType,
    pub name: Option<String>,
    pub members: Vec<i64>,
    #[schema(value_type=String)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, PartialOrd, Eq, ToSchema)]
#[sqlx(type_name = "chat_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ChatType {
    Single,
    Group,
    PrivateChannel,
    PublicChannel,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, PartialEq, Eq, ToSchema)]
pub struct Message {
    pub id: i64,
    pub chat_id: i64,
    pub sender_id: i64,
    pub content: String,
    pub files: Vec<String>,
    #[schema(value_type=String)]
    pub created_at: DateTime<Utc>,
}

// #[cfg(test)]
impl User {
    pub fn new(id: i64, email: &str, fullname: &str, password: &str) -> Self {
        Self {
            id,
            ws_id: 0,
            email: email.to_string(),
            fullname: fullname.to_string(),
            password_hash: Some(password.to_string()),
            created_at: Utc::now(),
        }
    }
}
