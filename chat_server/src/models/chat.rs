use serde::{Deserialize, Serialize};

use crate::{error::AppError, Chat, ChatType, ChatUser};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CreateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub public: bool,
}

impl Chat {
    /// ws_id: extract from jwt token
    pub async fn create(
        input: CreateChat,
        ws_id: u64,
        pool: &sqlx::PgPool,
    ) -> Result<Self, AppError> {
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
        let chat_users = ChatUser::fetch_by_ids(&input.members, pool).await?;
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
        .fetch_one(pool)
        .await?;

        Ok::<Chat, AppError>(chat)
    }

    pub async fn fetch_all(ws_id: u64, pool: &sqlx::PgPool) -> Result<Vec<Self>, AppError> {
        let recs = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, type, members, created_at
            FROM chats
            WHERE ws_id = $1
            "#,
        )
        .bind(ws_id as i64)
        .fetch_all(pool)
        .await?;
        Ok(recs)
    }

    pub async fn fetch_by_id(id: u64, pool: &sqlx::PgPool) -> Result<Option<Self>, AppError> {
        let rec = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, type, members, created_at
            FROM chats
            WHERE id = $1
            "#,
        )
        .bind(id as i64)
        .fetch_optional(pool)
        .await?;
        Ok(rec)
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
    use crate::test_utils::get_test_pool;

    #[tokio::test]
    async fn test_create_single_chat_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;
        let input = CreateChat::new("", &[1, 2], false);
        let chat = Chat::create(input, 1, &pool).await.unwrap();
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.r#type, ChatType::Single);
        assert_eq!(chat.members, vec![1, 2]);
    }

    #[tokio::test]
    async fn test_create_public_named_chat_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;
        let input = CreateChat::new("general", &[1, 2, 3], true);
        let chat = Chat::create(input, 1, &pool).await.unwrap();
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
        assert_eq!(chat.members, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn chat_get_by_id_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;
        let chat = Chat::fetch_by_id(2, &pool).await.unwrap().unwrap();
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.r#type, ChatType::Single);
        assert_eq!(chat.members, vec![1, 3]);
        assert_eq!(chat.name, None);
    }

    #[tokio::test]
    async fn chat_get_all_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;
        let chats = Chat::fetch_all(1, &pool).await.unwrap();
        assert_eq!(chats.len(), 4);
    }
}
