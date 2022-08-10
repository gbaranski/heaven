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
use store::Store;
use std::env;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let database = Database::new("heaven.db");
    let store = Store::new();
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let whitelist_channel_id = env::var("WHITELIST_CHANNEL_ID").expect("Expected a whitelist channel ID in the environment");
    let whitelist_channel_id = ChannelId::from_str(&whitelist_channel_id).unwrap();
    let http = Http::new(&token);

    tokio::spawn({
        let database = database.clone();
        let store = store.clone();
        async move {
            let address = SocketAddr::from(([0, 0, 0, 0], 3000));
            server::run(address, database, store, http).await;
        }
    });
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

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        tracing::error!("Client error: {:?}", why);
    }
}
