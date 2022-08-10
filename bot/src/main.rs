mod bot;
mod database;
mod models;
mod server;
mod store;

use bot::Bot;
use database::Database;

use dotenv::dotenv;
use serenity::client::bridge::gateway::ShardManager;
use serenity::http::Http;
use serenity::model::prelude::ChannelId;
use serenity::prelude::*;
use std::{env, time::Duration};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use store::Store;
use tokio_graceful_shutdown::{Toplevel, SubsystemHandle};

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

async fn subsystem(subsys: SubsystemHandle) -> Result<(), anyhow::Error> {
    dotenv().ok();
    let port = env::var("PORT")
        .map(|port| port.parse().unwrap())
        .unwrap_or(8080);
    let data_path = std::path::PathBuf::from_str(
        &env::var("DATA_PATH").expect("Expected a data path in the environment"),
    )
    .unwrap();
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let whitelist_channel_id = env::var("WHITELIST_CHANNEL_ID")
        .expect("Expected a whitelist channel ID in the environment");

    let whitelist_channel_id = ChannelId::from_str(&whitelist_channel_id).unwrap();
    let http = Http::new(&token);
    let database = Database::new(data_path.join("database.sqlite3"));
    let store = Store::new();

    let address = SocketAddr::from(([0, 0, 0, 0], port));
    let server_handle = server::run(address, database.clone(), store.clone(), http);
    let bot = Bot::new(database, store, whitelist_channel_id);
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(bot)
        .await
        .expect("Error creating client");
    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }
    let shard_manager = client.shard_manager.clone();

    tokio::select! {
        result = client.start() => {
            if let Err(why) = result {
                tracing::error!("Client error: {:?}", why);
            }
        }
        _ = subsys.on_shutdown_requested() => {
            tracing::info!("shutting down");
            server_handle.shutdown();
            shard_manager.lock().await.shutdown_all().await;
        }
    };

    Ok(())
}

#[tokio::main]
async fn main() ->  Result<(), anyhow::Error>  {
    tracing_subscriber::fmt::init();
    Toplevel::new()
        .start("bot", subsystem)
        .catch_signals()
        .handle_shutdown_requests(Duration::from_secs(10))
        .await?;

    Ok(())
}
