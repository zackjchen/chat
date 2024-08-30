use crate::{error::AppError, AppState, ChatUser};

use super::WorkSpace;
impl AppState {
    pub async fn create_workspace(&self, name: &str, user_id: u64) -> Result<WorkSpace, AppError> {
        let ws =
            sqlx::query_as("insert into workspaces (name, owner_id) values ($1, $2) returning *")
                .bind(name)
                .bind(user_id as i64)
                .fetch_one(&self.pool)
                .await?;

        Ok::<WorkSpace, AppError>(ws)
    }

    pub async fn find_workspace_by_name(
        &self,
        name: impl Into<String>,
    ) -> Result<Option<WorkSpace>, AppError> {
        let ws = sqlx::query_as("select * from workspaces where name = $1")
            .bind(name.into())
            .fetch_optional(&self.pool)
            .await?;
        Ok(ws)
    }

    #[allow(dead_code)]
    pub async fn find_workspace_by_email(
        &self,
        email: impl Into<String>,
    ) -> Result<Option<WorkSpace>, AppError> {
        let ws = sqlx::query_as("select * from workspaces where email = &1")
            .bind(email.into())
            .fetch_optional(&self.pool)
            .await?;
        Ok(ws)
    }

    #[allow(dead_code)]
    pub async fn update_workspace_owner(
        &self,
        wp: &WorkSpace,
        owner_id: u64,
    ) -> Result<WorkSpace, AppError> {
        let ws = sqlx::query_as(
            r#"
                update workspaces set owner_id = $1
                where id = $2
                and id = (select ws_id from users where id = $1)
                returning *
            "#,
        )
        .bind(owner_id as i64)
        .bind(wp.id)
        .fetch_one(&self.pool)
        .await?;
        Ok(ws)
    }

    /// id: ws_id
    pub async fn fetch_workspace_all_users(&self, id: u64) -> Result<Vec<ChatUser>, AppError> {
        let ws = sqlx::query_as(
            "select id, fullname, email from users where ws_id = $1 order by id asc",
        )
        .bind(id as i64)
        .fetch_all(&self.pool)
        .await?;
        Ok(ws)
    }
}

#[cfg(test)]
mod tests {
    use crate::{user::CreateUser, AppState};
    use anyhow::Result;
    #[tokio::test]
    async fn workspace_create_by_user_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        // 创建一个user,它将会插入ws_id为1, user_id为1
        let input = CreateUser::new("test1", "email", "fullname", "password");
        let user = state.create_user(&input).await.unwrap();
        assert_eq!(user.ws_id, 1);
        assert_eq!(user.id, 5);

        Ok(())
    }

    #[tokio::test]
    async fn workspace_create_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        // 创建一个workspace, 默认owner_id为0
        let ws = state.create_workspace("test", 0).await.unwrap();
        assert_eq!(ws.name, "test");
        assert_eq!(ws.owner_id, 0);

        let input = CreateUser::new("test", "email", "fullname", "password");
        let user = state.create_user(&input).await.unwrap();

        // 更新workspace的owner_id
        let ws = state
            .update_workspace_owner(&ws, user.id as u64)
            .await
            .unwrap();
        assert_eq!(ws.owner_id, user.id);
        Ok(())
    }

    #[tokio::test]
    async fn fetch_all_users_should_work() -> Result<()> {
        // let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test().await?;
        // let pool = tdb.get_pool().await;

        // let input1 = CreateUser::new("test", "email1@acme.come", "fullname1", "password1");
        // let user1 = User::create(&input1, &pool).await?;
        // let input2 = CreateUser::new("test", "email2@acme.come", "fullname2", "password2");
        // let user2 = User::create(&input2, &pool).await?;

        // let res = WorkSpace::fetch_all_users(user1.ws_id as u64, &pool).await?;
        // assert_eq!(res.len(), 2);
        // assert_eq!(res[0].id, user1.id);
        // assert_eq!(res[1].id, user2.id);
        let users = state.fetch_workspace_all_users(1).await?;
        assert_eq!(users.len(), 4);
        Ok(())
    }
}
