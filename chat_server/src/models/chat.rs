use crate::{error::AppError, Chat, ChatType};

pub struct CreateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
}

impl Chat {
    pub async fn create(
        input: CreateChat,
        ws_id: u64,
        pool: &sqlx::PgPool,
    ) -> Result<Self, AppError> {
        let chat = sqlx::query_as(
            r#"
            INSERT INTO chats (ws_id, name, type, members)
            VALUES ($1, $2, $3, #4)
            RETURNING id, ws_id, name, type, members, created_at
        "#,
        )
        .bind(ws_id as i64)
        .bind(input.name)
        .bind(ChatType::Group)
        .bind(&input.members)
        .fetch_one(pool)
        .await?;

        Ok::<Chat, AppError>(chat)
    }
}
