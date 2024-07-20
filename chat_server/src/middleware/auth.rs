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

use crate::AppState;

pub(crate) async fn verify_token(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let (mut parts, body) = req.into_parts();
    let auth_header =
        TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await;
    let req = match auth_header {
        Ok(TypedHeader(Authorization(bearer))) => {
            let token = bearer.token();
            // Use the token value here
            match state.dk.verify(token) {
                Ok(user) => {
                    let mut req = Request::from_parts(parts, body);
                    req.extensions_mut().insert(user);
                    req
                }
                Err(e) => {
                    let msg = format!("Verify token failed: {}", e);
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
