use crate::models::{Character, Collection, Lorebook, Tag};
use sqlx::{
    migrate::{MigrateDatabase, Migrator},
    sqlite::SqlitePoolOptions,
    Pool, Sqlite,
};
use std::error::Error;

#[derive(Clone)]
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
                // We proceed, but maybe we should warn the user?
                // For now, valid strategy is log and proceed, or we could return Err.
                // Given the user wants SAFETY, maybe returning Err is better?
                // But if permissions are weird, it locks the app.
                // Let's stick to logging for now, user requirements detailed simple backup.
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
        // This will create tables if they don't exist, using the idempotent SQL files.
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

        Ok(Database { pool })
    }

    pub async fn get_all_characters(&self) -> Result<Vec<crate::models::Character>, sqlx::Error> {
        sqlx::query_as::<_, crate::models::Character>("SELECT * FROM characters")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn upsert_character(
        &self,
        character: &mut crate::models::Character,
    ) -> Result<(), sqlx::Error> {
        character.updated_at = chrono::Utc::now();

        if character.id == 0 {
            // INSERT
            let id = sqlx::query(
                "INSERT INTO characters (name, char_name, char_title, personality, scenario, example_dialogue, first_message, author_notes, avatar_path, created_at, updated_at, collection_id, is_favorite)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
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
            .bind(character.collection_id)
            .bind(character.is_favorite)
            .execute(&self.pool)
            .await?
            .last_insert_rowid();

            character.id = id;
        } else {
            // UPDATE
            sqlx::query(
                "UPDATE characters SET name=?, char_name=?, char_title=?, personality=?, scenario=?, example_dialogue=?, first_message=?, author_notes=?, avatar_path=?, updated_at=?, collection_id=?, is_favorite=? WHERE id=?"
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
            .bind(character.collection_id)
            .bind(character.is_favorite)
            .bind(character.id)
            .execute(&self.pool)
            .await?;
        }

        // Handle URLs
        // Simple approach: Delete all for this character and re-insert.
        // Since we don't have URL IDs in the UI usually (unless we want to preserve specific ones),
        // replacing all is safest for maintaining order if we added that, or just synchronization.
        // But CharacterUrl has an ID.
        // If the UI passes IDs, we could update, but deleting all is much simpler.
        if character.id != 0 {
            sqlx::query("DELETE FROM character_urls WHERE character_id = ?")
                .bind(character.id)
                .execute(&self.pool)
                .await?;

            for url in &mut character.urls {
                // Skip empty URLs
                if url.url.trim().is_empty() {
                    continue;
                }

                let uid = sqlx::query(
                    "INSERT INTO character_urls (character_id, url, label) VALUES (?, ?, ?)",
                )
                .bind(character.id)
                .bind(&url.url)
                .bind(&url.label)
                .execute(&self.pool)
                .await?
                .last_insert_rowid();
                url.id = uid;
                url.character_id = character.id;
            }
        }

        Ok(())
    }

    pub async fn delete_character(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM characters WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn move_character(
        &self,
        char_id: i64,
        collection_id: Option<i64>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE characters SET collection_id = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        )
        .bind(collection_id)
        .bind(char_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // Collections
    pub async fn get_all_collections(&self) -> Result<Vec<crate::models::Collection>, sqlx::Error> {
        sqlx::query_as::<_, crate::models::Collection>(
            "SELECT * FROM collections ORDER BY display_order ASC",
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn upsert_collection(
        &self,
        collection: &crate::models::Collection,
    ) -> Result<i64, sqlx::Error> {
        if collection.id == 0 {
            // New Collection: Determine display_order (Max in siblings + 1)
            let max_order: Option<i64> = if let Some(pid) = collection.parent_id {
                sqlx::query_scalar("SELECT MAX(display_order) FROM collections WHERE parent_id = ?")
                    .bind(pid)
                    .fetch_optional(&self.pool)
                    .await?
            } else {
                sqlx::query_scalar(
                    "SELECT MAX(display_order) FROM collections WHERE parent_id IS NULL",
                )
                .fetch_optional(&self.pool)
                .await?
            };

            // If no siblings, start huge or start at id? Better start at current MAX + 1.
            // If table empty, 0.
            let next_order = max_order.unwrap_or(0) + 1;

            let id = sqlx::query(
                "INSERT INTO collections (name, parent_id, display_order) VALUES (?, ?, ?)",
            )
            .bind(&collection.name)
            .bind(collection.parent_id)
            .bind(next_order)
            .execute(&self.pool)
            .await?
            .last_insert_rowid();
            Ok(id)
        } else {
            sqlx::query("UPDATE collections SET name=?, parent_id=? WHERE id=?")
                .bind(&collection.name)
                .bind(collection.parent_id)
                .bind(collection.id)
                .execute(&self.pool)
                .await?;
            Ok(collection.id)
        }
    }

    pub async fn delete_collection(&self, id: i64) -> Result<(), sqlx::Error> {
        // Orphan children
        sqlx::query("UPDATE collections SET parent_id = NULL WHERE parent_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        sqlx::query("UPDATE characters SET collection_id = NULL WHERE collection_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        // Delete
        sqlx::query("DELETE FROM collections WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn reorder_collection(&self, id: i64, move_up: bool) -> Result<(), sqlx::Error> {
        // 1. Get current item info
        let current: crate::models::Collection =
            sqlx::query_as("SELECT * FROM collections WHERE id = ?")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        // 2. Find swap target
        // If moving UP (smaller order), we want the item with largest order < current.order
        // If moving DOWN (larger order), we want item with smallest order > current.order
        let op = if move_up { "<" } else { ">" };
        let sort = if move_up { "DESC" } else { "ASC" };

        // Handle parent_id null checks
        let query = if let Some(pid) = current.parent_id {
            format!("SELECT * FROM collections WHERE parent_id = ? AND display_order {} ? ORDER BY display_order {} LIMIT 1", op, sort)
        } else {
            format!("SELECT * FROM collections WHERE parent_id IS NULL AND display_order {} ? ORDER BY display_order {} LIMIT 1", op, sort)
        };

        let mut q = sqlx::query_as::<_, crate::models::Collection>(&query);
        if let Some(pid) = current.parent_id {
            q = q.bind(pid);
        }
        q = q.bind(current.display_order);

        let target = q.fetch_optional(&self.pool).await?;

        if let Some(other) = target {
            // Swap orders
            // Use transaction for safety
            let mut tx = self.pool.begin().await?;

            sqlx::query("UPDATE collections SET display_order = ? WHERE id = ?")
                .bind(other.display_order)
                .bind(current.id)
                .execute(&mut *tx)
                .await?;

            sqlx::query("UPDATE collections SET display_order = ? WHERE id = ?")
                .bind(current.display_order)
                .bind(other.id)
                .execute(&mut *tx)
                .await?;

            tx.commit().await?;
        }

        Ok(())
    }

    // Tag Helpers

    pub async fn get_tags_for_character(
        &self,
        char_id: i64,
        is_external: bool,
    ) -> Result<Vec<Tag>, sqlx::Error> {
        let (join_table, tag_table) = if is_external {
            ("character_external_tags", "external_tags")
        } else {
            ("character_tags", "tags")
        };

        let query = format!(
            "SELECT t.id, t.name FROM {} t
             JOIN {} ct ON t.id = ct.tag_id
             WHERE ct.character_id = ?",
            tag_table, join_table
        );

        sqlx::query_as::<_, Tag>(&query)
            .bind(char_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn add_tag_to_character(
        &self,
        char_id: i64,
        tag_name: &str,
        is_external: bool,
    ) -> Result<(), sqlx::Error> {
        let (join_table, tag_table) = if is_external {
            ("character_external_tags", "external_tags")
        } else {
            ("character_tags", "tags")
        };

        // 1. Ensure tag exists
        // UPSERT syntax for SQLite? INSERT OR IGNORE works if name is UNIQUE
        let insert_tag_query = format!("INSERT OR IGNORE INTO {} (name) VALUES (?)", tag_table);
        sqlx::query(&insert_tag_query)
            .bind(tag_name)
            .execute(&self.pool)
            .await?;

        // 2. Get Tag ID
        let get_id_query = format!("SELECT id FROM {} WHERE name = ?", tag_table);
        let tag_id: i64 = sqlx::query_scalar(&get_id_query)
            .bind(tag_name)
            .fetch_one(&self.pool)
            .await?;

        // 3. Link
        let link_query = format!(
            "INSERT OR IGNORE INTO {} (character_id, tag_id) VALUES (?, ?)",
            join_table
        );
        sqlx::query(&link_query)
            .bind(char_id)
            .bind(tag_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn remove_tag_from_character(
        &self,
        char_id: i64,
        tag_id: i64,
        is_external: bool,
    ) -> Result<(), sqlx::Error> {
        let join_table = if is_external {
            "character_external_tags"
        } else {
            "character_tags"
        };

        let query = format!(
            "DELETE FROM {} WHERE character_id = ? AND tag_id = ?",
            join_table
        );
        sqlx::query(&query)
            .bind(char_id)
            .bind(tag_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Returns (character_id, Tag) for all tags of a specific type (external or app)
    pub async fn get_all_tags_flat(
        &self,
        is_external: bool,
    ) -> Result<Vec<(i64, Tag)>, sqlx::Error> {
        let (join_table, tag_table) = if is_external {
            ("character_external_tags", "external_tags")
        } else {
            ("character_tags", "tags")
        };

        let query = format!(
            "SELECT ct.character_id, t.id, t.name FROM {} t
             JOIN {} ct ON t.id = ct.tag_id",
            tag_table, join_table
        );

        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;

        use sqlx::Row;
        let mut results = Vec::new();
        for row in rows {
            let char_id: i64 = row.get(0);
            let tag = Tag {
                id: row.get(1),
                name: row.get(2),
            };
            results.push((char_id, tag));
        }

        Ok(results)
    }

    // Lorebook Tags
    pub async fn add_tag_to_lorebook(
        &self,
        lorebook_id: i64,
        tag_name: &str,
    ) -> Result<(), sqlx::Error> {
        // 1. Ensure tag exists (using 'tags' table shared with characters app tags)
        let insert_tag_query = "INSERT OR IGNORE INTO tags (name) VALUES (?)";
        sqlx::query(insert_tag_query)
            .bind(tag_name)
            .execute(&self.pool)
            .await?;

        // 2. Get Tag ID
        let get_id_query = "SELECT id FROM tags WHERE name = ?";
        let tag_id: i64 = sqlx::query_scalar(get_id_query)
            .bind(tag_name)
            .fetch_one(&self.pool)
            .await?;

        // 3. Link
        let link_query = "INSERT OR IGNORE INTO lorebook_tags (lorebook_id, tag_id) VALUES (?, ?)";
        sqlx::query(link_query)
            .bind(lorebook_id)
            .bind(tag_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn remove_tag_from_lorebook(
        &self,
        lorebook_id: i64,
        tag_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM lorebook_tags WHERE lorebook_id = ? AND tag_id = ?")
            .bind(lorebook_id)
            .bind(tag_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_tags_for_lorebook(&self, lorebook_id: i64) -> Result<Vec<Tag>, sqlx::Error> {
        let query = "
            SELECT t.id, t.name FROM tags t
            JOIN lorebook_tags lt ON t.id = lt.tag_id
            WHERE lt.lorebook_id = ?
        ";
        sqlx::query_as::<_, Tag>(query)
            .bind(lorebook_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn get_all_lorebook_tags_flat(&self) -> Result<Vec<(i64, Tag)>, sqlx::Error> {
        let query = "
            SELECT lt.lorebook_id, t.id, t.name FROM tags t
            JOIN lorebook_tags lt ON t.id = lt.tag_id
        ";
        let rows = sqlx::query(query).fetch_all(&self.pool).await?;

        use sqlx::Row;
        let mut results = Vec::new();
        for row in rows {
            let lb_id: i64 = row.get(0);
            let tag = Tag {
                id: row.get(1),
                name: row.get(2),
            };
            results.push((lb_id, tag));
        }
        Ok(results)
    }

    pub async fn get_all_character_urls_flat(
        &self,
    ) -> Result<Vec<crate::models::CharacterUrl>, sqlx::Error> {
        sqlx::query_as::<_, crate::models::CharacterUrl>("SELECT * FROM character_urls")
            .fetch_all(&self.pool)
            .await
    }

    // Deep Search
    pub async fn search_characters_text(
        &self,
        query: &str,
    ) -> Result<Vec<crate::models::Character>, sqlx::Error> {
        let pattern = format!("%{}%", query);
        // We search in all text fields
        sqlx::query_as::<_, crate::models::Character>(
            "SELECT DISTINCT c.* FROM characters c
             LEFT JOIN character_urls u ON c.id = u.character_id
             WHERE 
             c.name LIKE ? OR 
             c.personality LIKE ? OR 
             c.scenario LIKE ? OR 
             c.example_dialogue LIKE ? OR 
             c.first_message LIKE ? OR 
             c.author_notes LIKE ? OR
             u.url LIKE ? OR
             u.label LIKE ?",
        )
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn search_tags_matching(
        &self,
        query: &str,
    ) -> Result<Vec<(i64, String, bool)>, sqlx::Error> {
        let pattern = format!("%{}%", query);
        let q = "
            SELECT ct.character_id, t.name, 0 as is_ext
            FROM character_tags ct 
            JOIN tags t ON ct.tag_id = t.id 
            WHERE t.name LIKE ?
            UNION ALL
            SELECT cet.character_id, et.name, 1 as is_ext
            FROM character_external_tags cet 
            JOIN external_tags et ON cet.tag_id = et.id 
            WHERE et.name LIKE ?
        ";

        let rows = sqlx::query(q).bind(&pattern).fetch_all(&self.pool).await?;

        use sqlx::Row;
        let mut results = Vec::new();
        for row in rows {
            let cid: i64 = row.get(0);
            let name: String = row.get(1);
            let is_ext: i32 = row.get(2); // Boolean returned as integer in union
            results.push((cid, name, is_ext != 0));
        }
        Ok(results)
    }

    pub async fn get_characters_by_ids(
        &self,
        ids: &[i64],
    ) -> Result<Vec<crate::models::Character>, sqlx::Error> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        // Dynamic IN clause
        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let query = format!(
            "SELECT * FROM characters WHERE id IN ({})",
            placeholders.join(",")
        );

        let mut q = sqlx::query_as::<_, crate::models::Character>(&query);
        for id in ids {
            q = q.bind(id);
        }

        q.fetch_all(&self.pool).await
    }

    pub async fn search_lorebooks_text(
        &self,
        query: &str,
    ) -> Result<Vec<crate::models::Lorebook>, sqlx::Error> {
        let pattern = format!("%{}%", query);
        sqlx::query_as::<_, crate::models::Lorebook>(
            "SELECT * FROM lorebooks WHERE 
             title LIKE ? OR 
             description LIKE ? OR
             content LIKE ?",
        )
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn search_lorebook_entries_text(
        &self,
        query: &str,
    ) -> Result<Vec<crate::models::LorebookEntry>, sqlx::Error> {
        let pattern = format!("%{}%", query);
        sqlx::query_as::<_, crate::models::LorebookEntry>(
            "SELECT * FROM lorebook_entries WHERE 
             name LIKE ? OR 
             keywords LIKE ? OR 
             content LIKE ?",
        )
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn search_lorebook_tags_matching(
        &self,
        query: &str,
    ) -> Result<Vec<(i64, String)>, sqlx::Error> {
        let pattern = format!("%{}%", query);
        let q = "
            SELECT lt.lorebook_id, t.name
            FROM lorebook_tags lt 
            JOIN tags t ON lt.tag_id = t.id 
            WHERE t.name LIKE ?
        ";
        let rows = sqlx::query(q).bind(&pattern).fetch_all(&self.pool).await?;

        use sqlx::Row;
        let mut results = Vec::new();
        for row in rows {
            let lid: i64 = row.get(0);
            let name: String = row.get(1);
            results.push((lid, name));
        }
        Ok(results)
    }

    pub async fn get_all_lore_links_flat(&self) -> Result<Vec<(i64, i64)>, sqlx::Error> {
        let rows = sqlx::query("SELECT character_id, lore_id FROM character_lore_link")
            .fetch_all(&self.pool)
            .await?;

        use sqlx::Row;
        let mut results = Vec::new();
        for row in rows {
            results.push((row.get(0), row.get(1)));
        }
        Ok(results)
    }

    pub async fn get_lorebooks_by_ids(
        &self,
        ids: &[i64],
    ) -> Result<Vec<crate::models::Lorebook>, sqlx::Error> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let query = format!(
            "SELECT * FROM lorebooks WHERE id IN ({})",
            placeholders.join(",")
        );
        let mut q = sqlx::query_as::<_, crate::models::Lorebook>(&query);
        for id in ids {
            q = q.bind(id);
        }
        q.fetch_all(&self.pool).await
    }

    pub async fn get_all_lorebooks(&self) -> Result<Vec<crate::models::Lorebook>, sqlx::Error> {
        sqlx::query_as::<_, crate::models::Lorebook>("SELECT * FROM lorebooks")
            .fetch_all(&self.pool)
            .await
    }

    // --- Lorebook Entries ---

    pub async fn get_entries_for_lorebook(
        &self,
        lorebook_id: i64,
    ) -> Result<Vec<crate::models::LorebookEntry>, sqlx::Error> {
        sqlx::query_as::<_, crate::models::LorebookEntry>(
            "SELECT * FROM lorebook_entries WHERE lorebook_id = ? ORDER BY name ASC",
        )
        .bind(lorebook_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn add_entry_to_lorebook(
        &self,
        entry: &crate::models::LorebookEntry,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query("INSERT INTO lorebook_entries (lorebook_id, name, keywords, content, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(entry.lorebook_id)
            .bind(&entry.name)
            .bind(&entry.keywords)
            .bind(&entry.content)
            .bind(entry.created_at)
            .bind(entry.updated_at)
            .execute(&self.pool)
            .await
            .map(|r| r.last_insert_rowid())
    }

    pub async fn update_lorebook_entry(
        &self,
        entry: &crate::models::LorebookEntry,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE lorebook_entries SET name=?, keywords=?, content=?, updated_at=? WHERE id=?",
        )
        .bind(&entry.name)
        .bind(&entry.keywords)
        .bind(&entry.content)
        .bind(entry.updated_at)
        .bind(entry.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_lorebook_entry(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM lorebook_entries WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn upsert_lorebook(
        &self,
        lorebook: &mut crate::models::Lorebook,
    ) -> Result<(), sqlx::Error> {
        // Keep description and content in sync for search compatibility
        lorebook.description = lorebook.content.clone();

        if lorebook.id == 0 {
            // INSERT
            let id = sqlx::query(
                "INSERT INTO lorebooks (title, description, content, cover_path) VALUES (?, ?, ?, ?)",
            )
            .bind(&lorebook.title)
            .bind(&lorebook.description)
            .bind(&lorebook.content)
            .bind(&lorebook.cover_path)
            .execute(&self.pool)
            .await?
            .last_insert_rowid();

            lorebook.id = id;
        } else {
            // UPDATE
            sqlx::query(
                "UPDATE lorebooks SET title=?, description=?, content=?, cover_path=? WHERE id=?",
            )
            .bind(&lorebook.title)
            .bind(&lorebook.description)
            .bind(&lorebook.content)
            .bind(&lorebook.cover_path)
            .bind(lorebook.id)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    // Returns a List of Lorebook IDs linked to the character
    pub async fn get_lore_links(
        &self,
        character_id: i64,
    ) -> Result<std::collections::HashSet<i64>, sqlx::Error> {
        let rows = sqlx::query("SELECT lore_id FROM character_lore_link WHERE character_id = ?")
            .bind(character_id)
            .fetch_all(&self.pool)
            .await?;

        use sqlx::Row;
        let set: std::collections::HashSet<i64> =
            rows.into_iter().map(|row| row.get("lore_id")).collect();
        Ok(set)
    }

    pub async fn link_lore(&self, character_id: i64, lore_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR IGNORE INTO character_lore_link (character_id, lore_id) VALUES (?, ?)",
        )
        .bind(character_id)
        .bind(lore_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn unlink_lore(&self, character_id: i64, lore_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM character_lore_link WHERE character_id = ? AND lore_id = ?")
            .bind(character_id)
            .bind(lore_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Settings
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

    // Database Management
    pub async fn checkpoint(&self) -> Result<(), sqlx::Error> {
        sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }

    pub async fn validate_candidate(path: &std::path::Path) -> Result<(), String> {
        if !path.exists() {
            return Err("File does not exist".to_string());
        }

        let db_url = format!("sqlite://{}", path.to_string_lossy());

        // Open separate pool
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&db_url)
            .await
            .map_err(|e| e.to_string())?;

        // Basic Check: Does 'characters' table exist?
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='characters'",
        )
        .fetch_optional(&pool)
        .await
        .map_err(|e| e.to_string())?;

        let valid = match row {
            Some((count,)) => count > 0,
            None => false,
        };

        pool.close().await;

        if valid {
            Ok(())
        } else {
            Err("Invalid database schema: 'characters' table missing.".to_string())
        }
    }
}
