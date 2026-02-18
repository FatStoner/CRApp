use crate::models::{Character, CharacterUrl, Collection, Lorebook, LorebookEntry, Tag, Template};
use sqlx::{
    migrate::{MigrateDatabase, Migrator},
    sqlite::SqlitePoolOptions,
    Pool, Sqlite,
};
use std::collections::HashSet;
use std::error::Error;

pub mod characters;
pub mod collections;
pub mod lorebooks;
pub mod tags;
pub mod templates;

#[derive(Clone, Debug)]
pub struct Database {
    pub pool: Pool<Sqlite>,
}

static MIGRATOR: Migrator = sqlx::migrate!();

impl Database {
    pub async fn init() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let db_url = "sqlite://crap_data.db";
        let db_path = "crap_data.db";

        // 1. Safety Backup
        if std::path::Path::new(db_path).exists() {
            println!("Found existing database. Creating safety backup at 'crap_data.db.bak'...");
            if let Err(e) = std::fs::copy(db_path, "crap_data.db.bak") {
                eprintln!("WARNING: Failed to create database backup: {}", e);
            }
        } else {
            println!("Creating database {}", db_url);
            Sqlite::create_database(db_url).await?;
        }

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await?;

        // 2. Run Migrations
        println!("Checking for migrations (templates support)...");
        println!("Applying migrations...");
        if let Err(e) = MIGRATOR.run(&pool).await {
            let err_msg = e.to_string();
            if err_msg.contains("duplicate column name: content") {
                println!("Note: 'content' column already exists in 'lorebooks', skipping that part of migration.");
            } else {
                return Err(Box::new(e));
            }
        }
        println!("Migrations applied successfully.");

        // Manual Migration Fixes

        // 1. Ensure 'templates' table exists
        println!("Verifying schema for 'templates'...");
        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS templates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                title TEXT NOT NULL,
                first_message TEXT NOT NULL DEFAULT '',
                personality TEXT NOT NULL DEFAULT '',
                scenario TEXT NOT NULL DEFAULT '',
                example_dialogue TEXT NOT NULL DEFAULT '',
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            );",
        )
        .execute(&pool)
        .await
        .map_err(|e| eprintln!("Warning: Failed to ensure templates table exists: {}", e));

        // 2. Ensure 'image_path' in 'collections'
        println!("Verifying schema for 'collections'...");
        let _ = sqlx::query("ALTER TABLE collections ADD COLUMN image_path TEXT")
            .execute(&pool)
            .await
            .map_err(|e| {
                if !e.to_string().contains("duplicate column name") {
                    eprintln!("Warning: Failed to add image_path column: {}", e);
                }
            });

        // 3. Ensure 'spell_check_overrides_json' in 'characters'
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN spell_check_overrides_json TEXT")
            .execute(&pool)
            .await
            .map_err(|e| {
                if !e.to_string().contains("duplicate column name") {
                    eprintln!(
                        "Warning: Failed to add spell_check_overrides_json column: {}",
                        e
                    );
                }
            });

        // 4. Ensure 'quick_notes' in 'characters'
        let _ =
            sqlx::query("ALTER TABLE characters ADD COLUMN quick_notes TEXT NOT NULL DEFAULT ''")
                .execute(&pool)
                .await
                .map_err(|e| {
                    if !e.to_string().contains("duplicate column name") {
                        eprintln!("Warning: Failed to add quick_notes column: {}", e);
                    }
                });

        // 5. Ensure 'is_nsfw' in 'characters'
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN is_nsfw BOOLEAN NOT NULL DEFAULT 0")
            .execute(&pool)
            .await
            .map_err(|e| {
                if !e.to_string().contains("duplicate column name") {
                    eprintln!("Warning: Failed to add is_nsfw column: {}", e);
                }
            });

        // 6. Ensure 'blur_avatar' in 'characters'
        let _ =
            sqlx::query("ALTER TABLE characters ADD COLUMN blur_avatar BOOLEAN NOT NULL DEFAULT 0")
                .execute(&pool)
                .await
                .map_err(|e| {
                    if !e.to_string().contains("duplicate column name") {
                        eprintln!("Warning: Failed to add blur_avatar column: {}", e);
                    }
                });

        Ok(Database { pool })
    }

    // --- Characters ---
    pub async fn get_all_characters(&self) -> Result<Vec<Character>, sqlx::Error> {
        characters::get_all(&self.pool).await
    }

    pub async fn upsert_character(&self, character: &mut Character) -> Result<(), sqlx::Error> {
        characters::upsert(&self.pool, character).await
    }

    pub async fn delete_character(&self, id: i64) -> Result<(), sqlx::Error> {
        characters::delete(&self.pool, id).await
    }

    pub async fn move_character(
        &self,
        char_id: i64,
        collection_id: Option<i64>,
    ) -> Result<(), sqlx::Error> {
        characters::move_to_collection(&self.pool, char_id, collection_id).await
    }

    pub async fn get_characters_by_ids(&self, ids: &[i64]) -> Result<Vec<Character>, sqlx::Error> {
        characters::get_by_ids(&self.pool, ids).await
    }

    pub async fn search_characters_text(&self, query: &str) -> Result<Vec<Character>, sqlx::Error> {
        characters::search_text(&self.pool, query).await
    }

    pub async fn get_all_character_urls_flat(&self) -> Result<Vec<CharacterUrl>, sqlx::Error> {
        characters::get_all_urls_flat(&self.pool).await
    }

    // --- Collections ---
    pub async fn get_all_collections(&self) -> Result<Vec<Collection>, sqlx::Error> {
        collections::get_all(&self.pool).await
    }

    pub async fn upsert_collection(&self, collection: &Collection) -> Result<i64, sqlx::Error> {
        collections::upsert(&self.pool, collection).await
    }

    pub async fn delete_collection(&self, id: i64) -> Result<(), sqlx::Error> {
        collections::delete(&self.pool, id).await
    }

    pub async fn reorder_collection(&self, id: i64, move_up: bool) -> Result<(), sqlx::Error> {
        collections::reorder(&self.pool, id, move_up).await
    }

    // --- Tags ---
    pub async fn get_tags_for_character(
        &self,
        char_id: i64,
        is_external: bool,
    ) -> Result<Vec<Tag>, sqlx::Error> {
        tags::get_for_character(&self.pool, char_id, is_external).await
    }

    pub async fn add_tag_to_character(
        &self,
        char_id: i64,
        tag_name: &str,
        is_external: bool,
    ) -> Result<(), sqlx::Error> {
        tags::add_to_character(&self.pool, char_id, tag_name, is_external).await
    }

    pub async fn remove_tag_from_character(
        &self,
        char_id: i64,
        tag_id: i64,
        is_external: bool,
    ) -> Result<(), sqlx::Error> {
        tags::remove_from_character(&self.pool, char_id, tag_id, is_external).await
    }

    pub async fn remove_all_tags_from_character(
        &self,
        char_id: i64,
        is_external: bool,
    ) -> Result<(), sqlx::Error> {
        tags::remove_all_from_character(&self.pool, char_id, is_external).await
    }

    pub async fn get_all_tags_flat(
        &self,
        is_external: bool,
    ) -> Result<Vec<(i64, Tag)>, sqlx::Error> {
        tags::get_all_flat(&self.pool, is_external).await
    }

    pub async fn search_tags_matching(
        &self,
        query: &str,
    ) -> Result<Vec<(i64, String, bool)>, sqlx::Error> {
        tags::search_matching(&self.pool, query).await
    }

    // --- Lorebooks ---
    pub async fn get_all_lorebooks(&self) -> Result<Vec<Lorebook>, sqlx::Error> {
        lorebooks::get_all(&self.pool).await
    }

    pub async fn get_lorebooks_by_ids(&self, ids: &[i64]) -> Result<Vec<Lorebook>, sqlx::Error> {
        lorebooks::get_by_ids(&self.pool, ids).await
    }

    pub async fn search_lorebooks_text(&self, query: &str) -> Result<Vec<Lorebook>, sqlx::Error> {
        lorebooks::search_text(&self.pool, query).await
    }

    pub async fn upsert_lorebook(&self, lorebook: &mut Lorebook) -> Result<(), sqlx::Error> {
        lorebooks::upsert(&self.pool, lorebook).await
    }

    pub async fn delete_lorebook(&self, id: i64) -> Result<(), sqlx::Error> {
        lorebooks::delete(&self.pool, id).await
    }

    // Lorebook Entries
    pub async fn get_entries_for_lorebook(
        &self,
        lorebook_id: i64,
    ) -> Result<Vec<LorebookEntry>, sqlx::Error> {
        lorebooks::get_entries(&self.pool, lorebook_id).await
    }

    pub async fn add_entry_to_lorebook(&self, entry: &LorebookEntry) -> Result<i64, sqlx::Error> {
        lorebooks::add_entry(&self.pool, entry).await
    }

    pub async fn update_lorebook_entry(&self, entry: &LorebookEntry) -> Result<(), sqlx::Error> {
        lorebooks::update_entry(&self.pool, entry).await
    }

    pub async fn delete_lorebook_entry(&self, id: i64) -> Result<(), sqlx::Error> {
        lorebooks::delete_entry(&self.pool, id).await
    }

    pub async fn search_lorebook_entries_text(
        &self,
        query: &str,
    ) -> Result<Vec<LorebookEntry>, sqlx::Error> {
        lorebooks::search_entries_text(&self.pool, query).await
    }

    // Lorebook Tags
    pub async fn add_tag_to_lorebook(
        &self,
        lorebook_id: i64,
        tag_name: &str,
    ) -> Result<(), sqlx::Error> {
        lorebooks::add_tag(&self.pool, lorebook_id, tag_name).await
    }

    pub async fn remove_tag_from_lorebook(
        &self,
        lorebook_id: i64,
        tag_id: i64,
    ) -> Result<(), sqlx::Error> {
        lorebooks::remove_tag(&self.pool, lorebook_id, tag_id).await
    }

    pub async fn get_tags_for_lorebook(&self, lorebook_id: i64) -> Result<Vec<Tag>, sqlx::Error> {
        lorebooks::get_tags(&self.pool, lorebook_id).await
    }

    pub async fn get_all_lorebook_tags_flat(&self) -> Result<Vec<(i64, Tag)>, sqlx::Error> {
        lorebooks::get_all_tags_flat(&self.pool).await
    }

    pub async fn search_lorebook_tags_matching(
        &self,
        query: &str,
    ) -> Result<Vec<(i64, String)>, sqlx::Error> {
        lorebooks::search_tags_matching(&self.pool, query).await
    }

    // Lore Links
    pub async fn get_lore_links(&self, character_id: i64) -> Result<HashSet<i64>, sqlx::Error> {
        lorebooks::get_links(&self.pool, character_id).await
    }

    pub async fn link_lore(&self, character_id: i64, lore_id: i64) -> Result<(), sqlx::Error> {
        lorebooks::link(&self.pool, character_id, lore_id).await
    }

    pub async fn unlink_lore(&self, character_id: i64, lore_id: i64) -> Result<(), sqlx::Error> {
        lorebooks::unlink(&self.pool, character_id, lore_id).await
    }

    pub async fn get_all_lore_links_flat(&self) -> Result<Vec<(i64, i64)>, sqlx::Error> {
        lorebooks::get_all_links_flat(&self.pool).await
    }

    // --- Templates ---
    pub async fn get_all_templates(&self) -> Result<Vec<Template>, sqlx::Error> {
        templates::get_all(&self.pool).await
    }

    pub async fn upsert_template(&self, template: &mut Template) -> Result<(), sqlx::Error> {
        templates::upsert(&self.pool, template).await
    }

    pub async fn delete_template(&self, id: i64) -> Result<(), sqlx::Error> {
        templates::delete(&self.pool, id).await
    }

    // --- Settings ---
    pub async fn get_setting(&self, key: &str) -> Result<Option<String>, sqlx::Error> {
        let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.0))
    }

    pub async fn set_setting(&self, key: &str, value: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = ?")
            .bind(key)
            .bind(value)
            .bind(value)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // --- DB Management ---
    pub async fn checkpoint(&self) -> Result<(), sqlx::Error> {
        sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn create_checkpoint_and_vacuum(&self, target_path: &str) -> Result<(), sqlx::Error> {
        self.checkpoint().await?;
        let query = format!("VACUUM INTO '{}'", target_path);
        sqlx::query(&query).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }
}
