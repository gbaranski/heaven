use crate::models::User;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::Path;

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
            CREATE TABLE users (
                discord_id TEXT PRIMARY KEY,
                discord_name TEXT NOT NULL,
                minecraft_name TEXT NOT NULL,
                minecraft_type TEXT NOT NULL,
            )
                ",
                (),
            )
            .unwrap();
    }

    pub fn user_exists(&self, discord_id: &str) -> bool {
        let connection = self.pool.get().unwrap();

        let mut query = connection
            .prepare("SELECT 1 FROM users WHERE discord_id=?")
            .unwrap();
        query.exists([discord_id]).unwrap()
    }

    pub fn insert_user(&self, user: User) {
        self.pool
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
                    &user.discord_id,
                    &user.discord_name,
                    &user.minecraft_name,
                    &user.minecraft_type.to_string(),
                ],
            )
            .unwrap();
    }
}
