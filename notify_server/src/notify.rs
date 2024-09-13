use std::{collections::HashSet, sync::Arc};

use crate::AppState;
use chat_core::{Chat, Message};
use jwt_simple::reexports::serde_json;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgListener;
use tokio_stream::StreamExt;
use tracing::{info, warn};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AppEvent {
    NewChat(Chat),
    AddToChat(Chat),
    RemoveFromChat(Chat),
    NewMessage(Message),
}

struct Notification {
    user_ids: HashSet<u64>,
    event: Arc<AppEvent>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatUpdated {
    op: String,
    old: Option<Chat>,
    new: Option<Chat>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MessageCreated {
    members: Vec<u64>,
    message: Message,
}

pub async fn setup_pg_listener(state: AppState) -> anyhow::Result<()> {
    println!("Connecting to database: {}", state.config.server.db_url);
    let mut listener = PgListener::connect(&state.config.server.db_url).await?;
    listener.listen("chat_updated").await?;
    listener.listen("message_added").await?;

    let mut stream = listener.into_stream();
    tokio::spawn(async move {
        while let Some(Ok(notify)) = stream.next().await {
            info!("Received notification: {:?}", notify);
            let notification = Notification::load(notify.channel(), notify.payload()).unwrap();
            let users = &state.users;
            for user_id in notification.user_ids {
                if let Some(tx) = users.get(&user_id) {
                    info!("Sending notification to user {}", user_id);

                    if let Err(e) = tx.send(notification.event.clone()) {
                        warn!("Failed to send notification to user {}: {:?}", user_id, e);
                    }
                }
            }
        }
        Ok::<_, anyhow::Error>(())
    });
    Ok(())
}

impl Notification {
    fn load(r#type: &str, payload: &str) -> anyhow::Result<Self> {
        match r#type {
            "chat_updated" => {
                let payload = serde_json::from_str::<ChatUpdated>(payload)?;
                info!("Chat updated: {:?}", payload);
                let user_ids = get_affected_user_ids(payload.old.as_ref(), payload.new.as_ref());
                let event = match payload.op.to_lowercase().as_str() {
                    "insert" => AppEvent::NewChat(payload.new.expect("new should exist")),
                    "update" => AppEvent::AddToChat(payload.new.expect("new should exist")),
                    "delete" => AppEvent::RemoveFromChat(payload.old.expect("old should exist")),
                    _ => return Err(anyhow::anyhow!("Invalid operation")),
                };
                Ok(Self {
                    user_ids,
                    event: Arc::new(event),
                })
            }
            "message_added" => {
                let payload = serde_json::from_str::<MessageCreated>(payload)?;
                info!("Message created: {:?}", payload);
                // let user_ids = payload.members.iter().map(|id| *id).collect();
                let user_ids = payload.members.iter().copied().collect();
                let event = AppEvent::NewMessage(payload.message);
                Ok(Self {
                    user_ids,
                    event: Arc::new(event),
                })
            }
            _ => Err(anyhow::anyhow!("Invalid notification type")),
        }
    }
}

fn get_affected_user_ids(old: Option<&Chat>, new: Option<&Chat>) -> HashSet<u64> {
    match (old, new) {
        (Some(old), Some(new)) => {
            let old_users: HashSet<u64> = old.members.iter().map(|id| *id as u64).collect();
            let new_users: HashSet<u64> = new.members.iter().map(|id| *id as u64).collect();
            if old_users != new_users {
                old_users.union(&new_users).copied().collect()
            } else {
                HashSet::new()
            }
        }
        (Some(old), None) => old.members.iter().map(|id| *id as u64).collect(),
        (None, Some(new)) => new.members.iter().map(|id| *id as u64).collect(),
        (None, None) => HashSet::new(),
    }
}
