use crate::Result;
use std::env;

#[derive(Clone)]
pub struct Environment {
    pub discord_bot_token: String,
}

impl Environment {
    pub fn load() -> Result<Self> {
        dotenvy::dotenv()?;
        Ok(Self {
            discord_bot_token: env::var("DISCORD_BOT_TOKEN")
                .map_err(|_| "Missing DISCORD_BOT_TOKEN in env values")?,
        })
    }
}
