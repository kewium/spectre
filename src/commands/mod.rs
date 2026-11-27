mod ping;

use std::{collections::HashMap, sync::LazyLock};
use twilight_model::{
    application::{
        command::{Command, CommandOption, CommandType},
        interaction::{InteractionContextType, InteractionData, InteractionType},
    },
    gateway::payload::incoming::InteractionCreate,
    guild::Permissions,
    http::interaction::InteractionResponse,
    oauth::ApplicationIntegrationType,
};
use twilight_util::builder::{
    command::CommandBuilder, interaction_response::ChannelMessageBuilder,
};

use crate::{BotState, Result};

#[allow(dead_code)]
pub type PinBoxFuture<'alive, T> = std::pin::Pin<Box<dyn Future<Output = T> + Send + 'alive>>;
#[allow(dead_code)]
pub type ExecuteResult<'alive> = PinBoxFuture<'alive, Result<()>>;

pub static ALL: LazyCommandItems =
    LazyLock::new(|| HashMap::from([(ping::COMMAND.internal_command_id, &*ping::COMMAND)]));

pub type LazyCommandItem = LazyLock<CommandItem>;
pub type LazyCommandItems = LazyLock<HashMap<&'static str, &'static CommandItem>>;
pub type CommandExecuteFunction = fn(ApplicationCommandContext) -> ExecuteResult<'static>;

#[allow(dead_code)]
pub struct CommandItem {
    #[allow(dead_code)]
    pub internal_command_id: &'static str,
    #[allow(dead_code)]
    pub features: Option<&'static [&'static str]>,
    #[allow(dead_code)]
    pub chat_input: Option<ChatInputCommand>,
    #[allow(dead_code)]
    pub user: Option<UserCommand>,
    #[allow(dead_code)]
    pub message: Option<MessageCommand>,
}

impl Default for CommandItem {
    fn default() -> Self {
        Self {
            internal_command_id: "",
            features: None,
            chat_input: None,
            user: None,
            message: None,
        }
    }
}

impl CommandItem {
    pub fn execute_for(&self, kind: CommandType) -> Option<CommandExecuteFunction> {
        match kind {
            CommandType::ChatInput => self.chat_input.as_ref().map(|c| c.execute),
            CommandType::User => self.user.as_ref().map(|c| c.execute),
            CommandType::Message => self.message.as_ref().map(|c| c.execute),
            _ => None,
        }
    }
}

fn not_implemented(_: ApplicationCommandContext) -> ExecuteResult<'static> {
    Box::pin(async move { Err("Missing command implementation".into()) })
}

pub struct ChatInputCommand {
    pub name: &'static str,
    pub description: &'static str,
    pub options: &'static [CommandOption],
    pub default_member_permissions: Permissions,
    pub nsfw: bool,
    pub integration_types: &'static [ApplicationIntegrationType],
    pub contexts: &'static [InteractionContextType],
    pub execute: CommandExecuteFunction,
}

impl Default for ChatInputCommand {
    fn default() -> Self {
        Self {
            name: "",
            description: "",
            options: &[],
            default_member_permissions: Permissions::empty(),
            nsfw: false,
            integration_types: &[ApplicationIntegrationType::GuildInstall],
            contexts: &[InteractionContextType::Guild],
            execute: not_implemented,
        }
    }
}

impl ChatInputCommand {
    pub fn build(&self) -> Command {
        let mut builder = CommandBuilder::new(self.name, self.description, CommandType::ChatInput);
        for option in self.options.iter().cloned() {
            builder = builder.option(option);
        }
        builder = builder.default_member_permissions(self.default_member_permissions);
        builder = builder.nsfw(self.nsfw);
        builder = builder.integration_types(self.integration_types.iter().copied());
        builder = builder.contexts(self.contexts.iter().copied());
        builder.build()
    }
}

pub struct UserCommand {
    pub name: &'static str,
    pub default_member_permissions: Permissions,
    pub nsfw: bool,
    pub integration_types: &'static [ApplicationIntegrationType],
    pub contexts: &'static [InteractionContextType],
    pub execute: CommandExecuteFunction,
}

impl Default for UserCommand {
    fn default() -> Self {
        Self {
            name: "",
            default_member_permissions: Permissions::empty(),
            nsfw: false,
            integration_types: &[ApplicationIntegrationType::GuildInstall],
            contexts: &[InteractionContextType::Guild],
            execute: not_implemented,
        }
    }
}

impl UserCommand {
    pub fn build(&self) -> Command {
        let mut builder = CommandBuilder::new(self.name, "", CommandType::User);
        builder = builder.default_member_permissions(self.default_member_permissions);
        builder = builder.nsfw(self.nsfw);
        builder = builder.integration_types(self.integration_types.iter().copied());
        builder = builder.contexts(self.contexts.iter().copied());
        builder.build()
    }
}

pub struct MessageCommand {
    pub name: &'static str,
    pub default_member_permissions: Permissions,
    pub nsfw: bool,
    pub integration_types: &'static [ApplicationIntegrationType],
    pub contexts: &'static [InteractionContextType],
    pub execute: CommandExecuteFunction,
}

impl Default for MessageCommand {
    fn default() -> Self {
        Self {
            name: "",
            default_member_permissions: Permissions::empty(),
            nsfw: false,
            integration_types: &[ApplicationIntegrationType::GuildInstall],
            contexts: &[InteractionContextType::Guild],
            execute: not_implemented,
        }
    }
}

impl MessageCommand {
    pub fn build(&self) -> Command {
        let mut builder = CommandBuilder::new(self.name, "", CommandType::Message);
        builder = builder.default_member_permissions(self.default_member_permissions);
        builder = builder.nsfw(self.nsfw);
        builder = builder.integration_types(self.integration_types.iter().copied());
        builder = builder.contexts(self.contexts.iter().copied());
        builder.build()
    }
}

pub struct ApplicationCommandContext {
    pub state: BotState,
    pub interaction: InteractionCreate,
}

impl ApplicationCommandContext {
    pub async fn respond(&self, response: &InteractionResponse) -> Result<()> {
        self.state
            .http
            .interaction(*self.state.application_id)
            .create_response(self.interaction.id, &self.interaction.token, response)
            .await?;
        Ok(())
    }

    pub async fn respond_with_text(&self, text: impl Into<String>) -> Result<()> {
        self.respond(&ChannelMessageBuilder::new().content(text).build())
            .await?;
        Ok(())
    }
}

pub async fn handle_application_command(
    state: BotState,
    interaction: Box<InteractionCreate>,
) -> Result<()> {
    match &interaction.kind {
        InteractionType::ApplicationCommand => {
            let Some(InteractionData::ApplicationCommand(command)) = &interaction.data else {
                return Ok(());
            };

            let key = match &command.kind {
                CommandType::ChatInput => format!("chatinput-{}", command.name),
                CommandType::User => format!("user-{}", command.name),
                CommandType::Message => format!("message-{}", command.name),
                _ => return Ok(()),
            };

            let Some(internal_command_id) = state.commands.get(&key) else {
                return Ok(());
            };
            let Some(item) = ALL.get(internal_command_id.as_str()) else {
                return Ok(());
            };
            let Some(execute) = item.execute_for(command.kind) else {
                return Ok(());
            };

            let context = ApplicationCommandContext {
                state: state,
                interaction: *interaction.clone(),
            };

            execute(context).await?;
        }
        _ => return Ok(()),
    };

    Ok(())
}
