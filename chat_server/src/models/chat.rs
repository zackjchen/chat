use crate::{error::AppError, AppState};
use chat_core::{Chat, ChatType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CreateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub public: bool,
}

impl AppState {
    /// ws_id: extract from jwt token
    pub async fn create_chat(&self, input: CreateChat, ws_id: u64) -> Result<Chat, AppError> {
        let len = input.members.len();
        if len < 2 {
            return Err(AppError::CreateChatError(
                "members must be more than 2".to_string(),
            ));
        }
        if len > 8 && input.name.is_none() {
            return Err(AppError::CreateChatError(
                "Group chat with more than 8, so name is required".to_string(),
            ));
        }
        // verity all members is exist
        let chat_users = self.fetch_chat_user_by_ids(&input.members).await?;
        if chat_users.len() != len {
            return Err(AppError::CreateChatError(
                "Some members not exist".to_string(),
            ));
        }

        let chat_type = match (&input.name, len) {
            (None, 2) => ChatType::Single,
            (None, _) => ChatType::Group,
            (Some(_), _) => {
                if input.public {
                    ChatType::PublicChannel
                } else {
                    ChatType::PrivateChannel
                }
            }
        };
        // r#"
        // INSERT INTO chats (ws_id, name, type, members)
        // VALUES ($1, $2, $3, $4)
        // RETURNING id, ws_id, name, type, members, created_at
        // "#,
        let chat = sqlx::query_as(
            r#"
            INSERT INTO chats (ws_id, name, type, members)
            VALUES ($1, $2, $3, $4)
            RETURNING id, ws_id, name, type, members, created_at
        "#,
        )
        .bind(ws_id as i64)
        .bind(input.name)
        .bind(chat_type)
        .bind(&input.members)
        .fetch_one(&self.pool)
        .await?;

        Ok::<Chat, AppError>(chat)
    }

    pub async fn fetch_chats_all(&self, ws_id: u64) -> Result<Vec<Chat>, AppError> {
        let recs = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, type, members, created_at
            FROM chats
            WHERE ws_id = $1
            "#,
        )
        .bind(ws_id as i64)
        .fetch_all(&self.pool)
        .await?;
        Ok(recs)
    }

    pub async fn fetch_chat_by_id(&self, id: u64) -> Result<Option<Chat>, AppError> {
        let rec = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, type, members, created_at
            FROM chats
            WHERE id = $1
            "#,
        )
        .bind(id as i64)
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }

    pub async fn is_chat_member(&self, chat_id: i64, user_id: u64) -> Result<bool, AppError> {
        let is_member = sqlx::query(r#"select 1 cnt from chats where id=$1 and $2 = any(members)"#)
            .bind(chat_id)
            .bind(user_id as i64)
            .fetch_optional(&self.pool)
            .await?;
        Ok(is_member.is_some())
    }
}

#[cfg(test)]
impl CreateChat {
    fn new(name: &str, members: &[i64], public: bool) -> Self {
        let name = if name.is_empty() {
            None
        } else {
            Some(name.into())
        };
        Self {
            name,
            members: members.to_vec(),
            public,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_single_chat_should_work() -> anyhow::Result<()> {
        // let (_tdb, pool) = get_test_pool(None).await;
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateChat::new("", &[1, 2], false);
        let chat = state.create_chat(input, 1).await.unwrap();
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.r#type, ChatType::Single);
        assert_eq!(chat.members, vec![1, 2]);
        Ok(())
    }

    #[tokio::test]
    async fn test_create_public_named_chat_should_work() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateChat::new("test-chat", &[2, 3, 4], true);
        let chat = state.create_chat(input, 2).await.unwrap();
        assert_eq!(chat.ws_id, 2);
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
        assert_eq!(chat.members, vec![2, 3, 4]);
        Ok(())
    }

    #[tokio::test]
    async fn chat_get_by_id_should_work() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let chat = state.fetch_chat_by_id(2).await.unwrap().unwrap();
        assert_eq!(chat.ws_id, 2);
        assert_eq!(chat.r#type, ChatType::Group);
        assert_eq!(chat.members, vec![2, 3, 4, 5]);
        assert_eq!(chat.name, Some("聊天室1".into()));
        Ok(())
    }

    #[tokio::test]
    async fn chat_get_all_should_work() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let chats = state.fetch_chats_all(2).await.unwrap();
        assert_eq!(chats.len(), 4);

        Ok(())
    }

    use crate::AppState;

    #[tokio::test]
    async fn test_is_chat_member() {
        let (_tpg, state) = AppState::new_for_test().await.unwrap();
        // 注意看下这里的chat id
        let res = state.is_chat_member(2, 2).await.unwrap();
        assert!(res);

        let res = state.is_chat_member(2, 6).await.unwrap();
        assert!(!res);
    }
}
