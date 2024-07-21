pub mod user;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct User {
    pub id: i64,
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
            email: email.to_string(),
            fullname: fullname.to_string(),
            password_hash: Some(password.to_string()),
            created_at: Utc::now(),
        }
    }
}
