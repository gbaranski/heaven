mod bot;
mod configuration;
mod database;
mod models;
mod server;
mod authorizations;

pub use bot::DiscordBot;
use database::Database;

use serenity::client::bridge::gateway::ShardManager;
use serenity::prelude::*;
use miette::{IntoDiagnostic, Result};
use server::Server;
use std::sync::Arc;
use std::time::Duration;
use tokio_graceful_shutdown::Toplevel;

use crate::configuration::Configuration;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let configuration = Arc::new(Configuration::get());
    let database = Database::new(configuration.data_path.join("database.sqlite3"));

    let discord_bot = DiscordBot::new(configuration.clone(), database.clone());
    let server = Server::new(database,  configuration, discord_bot.clone());

    Toplevel::new()
        .start("bot", |subsystem| discord_bot.run(subsystem))
        .start("server", |subsystem| server.run(subsystem))
        .catch_signals()
        .handle_shutdown_requests(Duration::from_millis(1000))
        .await
        .into_diagnostic()
}
