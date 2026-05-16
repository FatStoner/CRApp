use crate::error::DbError;
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

        match Self::try_init(db_url, db_path).await {
            Ok(db) => Ok(db),
            Err(e) => {
                if std::path::Path::new(db_path).exists() {
                    tracing::error!(
                        "Database initialization failed: {}. Attempting corruption isolation and automatic recovery...",
                        e
                    );
                    let ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    let corrupt_path = format!("crap_data.corrupted.{}.db", ts);
                    if let Ok(_) = std::fs::rename(db_path, &corrupt_path) {
                        tracing::warn!(
                            "Corrupted database isolated at '{}'. Creating fresh database...",
                            corrupt_path
                        );
                        return Self::try_init(db_url, db_path).await;
                    }
                }
                Err(e)
            }
        }
    }

    async fn try_init(db_url: &str, db_path: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        // 1. Safety Backup
        if std::path::Path::new(db_path).exists() {
            tracing::info!("Found existing database. Creating safety backup at 'crap_data.db.bak'...");
            if let Err(e) = std::fs::copy(db_path, "crap_data.db.bak") {
                tracing::warn!("Failed to create database backup: {}", e);
            }
        } else {
            tracing::info!("Creating database {}", db_url);
            Sqlite::create_database(db_url).await?;
        }

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await?;

        // 2. Run Migrations
        tracing::info!("Checking for migrations (templates support)...");
        tracing::info!("Applying migrations...");
        if let Err(e) = MIGRATOR.run(&pool).await {
            let err_msg = e.to_string();
            if err_msg.contains("duplicate column name: content") {
                tracing::info!("Note: 'content' column already exists in 'lorebooks', skipping that part of migration.");
            } else {
                return Err(Box::new(e));
            }
        }
        tracing::info!("Migrations applied successfully.");

        // Manual Migration Fixes

        // 1. Ensure 'templates' table exists
        tracing::info!("Verifying schema for 'templates'...");
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
        .map_err(|e| tracing::warn!("Failed to ensure templates table exists: {}", e));

        // 2. Ensure 'image_path' in 'collections'
        tracing::info!("Verifying schema for 'collections'...");
        let _ = sqlx::query("ALTER TABLE collections ADD COLUMN image_path TEXT")
            .execute(&pool)
            .await
            .map_err(|e| {
                if !e.to_string().contains("duplicate column name") {
                    tracing::warn!("Failed to add image_path column: {}", e);
                }
            });

        // 3. Ensure 'spell_check_overrides_json' in 'characters'
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN spell_check_overrides_json TEXT")
            .execute(&pool)
            .await
            .map_err(|e| {
                if !e.to_string().contains("duplicate column name") {
                    tracing::warn!(
                        "Failed to add spell_check_overrides_json column: {}",
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
                        tracing::warn!("Failed to add quick_notes column: {}", e);
                    }
                });

        // 5. Ensure 'is_nsfw' in 'characters'
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN is_nsfw BOOLEAN NOT NULL DEFAULT 0")
            .execute(&pool)
            .await
            .map_err(|e| {
                if !e.to_string().contains("duplicate column name") {
                    tracing::warn!("Failed to add is_nsfw column: {}", e);
                }
            });

        // 6. Ensure 'blur_avatar' in 'characters'
        let _ =
            sqlx::query("ALTER TABLE characters ADD COLUMN blur_avatar BOOLEAN NOT NULL DEFAULT 0")
                .execute(&pool)
                .await
                .map_err(|e| {
                    if !e.to_string().contains("duplicate column name") {
                        tracing::warn!("Failed to add blur_avatar column: {}", e);
                    }
                });

        Ok(Database { pool })
    }

    // --- Characters ---
    pub async fn get_all_characters(&self) -> Result<Vec<Character>, DbError> {
        characters::get_all(&self.pool).await
    }

    pub async fn upsert_character(&self, character: &mut Character) -> Result<(), DbError> {
        characters::upsert(&self.pool, character).await
    }

    pub async fn delete_character(&self, id: i64) -> Result<(), DbError> {
        characters::delete(&self.pool, id).await
    }

    pub async fn move_character(
        &self,
        char_id: i64,
        collection_id: Option<i64>,
    ) -> Result<(), DbError> {
        characters::move_to_collection(&self.pool, char_id, collection_id).await
    }

    pub async fn get_characters_by_ids(&self, ids: &[i64]) -> Result<Vec<Character>, DbError> {
        characters::get_by_ids(&self.pool, ids).await
    }

    pub async fn search_characters_text(&self, query: &str) -> Result<Vec<Character>, DbError> {
        characters::search_text(&self.pool, query).await
    }

    pub async fn get_all_character_urls_flat(&self) -> Result<Vec<CharacterUrl>, DbError> {
        characters::get_all_urls_flat(&self.pool).await
    }

    // --- Collections ---
    pub async fn get_all_collections(&self) -> Result<Vec<Collection>, DbError> {
        collections::get_all(&self.pool).await
    }

    pub async fn upsert_collection(&self, collection: &Collection) -> Result<i64, DbError> {
        collections::upsert(&self.pool, collection).await
    }

    pub async fn delete_collection(&self, id: i64) -> Result<(), DbError> {
        collections::delete(&self.pool, id).await
    }

    pub async fn reorder_collection(&self, id: i64, move_up: bool) -> Result<(), DbError> {
        collections::reorder(&self.pool, id, move_up).await
    }

    // --- Tags ---
    pub async fn get_tags_for_character(
        &self,
        char_id: i64,
        is_external: bool,
    ) -> Result<Vec<Tag>, DbError> {
        tags::get_for_character(&self.pool, char_id, is_external).await
    }

    pub async fn add_tag_to_character(
        &self,
        char_id: i64,
        tag_name: &str,
        is_external: bool,
    ) -> Result<(), DbError> {
        tags::add_to_character(&self.pool, char_id, tag_name, is_external).await
    }

    pub async fn remove_tag_from_character(
        &self,
        char_id: i64,
        tag_id: i64,
        is_external: bool,
    ) -> Result<(), DbError> {
        tags::remove_from_character(&self.pool, char_id, tag_id, is_external).await
    }

    pub async fn remove_all_tags_from_character(
        &self,
        char_id: i64,
        is_external: bool,
    ) -> Result<(), DbError> {
        tags::remove_all_from_character(&self.pool, char_id, is_external).await
    }

    pub async fn get_all_tags_flat(
        &self,
        is_external: bool,
    ) -> Result<Vec<(i64, Tag)>, DbError> {
        tags::get_all_flat(&self.pool, is_external).await
    }

    pub async fn search_tags_matching(
        &self,
        query: &str,
    ) -> Result<Vec<(i64, String, bool)>, DbError> {
        tags::search_matching(&self.pool, query).await
    }

    // --- Lorebooks ---
    pub async fn get_all_lorebooks(&self) -> Result<Vec<Lorebook>, DbError> {
        lorebooks::get_all(&self.pool).await
    }

    pub async fn get_lorebooks_by_ids(&self, ids: &[i64]) -> Result<Vec<Lorebook>, DbError> {
        lorebooks::get_by_ids(&self.pool, ids).await
    }

    pub async fn search_lorebooks_text(&self, query: &str) -> Result<Vec<Lorebook>, DbError> {
        lorebooks::search_text(&self.pool, query).await
    }

    pub async fn upsert_lorebook(&self, lorebook: &mut Lorebook) -> Result<(), DbError> {
        lorebooks::upsert(&self.pool, lorebook).await
    }

    pub async fn delete_lorebook(&self, id: i64) -> Result<(), DbError> {
        lorebooks::delete(&self.pool, id).await
    }

    // Lorebook Entries
    pub async fn get_entries_for_lorebook(
        &self,
        lorebook_id: i64,
    ) -> Result<Vec<LorebookEntry>, DbError> {
        lorebooks::get_entries(&self.pool, lorebook_id).await
    }

    pub async fn add_entry_to_lorebook(&self, entry: &LorebookEntry) -> Result<i64, DbError> {
        lorebooks::add_entry(&self.pool, entry).await
    }

    pub async fn update_lorebook_entry(&self, entry: &LorebookEntry) -> Result<(), DbError> {
        lorebooks::update_entry(&self.pool, entry).await
    }

    pub async fn delete_lorebook_entry(&self, id: i64) -> Result<(), DbError> {
        lorebooks::delete_entry(&self.pool, id).await
    }

    pub async fn search_lorebook_entries_text(
        &self,
        query: &str,
    ) -> Result<Vec<LorebookEntry>, DbError> {
        lorebooks::search_entries_text(&self.pool, query).await
    }

    // Lorebook Tags
    pub async fn add_tag_to_lorebook(
        &self,
        lorebook_id: i64,
        tag_name: &str,
    ) -> Result<(), DbError> {
        lorebooks::add_tag(&self.pool, lorebook_id, tag_name).await
    }

    pub async fn remove_tag_from_lorebook(
        &self,
        lorebook_id: i64,
        tag_id: i64,
    ) -> Result<(), DbError> {
        lorebooks::remove_tag(&self.pool, lorebook_id, tag_id).await
    }

    pub async fn get_tags_for_lorebook(&self, lorebook_id: i64) -> Result<Vec<Tag>, DbError> {
        lorebooks::get_tags(&self.pool, lorebook_id).await
    }

    pub async fn get_all_lorebook_tags_flat(&self) -> Result<Vec<(i64, Tag)>, DbError> {
        lorebooks::get_all_tags_flat(&self.pool).await
    }

    pub async fn search_lorebook_tags_matching(
        &self,
        query: &str,
    ) -> Result<Vec<(i64, String)>, DbError> {
        lorebooks::search_tags_matching(&self.pool, query).await
    }

    // Lore Links
    pub async fn get_lore_links(&self, character_id: i64) -> Result<HashSet<i64>, DbError> {
        lorebooks::get_links(&self.pool, character_id).await
    }

    pub async fn link_lore(&self, character_id: i64, lore_id: i64) -> Result<(), DbError> {
        lorebooks::link(&self.pool, character_id, lore_id).await
    }

    pub async fn unlink_lore(&self, character_id: i64, lore_id: i64) -> Result<(), DbError> {
        lorebooks::unlink(&self.pool, character_id, lore_id).await
    }

    pub async fn get_all_lore_links_flat(&self) -> Result<Vec<(i64, i64)>, DbError> {
        lorebooks::get_all_links_flat(&self.pool).await
    }

    // --- Templates ---
    pub async fn get_all_templates(&self) -> Result<Vec<Template>, DbError> {
        templates::get_all(&self.pool).await
    }

    pub async fn upsert_template(&self, template: &mut Template) -> Result<(), DbError> {
        templates::upsert(&self.pool, template).await
    }

    pub async fn delete_template(&self, id: i64) -> Result<(), DbError> {
        templates::delete(&self.pool, id).await
    }

    // --- Settings ---
    pub async fn get_setting(&self, key: &str) -> Result<Option<String>, DbError> {
        let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.0))
    }

    pub async fn set_setting(&self, key: &str, value: &str) -> Result<(), DbError> {
        sqlx::query("INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = ?")
            .bind(key)
            .bind(value)
            .bind(value)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // --- DB Management ---
    pub async fn checkpoint(&self) -> Result<(), DbError> {
        sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn create_checkpoint_and_vacuum(&self, target_path: &str) -> Result<(), DbError> {
        self.checkpoint().await?;
        let query = format!("VACUUM INTO '{}'", target_path);
        sqlx::query(&query).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }
}
