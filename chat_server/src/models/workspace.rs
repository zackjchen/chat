use crate::{error::AppError, ChatUser};

use super::WorkSpace;
impl WorkSpace {
    pub async fn create(name: &str, user_id: u64, pool: &sqlx::PgPool) -> Result<Self, AppError> {
        let ws =
            sqlx::query_as("insert into workspace (name, owner_id) values ($1, $2) returning *")
                .bind(name)
                .bind(user_id as i64)
                .fetch_one(pool)
                .await?;

        Ok::<WorkSpace, AppError>(ws)
    }

    pub async fn find_by_name(
        name: impl Into<String>,
        pool: &sqlx::PgPool,
    ) -> Result<Option<Self>, AppError> {
        let ws = sqlx::query_as("select * from workspace where name = $1")
            .bind(name.into())
            .fetch_optional(pool)
            .await?;
        Ok(ws)
    }

    pub async fn find_by_email(
        email: impl Into<String>,
        pool: &sqlx::PgPool,
    ) -> Result<Option<Self>, AppError> {
        let ws = sqlx::query_as("select * from workspace where email = &1")
            .bind(email.into())
            .fetch_optional(pool)
            .await?;
        Ok(ws)
    }
    pub async fn update_owner(&self, owner_id: u64, pool: &sqlx::PgPool) -> Result<Self, AppError> {
        let ws = sqlx::query_as(
            r#"
                update workspace set owner_id = $1
                where id = $2
                and id = (select ws_id from users where id = $1)
                returning *
            "#,
        )
        .bind(owner_id as i64)
        .bind(self.id)
        .fetch_one(pool)
        .await?;
        Ok(ws)
    }

    pub async fn fetch_all_users(id: u64, pool: &sqlx::PgPool) -> Result<Vec<ChatUser>, AppError> {
        let ws = sqlx::query_as(
            "select id, fullname, email from users where ws_id = $1 order by id asc",
        )
        .bind(id as i64)
        .fetch_all(pool)
        .await?;
        Ok(ws)
    }
}

#[cfg(test)]
mod tests {
    use crate::{user::CreateUser, AppConfig, AppState, User};

    use super::*;
    use anyhow::Result;
    #[tokio::test]
    async fn workspace_create_by_user_should_work() -> Result<()> {
        let config = AppConfig::load()?;
        let (tdb, _) = AppState::new_for_test(config).await?;
        let pool = tdb.get_pool().await;
        // 创建一个user,它将会插入ws_id为0, user_id为1
        let input = CreateUser::new("test", "email", "fullname", "password");
        let user = User::create(&input, &pool).await.unwrap();
        assert_eq!(user.ws_id, 1);
        assert_eq!(user.id, 1);

        Ok(())
    }

    #[tokio::test]
    async fn workspace_create_should_work() -> Result<()> {
        let config = AppConfig::load()?;
        let (tdb, _) = AppState::new_for_test(config).await?;
        let pool = tdb.get_pool().await;
        // 创建一个workspace, 默认owner_id为0
        let ws = WorkSpace::create("test", 0, &pool).await.unwrap();
        assert_eq!(ws.name, "test");
        assert_eq!(ws.owner_id, 0);

        let input = CreateUser::new("test", "email", "fullname", "password");
        let user = User::create(&input, &pool).await.unwrap();

        // 更新workspace的owner_id
        let ws = ws.update_owner(user.id as u64, &pool).await.unwrap();
        assert_eq!(ws.owner_id, user.id);
        Ok(())
    }

    #[tokio::test]
    async fn fetch_all_users_should_work() -> Result<()> {
        let config = AppConfig::load()?;
        let (tdb, _) = AppState::new_for_test(config).await?;
        let pool = tdb.get_pool().await;

        let input1 = CreateUser::new("test", "email1@acme.come", "fullname1", "password1");
        let user1 = User::create(&input1, &pool).await?;
        let input2 = CreateUser::new("test", "email2@acme.come", "fullname2", "password2");
        let user2 = User::create(&input2, &pool).await?;

        let res = WorkSpace::fetch_all_users(user1.ws_id as u64, &pool).await?;
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].id, user1.id);
        assert_eq!(res[1].id, user2.id);
        Ok(())
    }
}
