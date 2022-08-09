use crate::models::{MinecraftType, Angel, AngelID};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{OptionalExtension, ToSql};
use serenity::model::prelude::UserId as DiscordUserID;
use std::{path::Path, str::FromStr};

#[derive(Debug, Clone)]
pub struct Database {
    pool: Pool<SqliteConnectionManager>,
}

impl Database {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let manager = SqliteConnectionManager::file(path);
        let pool = r2d2::Pool::new(manager).unwrap();

        let database = Self { pool };
        database.initialize();
        database
    }

    fn initialize(&self) {
        self.pool
            .get()
            .unwrap()
            .execute(
                "
            CREATE TABLE IF NOT EXISTS angels (
                id TEXT PRIMARY KEY,
                discord_id TEXT NOT NULL UNIQUE,
                discord_name TEXT NOT NULL,
                minecraft_name TEXT NOT NULL UNIQUE,
                minecraft_type TEXT NOT NULL
            )
                ",
                (),
            )
            .unwrap();
    }

    fn get_angel_by(&self, selector: &'static str, by: impl ToSql) -> Option<Angel> {
        let connection = self.pool.get().unwrap();

        let mut query = connection
            .prepare(&format!("SELECT * FROM angels WHERE {selector}=?"))
            .unwrap();
        query
            .query_row([by], |row| {
                Ok(Angel {
                    id: AngelID::from_str(row.get::<_, String>("id")?.as_str()).unwrap(),
                    discord_id: DiscordUserID::from_str(row.get::<_, String>("discord_id")?.as_str()).unwrap(),
                    discord_name: row.get("discord_name")?,
                    minecraft_name: row.get("minecraft_name")?,
                    minecraft_type: MinecraftType::from_str(row.get::<_, String>("minecraft_type")?.as_str()).unwrap(),
                })
            })
            .optional()
            .unwrap()
    }

    pub fn get_angel_by_discord_id(&self, discord_id: DiscordUserID) -> Option<Angel> {
        self.get_angel_by("discord_id", discord_id.as_u64())
    }

    pub fn get_angel_by_minecraft_name(&self, minecraft_name: &str) -> Option<Angel> {
        self.get_angel_by("minecraft_name", minecraft_name)
    }

    pub fn get_angel_by_id(&self, id: &AngelID) -> Option<Angel> {
        self.get_angel_by("id", id.to_string())
    }

    pub fn insert_angel(&self, angel: &Angel) {
        let n = self
            .pool
            .get()
            .unwrap()
            .execute(
                "
            INSERT INTO angels (
                id,
                discord_id,
                discord_name,
                minecraft_name,
                minecraft_type
            ) VALUES (
                ?,
                ?,
                ?,
                ?,
                ?
            )
                ",
                &[
                    &angel.id.to_string(),
                    &angel.discord_id.to_string(),
                    &angel.discord_name,
                    &angel.minecraft_name,
                    &angel.minecraft_type.to_string(),
                ],
            )
            .unwrap();

        assert_eq!(n, 1);
        tracing::info!("added user to the database");
    }
}