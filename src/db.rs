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
                scenario TEXT NOT NULL DEFAULT '',
                example_dialogue TEXT NOT NULL DEFAULT '',
                first_message TEXT NOT NULL,
                author_notes TEXT NOT NULL,
                avatar_path TEXT,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )"
        )
        .execute(&pool)
        .await?;

        // Migration: Attempt to add columns to existing table
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN scenario TEXT NOT NULL DEFAULT ''").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN example_dialogue TEXT NOT NULL DEFAULT ''").execute(&pool).await;


        // Create lorebooks table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS lorebooks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                cover_path TEXT
            )"
        )
        .execute(&pool)
        .await?;

        // Migration: Attempt to add columns to existing table
        let _ = sqlx::query("ALTER TABLE lorebooks ADD COLUMN description TEXT NOT NULL DEFAULT ''").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE lorebooks ADD COLUMN cover_path TEXT").execute(&pool).await;
        
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

    pub async fn upsert_character(&self, character: &mut crate::models::Character) -> Result<(), sqlx::Error> {
        character.updated_at = chrono::Utc::now();
        
        if character.id == 0 {
            // INSERT
            let id = sqlx::query(
                "INSERT INTO characters (name, char_name, char_title, personality, scenario, example_dialogue, first_message, author_notes, avatar_path, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&character.name)
            .bind(&character.char_name)
            .bind(&character.char_title)
            .bind(&character.personality)
            .bind(&character.scenario)
            .bind(&character.example_dialogue)
            .bind(&character.first_message)
            .bind(&character.author_notes)
            .bind(&character.avatar_path)
            .bind(character.created_at)
            .bind(character.updated_at)
            .execute(&self.pool)
            .await?
            .last_insert_rowid();

            character.id = id;
        } else {
            // UPDATE
            sqlx::query(
                "UPDATE characters SET name=?, char_name=?, char_title=?, personality=?, scenario=?, example_dialogue=?, first_message=?, author_notes=?, avatar_path=?, updated_at=? WHERE id=?"
            )
            .bind(&character.name)
            .bind(&character.char_name)
            .bind(&character.char_title)
            .bind(&character.personality)
            .bind(&character.scenario)
            .bind(&character.example_dialogue)
            .bind(&character.first_message)
            .bind(&character.author_notes)
            .bind(&character.avatar_path)
            .bind(character.updated_at)
            .bind(character.id)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn get_all_lorebooks(&self) -> Result<Vec<crate::models::Lorebook>, sqlx::Error> {
        sqlx::query_as::<_, crate::models::Lorebook>("SELECT * FROM lorebooks")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn upsert_lorebook(&self, lorebook: &mut crate::models::Lorebook) -> Result<(), sqlx::Error> {
        if lorebook.id == 0 {
            // INSERT
            let id = sqlx::query(
                "INSERT INTO lorebooks (title, description, cover_path) VALUES (?, ?, ?)"
            )
            .bind(&lorebook.title)
            .bind(&lorebook.description)
            .bind(&lorebook.cover_path)
            .execute(&self.pool)
            .await?
            .last_insert_rowid();

            lorebook.id = id;
        } else {
            // UPDATE
            sqlx::query(
                "UPDATE lorebooks SET title=?, description=?, cover_path=? WHERE id=?"
            )
            .bind(&lorebook.title)
            .bind(&lorebook.description)
            .bind(&lorebook.cover_path)
            .bind(lorebook.id)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    // Returns a List of Lorebook IDs linked to the character
    pub async fn get_lore_links(&self, character_id: i64) -> Result<std::collections::HashSet<i64>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT lore_id FROM character_lore_link WHERE character_id = ?"
        )
        .bind(character_id)
        .fetch_all(&self.pool)
        .await?;

        use sqlx::Row;
        let set: std::collections::HashSet<i64> = rows.into_iter().map(|row| row.get("lore_id")).collect();
        Ok(set)
    }

    pub async fn link_lore(&self, character_id: i64, lore_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR IGNORE INTO character_lore_link (character_id, lore_id) VALUES (?, ?)"
        )
        .bind(character_id)
        .bind(lore_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn unlink_lore(&self, character_id: i64, lore_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "DELETE FROM character_lore_link WHERE character_id = ? AND lore_id = ?"
        )
        .bind(character_id)
        .bind(lore_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
