use serde::Deserialize;
use serenity::model::prelude::{ChannelId, GuildId};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub port: u16,
    pub data_path: PathBuf,
    pub discord_token: String,
    pub whitelist_channel_id: ChannelId,
    pub guild_id: GuildId,
}

impl Configuration {
    pub fn get() -> Self {
        dotenv::dotenv().ok();
        envy::from_env().unwrap()
    }
}
