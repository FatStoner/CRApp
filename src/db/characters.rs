use crate::models::{Character, CharacterUrl};
use sqlx::SqlitePool;

pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Character>, sqlx::Error> {
    let mut list = sqlx::query_as::<_, Character>("SELECT * FROM characters")
        .fetch_all(pool)
        .await?;
    for c in &mut list {
        c.post_load();
    }
    Ok(list)
}

pub async fn upsert(pool: &SqlitePool, character: &mut Character) -> Result<(), sqlx::Error> {
    character.updated_at = chrono::Utc::now();
    character.spell_check_overrides_json = if character.spell_check_overrides.is_empty() {
        None
    } else {
        serde_json::to_string(&character.spell_check_overrides).ok()
    };

    if character.id == 0 {
        // INSERT
        let id = sqlx::query(
            "INSERT INTO characters (name, char_name, char_title, personality, scenario, example_dialogue, first_message, author_notes, avatar_path, created_at, updated_at, collection_id, is_favorite, spell_check_overrides_json, quick_notes)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
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
        .bind(&character.spell_check_overrides_json)
        .bind(&character.quick_notes)
        .execute(pool)
        .await?
        .last_insert_rowid();

        character.id = id;
    } else {
        // UPDATE
        sqlx::query(
            "UPDATE characters SET name=?, char_name=?, char_title=?, personality=?, scenario=?, example_dialogue=?, first_message=?, author_notes=?, avatar_path=?, updated_at=?, collection_id=?, is_favorite=?, spell_check_overrides_json=?, quick_notes=? WHERE id=?"
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
        .bind(&character.spell_check_overrides_json)
        .bind(&character.quick_notes)
        .bind(character.id)
        .execute(pool)
        .await?;
    }

    // Handle URLs
    if character.id != 0 {
        sqlx::query("DELETE FROM character_urls WHERE character_id = ?")
            .bind(character.id)
            .execute(pool)
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
            .execute(pool)
            .await?
            .last_insert_rowid();
            url.id = uid;
            url.character_id = character.id;
        }
    }

    Ok(())
}

pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM characters WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn move_to_collection(
    pool: &SqlitePool,
    char_id: i64,
    collection_id: Option<i64>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE characters SET collection_id = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
    )
    .bind(collection_id)
    .bind(char_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_all_urls_flat(pool: &SqlitePool) -> Result<Vec<CharacterUrl>, sqlx::Error> {
    sqlx::query_as::<_, CharacterUrl>("SELECT * FROM character_urls")
        .fetch_all(pool)
        .await
}

pub async fn search_text(pool: &SqlitePool, query: &str) -> Result<Vec<Character>, sqlx::Error> {
    let pattern = format!("%{}%", query);
    // We search in all text fields
    let mut list = sqlx::query_as::<_, Character>(
        "SELECT DISTINCT c.* FROM characters c
         LEFT JOIN character_urls u ON c.id = u.character_id
         WHERE 
         c.name LIKE ? OR 
         c.personality LIKE ? OR 
         c.scenario LIKE ? OR 
         c.char_title LIKE ? OR 
         c.example_dialogue LIKE ? OR 
         c.first_message LIKE ? OR 
         c.author_notes LIKE ? OR
         c.quick_notes LIKE ? OR
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
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(pool)
    .await?;
    for c in &mut list {
        c.post_load();
    }
    Ok(list)
}

pub async fn get_by_ids(pool: &SqlitePool, ids: &[i64]) -> Result<Vec<Character>, sqlx::Error> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    // Dynamic IN clause
    let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
    let query = format!(
        "SELECT * FROM characters WHERE id IN ({})",
        placeholders.join(",")
    );

    let mut q = sqlx::query_as::<_, Character>(&query);
    for id in ids {
        q = q.bind(id);
    }

    let mut list = q.fetch_all(pool).await?;
    for c in &mut list {
        c.post_load();
    }
    Ok(list)
}
