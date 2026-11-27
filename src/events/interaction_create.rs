use twilight_model::{
    application::interaction::InteractionType, gateway::payload::incoming::InteractionCreate,
};

use crate::{BotState, Result, commands::handle_application_command};

pub async fn on_interaction_create(
    state: BotState,
    interaction_create: Box<InteractionCreate>,
) -> Result<()> {
    match interaction_create.kind {
        InteractionType::ApplicationCommand => {
            handle_application_command(state, interaction_create).await?;
        }
        _ => return Ok(()),
    }
    Ok(())
}
