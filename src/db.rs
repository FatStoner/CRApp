use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::error::Error;

#[derive(Clone)]
pub struct Database {
    pub pool: Pool<Sqlite>,
}

impl Database {
    pub async fn init() -> Result<Self, Box<dyn Error>> {
        let db_url = "sqlite://crap_data.db";

        if !Sqlite::database_exists(db_url).await.unwrap_or(false) {
            println!("Creating database {}", db_url);
            Sqlite::create_database(db_url).await?;
        } else {
            println!("Database already exists");
        }

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await?;

        // Create characters table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS characters (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                char_name TEXT NOT NULL,
                char_title TEXT NOT NULL,
                personality TEXT NOT NULL,
                first_message TEXT NOT NULL,
                author_notes TEXT NOT NULL,
                avatar_path TEXT,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )"
        )
        .execute(&pool)
        .await?;

        // Create lorebooks table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS lorebooks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                keywords TEXT
            )"
        )
        .execute(&pool)
        .await?;
        
        // Create character_lore_link table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS character_lore_link (
                character_id INTEGER NOT NULL,
                lore_id INTEGER NOT NULL,
                PRIMARY KEY (character_id, lore_id),
                FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE,
                FOREIGN KEY (lore_id) REFERENCES lorebooks(id) ON DELETE CASCADE
            )"
        )
        .execute(&pool)
        .await?;

        Ok(Database { pool })
    }

    pub async fn get_all_characters(&self) -> Result<Vec<crate::models::Character>, sqlx::Error> {
        sqlx::query_as::<_, crate::models::Character>("SELECT * FROM characters")
            .fetch_all(&self.pool)
            .await
    }
}
