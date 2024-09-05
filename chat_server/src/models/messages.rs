use serde::{Deserialize, Serialize};

use crate::{error::AppError, AppState, ChatFile};
use chat_core::Message;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMessage {
    pub content: String,
    pub files: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListMessages {
    pub chat_id: i64,
    pub last_id: Option<i64>,
    pub limit: u64,
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
            if !file.path(base_dir).exists() {
                return Err(AppError::CreateMessageError(
                    "Chat File error: Invalid file url".to_string(),
                ));
            }
        }

        let message = sqlx::query_as(
            r#"
                INSERT INTO messages (chat_id, sender_id, content, files)
                VALUES ($1, $2, $3, $4) RETURNING id, chat_id, sender_id, content, files, created_at
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

    pub async fn list_messages(&self, opts: ListMessages) -> Result<Vec<Message>, AppError> {
        let last_id = opts.last_id.unwrap_or(i64::MAX);
        let messages = sqlx::query_as(
            r#"
            SELECT id, chat_id, sender_id, content, files, created_at
            FROM messages
            WHERE chat_id = $1
            AND id < $2
            ORDER BY id DESC
            LIMIT $3
            "#,
        )
        .bind(opts.chat_id)
        .bind(last_id)
        .bind(opts.limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_message_should_work() {
        let (_tdb, state) = AppState::new_for_test().await.unwrap();
        let input = CreateMessage {
            content: "hello".to_string(),
            files: vec![
                "/files/1/ce2/547/490db16893a1d5420fb0ea94010d6e96ba.png".into(),
                "/files/1/103/046/ff8c396ff66cca7a7d651117a8e3b2b97a.jpeg".into(),
                "/files/1/48a/602/0704162bf08e7e123351bfd6b4f9d61939.csv".into(),
            ],
        };
        let message = state.create_message(input, 2, 3).await.unwrap();
        assert_eq!(message.content, "hello");
        assert_eq!(message.files.len(), 3);

        let input = CreateMessage {
            content: "hello".to_string(),
            files: vec!["abc".into()],
        };
        let result = state.create_message(input, 2, 3).await.unwrap_err();
        assert_eq!(result.to_string(), "Chat File error: Invalid file url: abc");
    }

    #[tokio::test]
    async fn test_list_messages_should_work() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let opts = ListMessages {
            chat_id: 2,
            last_id: None,
            limit: 10,
        };
        let messages = state.list_messages(opts).await?;
        assert_eq!(messages.len(), 10);

        let id = messages.last().unwrap().id;
        let opts = ListMessages {
            chat_id: 2,
            last_id: Some(id),
            limit: 10,
        };
        let messages = state.list_messages(opts).await?;
        assert_eq!(messages.len(), 10);

        let id = messages.last().unwrap().id;
        let opts = ListMessages {
            chat_id: 2,
            last_id: Some(id),
            limit: 10,
        };
        let messages = state.list_messages(opts).await?;
        assert_eq!(messages.len(), 4);
        assert_eq!(messages.last().unwrap().id, 1);
        assert_eq!(messages.last().unwrap().content, "hello1".to_string());
        Ok(())
    }
}
