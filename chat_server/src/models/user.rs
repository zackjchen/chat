#![allow(unused)]
use std::mem;

use crate::{config, AppError, ChatUser, User, WorkSpace};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};
use sqlx::pool;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub fullname: String,
    pub password: String,
    pub workspace: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
}

#[cfg(test)]
impl SigninUser {
    pub fn new(email: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            password: password.into(),
        }
    }
}
#[cfg(test)]
impl CreateUser {
    pub fn new(
        ws_name: impl Into<String>,
        email: impl Into<String>,
        fullname: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            email: email.into(),
            fullname: fullname.into(),
            password: password.into(),
            workspace: ws_name.into(),
        }
    }
}

impl ChatUser {
    pub async fn fetch_all(ws_id: u64, pool: &sqlx::PgPool) -> Result<Vec<Self>, AppError> {
        let recs = sqlx::query_as(r#"select id, fullname, email from users where ws_id = $1"#)
            .bind(ws_id as i64)
            .fetch_all(pool)
            .await?;
        Ok(recs)
    }

    pub async fn fetch_by_ids(ids: &[i64], pool: &sqlx::PgPool) -> Result<Vec<Self>, AppError> {
        let recs = sqlx::query_as(r#"select id, fullname, email from users where id = any($1)"#)
            .bind(ids)
            .fetch_all(pool)
            .await?;
        Ok(recs)
    }
}

impl User {
    pub async fn find_by_email(email: &str, pool: &sqlx::PgPool) -> Result<Option<Self>, AppError> {
        // let rec = sqlx::query_as!(User, r#"select id,fullname,email, created_at from users where email = $1"#, email)
        // .fetch_optional(pool).await?;

        let rec = sqlx::query_as(
            r#"select id, ws_id, fullname,email, created_at from users where email = $1"#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        Ok(rec)
    }

    // TODO: use trancaction for workspace creation
    pub async fn create(input: &CreateUser, pool: &sqlx::PgPool) -> Result<Self, AppError> {
        // 插入用户时，检查email是否已经存在，如果存在表示已经注册，则返回错误
        let user = Self::find_by_email(&input.email, pool).await?;
        if let Some(user) = user {
            return Err(AppError::EmailAlreadyExists(user.email));
        }
        let password_hash = hash_password(&input.password)?;

        // 插入用户时，需要先判断workspace是否存在，如果不存在则创建
        let ws = match WorkSpace::find_by_name(&input.workspace, pool).await? {
            Some(ws) => ws,
            None => WorkSpace::create(&input.workspace, 0, pool).await?,
        };

        // 这里需要通过workspace的name去找id然后插入
        let user: User = sqlx::query_as(
            r#"insert into users (fullname, email, password_hash, ws_id) values ($1, $2, $3, $4) returning id, ws_id, fullname, email, created_at"#
        )
        .bind(&input.fullname)
        .bind(&input.email)
        .bind(password_hash)
        .bind(ws.id)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn verify(input: SigninUser, pool: &sqlx::PgPool) -> Result<Option<User>, AppError> {
        let user: Option<User> = sqlx::query_as(
            r#"select id, ws_id, fullname, email, password_hash,created_at from users where email = $1"#,
        )
        .bind(input.email)
        .fetch_optional(pool)
        .await?;

        match user {
            Some(mut user) => {
                let password_hash = mem::take(&mut user.password_hash).unwrap_or_default();
                let is_valid = verify_password(&input.password, &password_hash)?;
                if is_valid {
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    // pub async fn add_to_workspace(&self, ws_id: i64, pool: &sqlx::PgPool) -> Result<User, AppError> {
    //     let user = sqlx::query_as("update users set ws_id = $1 where id = $2 and ws_id = 0 returning *")
    //         .bind(ws_id)
    //         .bind(self.id)
    //         .fetch_one(pool)
    //         .await?;
    //     Ok(user)

    // }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(hash)
}

fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(password_hash)?;
    let is_valid = argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();
    Ok(is_valid)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use anyhow::Result;
    use sqlx_db_tester::TestPg;

    use crate::{test_utils::get_test_pool, AppConfig, AppState};

    use super::*;
    #[tokio::test]
    async fn create_user_should_work() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;

        let email = "zackjchen@hkjc.org.hk";
        let fullname = "zackjchen";
        let password = "hunter42";
        let input = CreateUser {
            email: email.into(),
            fullname: fullname.into(),
            password: password.into(),
            workspace: "hkjc".into(),
        };
        let user = User::create(&input, &pool).await?;

        assert_eq!(user.email, email);
        assert_eq!(user.fullname, fullname);

        let input = SigninUser {
            email: "zackjchen@hkjc.org.hk".into(),
            password: "hunter42".into(),
        };

        let res = User::verify(input, &pool).await?;
        if let Some(user) = res {
            assert_eq!(user.email, email);
            assert_eq!(user.fullname, fullname);
        } else {
            panic!("User not found");
        }

        Ok(())
    }

    #[test]
    fn password_hash_should_work() {
        let password = "password";
        let hash = hash_password(password).unwrap();
        assert_ne!(password, hash);
        println!("Hash: {}", hash);

        let valiad1 = verify_password(password, &hash).is_ok();
        let valiad2 = verify_password("password1", &hash).is_ok();
        assert!(valiad1);
        assert!(valiad2);
    }
}
