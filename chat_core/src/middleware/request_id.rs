use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use tracing::warn;

use super::REQUEST_ID_HEADER;

/// 一个中间件，通常接受三个参数， 然后返回一个Response
/// 1. 状态 state
/// 2. 请求 request
/// 3. 下一个中间件 next
pub async fn set_request_id(mut req: Request, next: Next) -> Response {
    let id = match req.headers().get(REQUEST_ID_HEADER) {
        Some(id) => Some(id.clone()),
        None => {
            let request_id = uuid::Uuid::now_v7().to_string();
            match request_id.parse::<HeaderValue>() {
                Ok(v) => {
                    req.headers_mut().insert(REQUEST_ID_HEADER, v.clone());
                    Some(v)
                }
                Err(e) => {
                    warn!("failed to generate request id: {}", e);
                    None
                }
            }
        }
    };

    let mut res = next.run(req).await;
    if let Some(id) = id {
        res.headers_mut().insert(REQUEST_ID_HEADER, id);
    }
    res
}
