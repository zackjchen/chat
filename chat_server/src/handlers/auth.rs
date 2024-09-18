use crate::{
    error::{AppError, ErrorOutput},
    user::{CreateUser, SigninUser},
    AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthOutput {
    pub token: String,
}

#[utoipa::path(
    post,
    path = "/api/signup",
    request_body = CreateUser,
    responses(
        (status=200, description="User created successfully", body=AuthOutput),
    ),
    tag = "chat"

)]
/// Create a new user in the chat system with email, password workspace and full name.
///
/// - If the email already exists, it will return 409.
/// - Otherwise, it will return 201 with a token.
/// - If the workspace doesn't exist, it will create one.
pub(crate) async fn signup_handler(
    State(state): State<AppState>,
    Json(input): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.create_user(&input).await?;
    let token = state.ek.sign(user)?;
    // let mut header = HeaderMap::new();
    // header.insert("X-Token", token.parse()?);
    // Ok((StatusCode::CREATED, header))
    let body = Json(AuthOutput { token });
    Ok((StatusCode::CREATED, body))
}

/// Sign in a user with email and password.
#[utoipa::path(
    post,
    path = "/api/signin",
    request_body = SigninUser,
    responses(
        (status=200, description="User Signin", body=Chat),
    ),
    tag = "chat"

)]
pub(crate) async fn signin_handler(
    State(state): State<AppState>,
    Json(input): Json<SigninUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.verify_user(input).await?;
    match user {
        Some(user) => {
            let token = state.ek.sign(user)?;
            Ok((StatusCode::OK, Json(AuthOutput { token })).into_response())
        }
        None => Ok((
            StatusCode::FORBIDDEN,
            Json(ErrorOutput::new("Invalid email or password")),
        )
            .into_response()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use http_body_util::BodyExt;
    use jwt_simple::reexports::serde_json;

    #[tokio::test]
    async fn test_signup_handler() -> Result<()> {
        let (_tgp, state) = AppState::new_for_test().await?;
        let input = CreateUser::new("default", "aaa@hkjc.org.hk", "zackjchen", "hunter42");
        let res = signup_handler(State(state), Json(input))
            .await?
            .into_response();
        assert_eq!(res.status(), StatusCode::CREATED);

        let body = res.into_body();
        let bytes = body.collect().await?.to_bytes();
        let auth: AuthOutput = serde_json::from_slice(&bytes)?;
        assert_ne!(auth.token, "");
        Ok(())
    }

    #[tokio::test]
    async fn test_duplicate_signup_handler_should_409() -> Result<()> {
        let (_tgp, state) = AppState::new_for_test().await?;
        let input1 = CreateUser::new("default", "zackjchen@hkjc.org.hk", "zackjchen", "hunter43");
        let input2 = CreateUser::new("default", "zackjchen@hkjc.org.hk", "zackjchen", "hunter43");

        let _res1 = signup_handler(State(state.clone()), Json(input1))
            .await?
            .into_response();
        let res2 = signup_handler(State(state), Json(input2))
            .await
            .into_response();
        assert_eq!(res2.status(), StatusCode::CONFLICT);
        let body = res2.into_body().collect().await?.to_bytes();
        // let error: serde_json::Value = serde_json::from_slice(&body)?;
        // println!("Error: {:?}", error);
        // assert_eq!(error.get("error").unwrap(), "Email already exists: zackjchen@hkjc.org.hk");
        let error: ErrorOutput = serde_json::from_slice(&body)?;
        assert_eq!(
            error,
            ErrorOutput::new("Email already exists: zackjchen@hkjc.org.hk")
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_signin_handler() -> Result<()> {
        let (_tgp, state) = AppState::new_for_test().await?;
        let input = CreateUser::new("default", "zackjchen@hkjc.org.hk", "zackjchen", "hunter43");
        let _res1 = signup_handler(State(state.clone()), Json(input))
            .await?
            .into_response();

        let signinuser = SigninUser::new("zackjchen@hkjc.org.hk", "hunter43");
        let res2 = signin_handler(State(state), Json(signinuser))
            .await?
            .into_response();
        assert_eq!(res2.status(), StatusCode::OK);

        let body = res2.into_body();
        let bytes = body.collect().await?.to_bytes();
        let auth: AuthOutput = serde_json::from_slice(&bytes)?;
        assert_ne!(auth.token, "");

        Ok(())
    }
    #[tokio::test]
    async fn test_signin_with_non_exists_should_403() -> Result<()> {
        let (_tgp, state) = AppState::new_for_test().await?;
        let input = SigninUser::new("zack.j.chen@hkjc.org.hk", "hunter42");
        let res = signin_handler(State(state), Json(input))
            .await
            .into_response();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);

        let body = res.into_body();
        let bytes = body.collect().await?.to_bytes();
        let error: ErrorOutput = serde_json::from_slice(&bytes)?;
        assert_eq!(error, ErrorOutput::new("Invalid email or password"));
        // if let  serde_json::Value::String(e) = serde_json::from_slice(&bytes)?{
        //     assert_eq!(e, "Invalid email or password");
        // }else {
        //     panic!("test_signin_with_non_exists_should_403: Expecting a string");
        // }
        Ok(())
    }
    #[tokio::test]
    async fn create_duplicate_user_should_failed() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateUser::new("default", "abc@hkjc.org.hk", "zackjchen", "hunter43");
        state.create_user(&input).await?;
        let res2 = state.create_user(&input).await;
        match res2 {
            Err(AppError::EmailAlreadyExists(e)) => assert_eq!(e, "abc@hkjc.org.hk"),
            _ => panic!("Expecting EmailAlreadyExists error"),
        }

        Ok(())
    }
}
