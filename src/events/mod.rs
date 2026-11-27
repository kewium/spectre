mod interaction_create;
mod ready;

use twilight_gateway::Event;

use crate::{BotState, Result};

pub async fn handle_event(state: BotState, event: Event) -> Result<()> {
    match event {
        Event::InteractionCreate(interaction_create) => {
            interaction_create::on_interaction_create(state, interaction_create).await?;
        }
        Event::Ready(ready) => ready::on_ready(state, ready).await?,
        _ => {}
    }

    Ok(())
}
