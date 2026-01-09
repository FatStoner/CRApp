use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::error::Error;
use crate::models::{Character, Lorebook, Collection, Tag};

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
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN collection_id INTEGER REFERENCES collections(id) ON DELETE SET NULL").execute(&pool).await;

        // Create collections table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS collections (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                parent_id INTEGER REFERENCES collections(id) ON DELETE CASCADE
            )"
        )
        .execute(&pool)
        .await?;

        // Create tags (App)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            )",
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS character_tags (
                character_id INTEGER REFERENCES characters(id) ON DELETE CASCADE,
                tag_id INTEGER REFERENCES tags(id) ON DELETE CASCADE,
                PRIMARY KEY (character_id, tag_id)
            )",
        )
        .execute(&pool)
        .await?;

        // Create tags (External)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS external_tags (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            )",
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS character_external_tags (
                 character_id INTEGER REFERENCES characters(id) ON DELETE CASCADE,
                 tag_id INTEGER REFERENCES external_tags(id) ON DELETE CASCADE,
                 PRIMARY KEY (character_id, tag_id)
            )",
        )
        .execute(&pool)
        .await?;

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

        // Create settings table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
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
                "INSERT INTO characters (name, char_name, char_title, personality, scenario, example_dialogue, first_message, author_notes, avatar_path, created_at, updated_at, collection_id)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
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
            .execute(&self.pool)
            .await?
            .last_insert_rowid();

            character.id = id;
        } else {
            // UPDATE
            sqlx::query(
                "UPDATE characters SET name=?, char_name=?, char_title=?, personality=?, scenario=?, example_dialogue=?, first_message=?, author_notes=?, avatar_path=?, updated_at=?, collection_id=? WHERE id=?"
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
            .bind(character.id)
            .execute(&self.pool)
            .await?;
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

    pub async fn move_character(&self, char_id: i64, collection_id: Option<i64>) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE characters SET collection_id = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(collection_id)
            .bind(char_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Collections
    pub async fn get_all_collections(&self) -> Result<Vec<crate::models::Collection>, sqlx::Error> {
        sqlx::query_as::<_, crate::models::Collection>("SELECT * FROM collections")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn upsert_collection(&self, collection: &crate::models::Collection) -> Result<i64, sqlx::Error> {
        if collection.id == 0 {
             let id = sqlx::query("INSERT INTO collections (name, parent_id) VALUES (?, ?)")
                .bind(&collection.name)
                .bind(collection.parent_id)
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

    
    // Tag Helpers
    
    pub async fn get_tags_for_character(&self, char_id: i64, is_external: bool) -> Result<Vec<Tag>, sqlx::Error> {
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
    
    pub async fn add_tag_to_character(&self, char_id: i64, tag_name: &str, is_external: bool) -> Result<(), sqlx::Error> {
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
        let link_query = format!("INSERT OR IGNORE INTO {} (character_id, tag_id) VALUES (?, ?)", join_table);
        sqlx::query(&link_query)
            .bind(char_id)
            .bind(tag_id)
            .execute(&self.pool)
            .await?;
            
        Ok(())
    }
    
    pub async fn remove_tag_from_character(&self, char_id: i64, tag_id: i64, is_external: bool) -> Result<(), sqlx::Error> {
        let join_table = if is_external {
            "character_external_tags"
        } else {
            "character_tags"
        };
        
        let query = format!("DELETE FROM {} WHERE character_id = ? AND tag_id = ?", join_table);
        sqlx::query(&query)
            .bind(char_id)
            .bind(tag_id)
            .execute(&self.pool)
            .await?;
            
        Ok(())
    }

    // Returns (character_id, Tag) for all tags of a specific type (external or app)
    pub async fn get_all_tags_flat(&self, is_external: bool) -> Result<Vec<(i64, Tag)>, sqlx::Error> {
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
    
    // Deep Search
    pub async fn search_characters_text(&self, query: &str) -> Result<Vec<crate::models::Character>, sqlx::Error> {
        let pattern = format!("%{}%", query);
        // We search in all text fields
        sqlx::query_as::<_, crate::models::Character>(
            "SELECT * FROM characters WHERE 
             name LIKE ? OR 
             personality LIKE ? OR 
             scenario LIKE ? OR 
             example_dialogue LIKE ? OR 
             first_message LIKE ? OR 
             author_notes LIKE ?"
        )
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(&self.pool)
        .await
    }
    
    pub async fn search_tags_matching(&self, query: &str) -> Result<Vec<(i64, String, bool)>, sqlx::Error> {
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
        
        let rows = sqlx::query(q)
            .bind(&pattern)
            .bind(&pattern)
            .fetch_all(&self.pool)
            .await?;
            
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
    
    pub async fn get_characters_by_ids(&self, ids: &[i64]) -> Result<Vec<crate::models::Character>, sqlx::Error> {
        if ids.is_empty() { return Ok(Vec::new()); }
        
        // Dynamic IN clause
        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let query = format!("SELECT * FROM characters WHERE id IN ({})", placeholders.join(","));
        
        let mut q = sqlx::query_as::<_, crate::models::Character>(&query);
        for id in ids {
            q = q.bind(id);
        }
        
        q.fetch_all(&self.pool).await
    }
    
    pub async fn search_lorebooks_text(&self, query: &str) -> Result<Vec<crate::models::Lorebook>, sqlx::Error> {
        let pattern = format!("%{}%", query);
        sqlx::query_as::<_, crate::models::Lorebook>(
            "SELECT * FROM lorebooks WHERE 
             title LIKE ? OR 
             description LIKE ?"
        )
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(&self.pool)
        .await
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
}
