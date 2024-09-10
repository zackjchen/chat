use super::TokenVerify;
use axum::{
    extract::{FromRequestParts, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use tracing::warn;

pub async fn verify_token<T>(State(state): State<T>, req: Request, next: Next) -> Response
where
    T: TokenVerify + Clone + Send + Sync + 'static,
{
    let (mut parts, body) = req.into_parts();
    let auth_header =
        TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await;
    let req = match auth_header {
        Ok(TypedHeader(Authorization(bearer))) => {
            let token = bearer.token();
            // Use the token value here
            match state.verify(token) {
                Ok(user) => {
                    let mut req = Request::from_parts(parts, body);
                    req.extensions_mut().insert(user);
                    req
                }
                Err(e) => {
                    let msg = format!("Verify token failed: {:?}", e);
                    warn!(msg);
                    return (StatusCode::FORBIDDEN, msg).into_response();
                }
            }
        }
        Err(e) => {
            let msg = format!("failed to get authorization header: {}", e);
            warn!(msg);
            return (StatusCode::UNAUTHORIZED, msg).into_response();
        }
    };
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::{
        utils::jwt::{DecodingKey, EncodingKey},
        User,
    };
    use anyhow::Result;
    use axum::{
        body::Body, extract::Request, middleware::from_fn_with_state, routing::get, Router,
    };
    use tower::ServiceExt;

    #[derive(Clone)]
    struct AppState(Arc<AppStateInner>);
    struct AppStateInner {
        ek: EncodingKey,
        dk: DecodingKey,
    }
    impl TokenVerify for AppState {
        type Error = ();
        fn verify(&self, token: &str) -> Result<User, ()> {
            let user = self.0.dk.verify(token);
            user.map_err(|_| ())
        }
    }

    async fn handler(_req: Request) -> impl IntoResponse {
        (StatusCode::OK, "Ok")
    }
    #[tokio::test]
    async fn test_verify_token_middleware_should_work() -> Result<()> {
        let decoding_pem = include_str!("../../../chat_server/fixtures/decoding_key.pem");
        let encoding_pem = include_str!("../../../chat_server/fixtures/encoding_key.pem");
        let state = AppState(Arc::new(AppStateInner {
            ek: EncodingKey::load(encoding_pem)?,
            dk: DecodingKey::load(decoding_pem)?,
        }));

        let user = User::new(1, "zack.j.chen@hkjc.org.hk", "Zack", "Jiajia520,");
        let token = state.0.ek.sign(user)?;

        let app = Router::new()
            .route("/test", get(handler))
            .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
            .with_state(state);

        // good token
        let req = Request::builder()
            .uri("/test")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())?;

        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::OK);

        // no token
        let req2 = Request::builder().uri("/test").body(Body::empty())?;
        let res = app.clone().oneshot(req2).await?;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        // bad token
        let req2 = Request::builder()
            .uri("/test")
            .header("Authorization", "Bearer bad_token")
            .body(Body::empty())?;
        let res = app.clone().oneshot(req2).await?;
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
        Ok(())
    }
}
