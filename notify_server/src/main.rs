use anyhow::Result;
use notify_server::{get_router, setup_pg_listener};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _, Layer as _,
};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let addr = "0.0.0.0:6687";

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Starting server at {}", addr);
    let (app, state) = get_router();
    setup_pg_listener(state).await?;

    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
