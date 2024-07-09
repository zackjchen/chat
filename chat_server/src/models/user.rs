#![allow(unused)]
use std::mem;

use crate::{AppError, User};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

impl User {
    pub async fn find_by_email(
        email: String,
        pool: &sqlx::PgPool,
    ) -> Result<Option<Self>, AppError> {
        // let rec = sqlx::query_as!(User, r#"select id,fullname,email, created_at from users where email = $1"#, email)
        // .fetch_optional(pool).await?;

        let rec =
            sqlx::query_as(r#"select id,fullname,email, created_at from users where email = $1"#)
                .bind(email)
                .fetch_optional(pool)
                .await?;

        Ok(rec)
    }

    pub async fn create(
        email: &str,
        fullname: &str,
        password: &str,
        pool: &sqlx::PgPool,
    ) -> Result<Self, AppError> {
        let user = sqlx::query_as(
            r#"insert into users (fullname, email, password_hash) values ($1, $2, $3) returning id,fullname,email, created_at"#
        )
        .bind(fullname)
        .bind(email)
        .bind(password)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn verify(
        email: &str,
        password: &str,
        pool: &sqlx::PgPool,
    ) -> Result<bool, AppError> {
        let user: Option<User> = sqlx::query_as(
            r#"select id,fullname,email, password_hash,created_at from users where email = $1"#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        match user {
            Some(mut user) => {
                let password_hash = mem::take(&mut user.password_hash).unwrap_or_default();
                let is_valid = verify_password(password, &password_hash)?;
                Ok(is_valid)
            }
            None => Ok(false),
        }
    }
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

    use super::*;
    #[tokio::test]
    async fn create_user_should_work() -> Result<()> {
        let tdb = TestPg::new(
            "postgresql://zackjchen:postgres@localhost:5432".into(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;

        let email = "zackjchen@hkjc.org.hk";
        let fullname = "zackjchen";
        let password = "hunter42";
        let user = User::create(email, fullname, password, &pool).await?;

        assert_eq!(user.email, email);
        assert_eq!(user.fullname, fullname);

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
