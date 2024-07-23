pub mod user;
pub mod workspace;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
    pub id: i64,
    pub ws_id: i64,
    pub fullname: String,
    pub email: String,
    #[sqlx(default)]
    #[serde(skip)]
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
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

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, PartialEq, Eq)]
pub struct ChatUser {
    pub id: i64,
    pub fullname: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, PartialEq, Eq)]
pub struct WorkSpace {
    pub id: i64,
    pub name: String,
    pub owner_id: i64,
    pub created_at: DateTime<Utc>,
}
