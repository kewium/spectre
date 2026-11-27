use twilight_model::gateway::payload::incoming::Ready;

use crate::{BotState, Result};

pub async fn on_ready(_state: BotState, ready: Ready) -> Result<()> {
    match ready.shard {
        Some(shard) => println!(
            "Shard {}/{} is ready for {}#{}",
            shard.number(),
            shard.total(),
            ready.user.name,
            ready.user.discriminator()
        ),
        None => println!(
            "{}#{} is ready",
            ready.user.name,
            ready.user.discriminator()
        ),
    }
    Ok(())
}
