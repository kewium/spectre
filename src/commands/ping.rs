use twilight_model::{
    application::interaction::InteractionContextType, oauth::ApplicationIntegrationType,
};

use crate::commands::{ChatInputCommand, CommandItem, LazyCommandItem};
use std::sync::LazyLock;

pub static COMMAND: LazyCommandItem = LazyLock::new(|| CommandItem {
    internal_command_id: "ping",
    chat_input: Some(ChatInputCommand {
        name: "ping",
        description: "ping",
        integration_types: &[
            ApplicationIntegrationType::GuildInstall,
            ApplicationIntegrationType::UserInstall,
        ],
        contexts: &[InteractionContextType::Guild, InteractionContextType::BotDm],
        execute: |context| {
            Box::pin(async move {
                context.respond_with_text("pong!").await?;
                Ok(())
            })
        },
        ..Default::default()
    }),
    ..Default::default()
});
