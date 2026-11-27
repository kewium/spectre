mod commands;
mod environment;
mod events;
mod shard;

use std::{collections::HashMap, sync::Arc};

use tracing_subscriber::EnvFilter;
use twilight_gateway::{Intents, Shard, ShardId};
use twilight_http::Client;
use twilight_model::id::{Id, marker::ApplicationMarker};
use twilight_standby::Standby;

#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone)]
pub struct BotState {
    pub application_id: Arc<Id<ApplicationMarker>>,
    pub http: Arc<Client>,
    pub standby: Arc<Standby>,
    pub commands: Arc<HashMap<String, String>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("[RustLS] (Ring) couldnt install");

    let env = environment::Environment::load()?;
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let intents = Intents::GUILD_MESSAGES | Intents::DIRECT_MESSAGES | Intents::MESSAGE_CONTENT;
    let shard = Shard::new(ShardId::ONE, env.discord_bot_token.clone(), intents);
    let http = Arc::new(Client::new(env.discord_bot_token));
    let standby = Arc::new(Standby::new());

    // Get commands
    let mut built_commands = vec![];
    let mut all_commands = HashMap::new();
    for (_, command) in commands::ALL.iter() {
        if let Some(chat_input) = &command.chat_input {
            all_commands.insert(
                format!("chatinput-{}", chat_input.name),
                command.internal_command_id.to_string(),
            );
            built_commands.push(chat_input.build());
        }
        if let Some(user) = &command.user {
            all_commands.insert(
                format!("user-{}", user.name),
                command.internal_command_id.to_string(),
            );
            built_commands.push(user.build());
        }
        if let Some(message) = &command.message {
            all_commands.insert(
                format!("message-{}", message.name),
                command.internal_command_id.to_string(),
            );
            built_commands.push(message.build());
        }
    }

    let application_id = Arc::new(http.current_user_application().await?.model().await?.id);
    let state = BotState {
        application_id: application_id,
        http: http,
        standby: standby,
        commands: Arc::new(all_commands),
    };

    state
        .http
        .interaction(*state.application_id)
        .set_global_commands(&built_commands)
        .await?;

    shard::run(shard, state).await?;
    Ok(())
}
