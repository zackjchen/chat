use axum::{
    extract::{FromRequestParts, Path, Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{error::AppError, AppState, User};

pub(crate) async fn verify_chat(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let (mut parts, body) = req.into_parts();
    let Path(chat_id) = Path::<u64>::from_request_parts(&mut parts, &state)
        .await
        .unwrap();
    let user = parts.extensions.get::<User>().unwrap();
    println!("User: {:?}", user);
    if !state
        .is_chat_member(chat_id as i64, user.id as u64)
        .await
        .unwrap()
    {
        let err = AppError::CreateMessageError(format!(
            "user {} is not a member of chat {}",
            user.id, chat_id
        ));
        return err.into_response();
    }
    let req = Request::from_parts(parts, body);
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use crate::{middleware::auth::verify_token, User};

    use super::*;
    use anyhow::Result;
    use axum::{
        body::Body, extract::Request, http::StatusCode, middleware::from_fn_with_state,
        routing::get, Router,
    };
    use tower::ServiceExt;

    async fn handler(_req: Request) -> impl IntoResponse {
        (StatusCode::OK, "Ok")
    }
    #[tokio::test]
    async fn test_verify_token_middleware_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let user = User::new(1, "test.chat@test.com", "test_chat", "123asd");
        let token = state.ek.sign(user.clone())?;
        let app = Router::new()
            .route("/test/:id", get(handler))
            .route("/test/:id/messages", get(handler))
            .layer(from_fn_with_state(state.clone(), verify_chat))
            .layer(from_fn_with_state(state.clone(), verify_token))
            .with_state(state.clone());

        // new user should not be a member of chat
        let req = Request::builder()
            .uri("/test/2")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())?;

        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);

        // no token
        let user = state.find_user_by_email("zack@email.com").await?.unwrap();
        let token = state.ek.sign(user)?;
        let req2 = Request::builder()
            .uri("/test/2")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())?;
        let res = app.clone().oneshot(req2).await?;
        assert_eq!(res.status(), StatusCode::OK);

        // chat id not exist
        let req2 = Request::builder()
            .uri("/test/100")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())?;
        let res = app.clone().oneshot(req2).await?;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        Ok(())
    }
}
