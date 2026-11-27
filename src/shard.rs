use twilight_gateway::{EventTypeFlags, Shard, StreamExt};

use crate::{BotState, Result, events::handle_event};

pub async fn run(mut shard: Shard, state: BotState) -> Result<()> {
    while let Some(item) = shard.next_event(EventTypeFlags::all()).await {
        let Ok(event) = item else { continue };
        tokio::spawn(handle_event(state.clone(), event));
    }
    Ok(())
}
