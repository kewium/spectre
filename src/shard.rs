use twilight_gateway::{EventTypeFlags, Shard, StreamExt};

use crate::{BotState, Result, events::handle_event};

pub async fn run(mut shard: Shard, state: BotState) -> Result<()> {
    while let Some(item) = shard.next_event(EventTypeFlags::all()).await {
        let event = match item {
            Ok(event) => event,
            Err(source) => {
                tracing::warn!(?source, "failed to receive event");
                continue;
            }
        };
        let state = state.clone();
        tokio::spawn(async move {
            let kind = event.kind();
            if let Err(error) = handle_event(state, event).await {
                tracing::error!(?kind, %error, "event handler failed");
            }
        });
    }
    tracing::error!("gateway connection closed fatally");
    Err("gateway connection closed fatally".into())
}
