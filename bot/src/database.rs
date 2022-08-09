use crate::models::{User, MinecraftType};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::OptionalExtension;
use serenity::model::prelude::UserId;
use std::{path::Path, str::FromStr};

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
            CREATE TABLE IF NOT EXISTS users (
                discord_id TEXT PRIMARY KEY,
                discord_name TEXT NOT NULL,
                minecraft_name TEXT NOT NULL,
                minecraft_type TEXT NOT NULL
            )
                ",
                (),
            )
            .unwrap();
    }

    pub fn get_user(&self, discord_id: UserId) -> Option<User> {
        let connection = self.pool.get().unwrap();

        let mut query = connection
            .prepare(
                "SELECT discord_name, minecraft_name, minecraft_type FROM users WHERE discord_id=?",
            )
            .unwrap();
        query
            .query_row([discord_id.as_u64()], |v| {
                Ok(User {
                    discord_id,
                    discord_name: v.get(0)?,
                    minecraft_name: v.get(1)?,
                    minecraft_type: MinecraftType::from_str(&v.get::<_, String>(2)?).unwrap(),
                })
            })
            .optional()
            .unwrap()
    }

    pub fn insert_user(&self, user: &User) {
        let n = self.pool
            .get()
            .unwrap()
            .execute(
                "
            INSERT INTO users (
                discord_id,
                discord_name,
                minecraft_name,
                minecraft_type
            ) VALUES (
                ?,
                ?,
                ?,
                ?
            )
                ",
                &[
                    &user.discord_id.to_string(),
                    &user.discord_name,
                    &user.minecraft_name,
                    &user.minecraft_type.to_string(),
                ],
            )
            .unwrap();

        assert_eq!(n, 1);
        tracing::info!("added user to the database");
    }
}
