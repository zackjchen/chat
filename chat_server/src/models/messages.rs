use super::Message;
use crate::{error::AppError, AppState, ChatFile};
use std::str::FromStr;

pub struct CreateMessage {
    pub content: String,
    pub files: Vec<String>,
}

#[allow(dead_code)]
impl AppState {
    pub async fn create_message(
        &self,
        input: CreateMessage,
        chat_id: i64,
        user_id: u64,
    ) -> Result<Message, AppError> {
        let base_dir = &self.config.server.base_dir;
        if input.content.is_empty() {
            return Err(AppError::CreateMessageError(
                "content is required".to_string(),
            ));
        }
        for s in &input.files {
            let file = ChatFile::from_str(s)?;
            if file.path(base_dir).exists() {
                return Err(AppError::CreateMessageError("file not found".to_string()));
            }
        }

        let message = sqlx::query_as(
            r#"
                INSERT INTO messages (chat_id, sender_id, content, files)
                VALUES ($1, $2, $3, $4) RETURNING id, chat_id, user_id, content, files, created_at
            "#,
        )
        .bind(chat_id)
        .bind(user_id as i64)
        .bind(input.content)
        .bind(&input.files)
        .fetch_one(&self.pool)
        .await?;
        Ok(message)
    }
}
