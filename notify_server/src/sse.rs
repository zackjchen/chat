use std::{convert::Infallible, time::Duration};

use crate::AppEvent;
use axum::{
    debug_handler,
    extract::State,
    response::{sse::Event, Sse},
    Extension,
};

use chat_core::User;
use futures::stream::Stream;
use jwt_simple::reexports::serde_json;
use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tracing::debug;

use crate::AppState;

const CHANNEL_CAPACITY: usize = 256;
#[debug_handler]
pub(crate) async fn sse_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let user_id = user.id as u64;
    // let user_id = 2;
    let users = &state.users;
    let rx = if let Some(tx) = users.get(&user_id) {
        tx.subscribe()
    } else {
        let (tx, rx) = broadcast::channel(CHANNEL_CAPACITY);
        users.insert(user_id, tx);
        rx
    };

    let broadcast_stream = BroadcastStream::new(rx).filter_map(|v| v.ok()).map(|v| {
        let name = match &v.as_ref() {
            AppEvent::NewChat(_) => "NewChat",
            AppEvent::AddToChat(_) => "AddToChat",
            AppEvent::RemoveFromChat(_) => "RemoveFromChat",
            AppEvent::NewMessage(_) => "NewMessage",
        };
        let v = serde_json::to_string(&v).expect("failed to serialize event");
        debug!("sending event {} :{:?}", name, v);

        Ok(Event::default().data(v).event(name))
    });

    Sse::new(broadcast_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
