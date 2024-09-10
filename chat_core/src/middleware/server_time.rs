/// 使用tower::Layer和tower::Service实现一个中间件，用于在响应头中添加服务器处理时间
use axum::{extract::Request, response::Response};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::time::Instant;
use tower::{Layer, Service};
use tracing::warn;

use crate::middleware::REQUEST_ID_HEADER;

use super::SERVER_TIME_HEADER;

#[derive(Clone)]
pub struct ServerTimeLayer;

impl<S> Layer<S> for ServerTimeLayer {
    type Service = ServerTimeMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ServerTimeMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct ServerTimeMiddleware<S> {
    inner: S,
}

impl<S> Service<Request> for ServerTimeMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    // `BoxFuture` is a type alias for `Pin<Box<dyn Future + Send + 'a>>`
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let start = Instant::now();
        let future = self.inner.call(request);
        Box::pin(async move {
            let mut response: Response = future.await?;
            let elapsed = format!("{:?}us", start.elapsed().as_micros());
            match elapsed.parse() {
                Ok(v) => {
                    response.headers_mut().insert(SERVER_TIME_HEADER, v);
                }
                Err(e) => {
                    let request_id = response.headers().get(REQUEST_ID_HEADER);
                    warn!(
                        "parse elapsed time failed: {} for request_id:{:?}",
                        e, request_id
                    );
                }
            }
            Ok(response)
        })
    }
}
