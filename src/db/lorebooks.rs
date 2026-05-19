use crate::error::DbError;
use crate::models::{Lorebook, LorebookEntry, Tag};
use sqlx::SqlitePool;
use std::collections::HashSet;

pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Lorebook>, DbError> {
    let list = sqlx::query_as::<_, Lorebook>("SELECT * FROM lorebooks")
        .fetch_all(pool)
        .await?;
    Ok(list)
}

pub async fn get_by_ids(pool: &SqlitePool, ids: &[i64]) -> Result<Vec<Lorebook>, DbError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
    let query = format!(
        "SELECT * FROM lorebooks WHERE id IN ({})",
        placeholders.join(",")
    );
    let mut q = sqlx::query_as::<_, Lorebook>(&query);
    for id in ids {
        q = q.bind(id);
    }
    let list = q.fetch_all(pool).await?;
    Ok(list)
}

pub async fn search_text(pool: &SqlitePool, query: &str) -> Result<Vec<Lorebook>, DbError> {
    let pattern = format!("%{}%", query);
    let list = sqlx::query_as::<_, Lorebook>(
        "SELECT * FROM lorebooks WHERE 
         title LIKE ? OR 
         description LIKE ? OR
         content LIKE ?",
    )
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(pool)
    .await?;
    Ok(list)
}

pub async fn upsert(pool: &SqlitePool, lorebook: &mut Lorebook) -> Result<(), DbError> {
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
        .execute(pool)
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
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), DbError> {
    // 1. Delete Entries
    sqlx::query("DELETE FROM lorebook_entries WHERE lorebook_id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    // 2. Delete Tags Link
    sqlx::query("DELETE FROM lorebook_tags WHERE lorebook_id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    // 3. Delete Character Links
    sqlx::query("DELETE FROM character_lore_link WHERE lore_id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    // 4. Delete Lorebook
    sqlx::query("DELETE FROM lorebooks WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

// --- Entries ---

pub async fn get_entries(
    pool: &SqlitePool,
    lorebook_id: i64,
) -> Result<Vec<LorebookEntry>, DbError> {
    let list = sqlx::query_as::<_, LorebookEntry>(
        "SELECT * FROM lorebook_entries WHERE lorebook_id = ? ORDER BY name ASC",
    )
    .bind(lorebook_id)
    .fetch_all(pool)
    .await?;
    Ok(list)
}

pub async fn add_entry(pool: &SqlitePool, entry: &LorebookEntry) -> Result<i64, DbError> {
    let id = sqlx::query("INSERT INTO lorebook_entries (lorebook_id, name, keywords, content, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(entry.lorebook_id)
        .bind(&entry.name)
        .bind(&entry.keywords)
        .bind(&entry.content)
        .bind(entry.created_at)
        .bind(entry.updated_at)
        .execute(pool)
        .await?
        .last_insert_rowid();
    Ok(id)
}

pub async fn update_entry(pool: &SqlitePool, entry: &LorebookEntry) -> Result<(), DbError> {
    sqlx::query(
        "UPDATE lorebook_entries SET name=?, keywords=?, content=?, updated_at=? WHERE id=?",
    )
    .bind(&entry.name)
    .bind(&entry.keywords)
    .bind(&entry.content)
    .bind(entry.updated_at)
    .bind(entry.id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_entry(pool: &SqlitePool, id: i64) -> Result<(), DbError> {
    sqlx::query("DELETE FROM lorebook_entries WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn search_entries_text(
    pool: &SqlitePool,
    query: &str,
) -> Result<Vec<LorebookEntry>, DbError> {
    let pattern = format!("%{}%", query);
    let list = sqlx::query_as::<_, LorebookEntry>(
        "SELECT * FROM lorebook_entries WHERE 
         name LIKE ? OR 
         keywords LIKE ? OR 
         content LIKE ?",
    )
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(pool)
    .await?;
    Ok(list)
}

// --- Tags ---

pub async fn add_tag(
    pool: &SqlitePool,
    lorebook_id: i64,
    tag_name: &str,
) -> Result<(), DbError> {
    // 1. Ensure tag exists
    let insert_tag_query = "INSERT OR IGNORE INTO tags (name) VALUES (?)";
    sqlx::query(insert_tag_query)
        .bind(tag_name)
        .execute(pool)
        .await?;

    // 2. Get Tag ID
    let get_id_query = "SELECT id FROM tags WHERE name = ?";
    let tag_id: i64 = sqlx::query_scalar(get_id_query)
        .bind(tag_name)
        .fetch_one(pool)
        .await?;

    // 3. Link
    let link_query = "INSERT OR IGNORE INTO lorebook_tags (lorebook_id, tag_id) VALUES (?, ?)";
    sqlx::query(link_query)
        .bind(lorebook_id)
        .bind(tag_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn remove_tag(
    pool: &SqlitePool,
    lorebook_id: i64,
    tag_id: i64,
) -> Result<(), DbError> {
    sqlx::query("DELETE FROM lorebook_tags WHERE lorebook_id = ? AND tag_id = ?")
        .bind(lorebook_id)
        .bind(tag_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_tags(pool: &SqlitePool, lorebook_id: i64) -> Result<Vec<Tag>, DbError> {
    let query = "
        SELECT t.id, t.name FROM tags t
        JOIN lorebook_tags lt ON t.id = lt.tag_id
        WHERE lt.lorebook_id = ?
    ";
    let list = sqlx::query_as::<_, Tag>(query)
        .bind(lorebook_id)
        .fetch_all(pool)
        .await?;
    Ok(list)
}

pub async fn get_all_tags_flat(pool: &SqlitePool) -> Result<Vec<(i64, Tag)>, DbError> {
    let query = "
        SELECT lt.lorebook_id, t.id, t.name FROM tags t
        JOIN lorebook_tags lt ON t.id = lt.tag_id
    ";
    let rows = sqlx::query(query).fetch_all(pool).await?;

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

pub async fn search_tags_matching(
    pool: &SqlitePool,
    query: &str,
) -> Result<Vec<(i64, String)>, DbError> {
    let pattern = format!("%{}%", query);
    let q = "
        SELECT lt.lorebook_id, t.name
        FROM lorebook_tags lt 
        JOIN tags t ON lt.tag_id = t.id 
        WHERE t.name LIKE ?
    ";
    let rows = sqlx::query(q).bind(&pattern).fetch_all(pool).await?;

    use sqlx::Row;
    let mut results = Vec::new();
    for row in rows {
        let lid: i64 = row.get(0);
        let name: String = row.get(1);
        results.push((lid, name));
    }
    Ok(results)
}

// --- Lore Links ---

pub async fn get_links(pool: &SqlitePool, character_id: i64) -> Result<HashSet<i64>, DbError> {
    let rows = sqlx::query("SELECT lore_id FROM character_lore_link WHERE character_id = ?")
        .bind(character_id)
        .fetch_all(pool)
        .await?;

    use sqlx::Row;
    let set: HashSet<i64> = rows.into_iter().map(|row| row.get("lore_id")).collect();
    Ok(set)
}

pub async fn link(pool: &SqlitePool, character_id: i64, lore_id: i64) -> Result<(), DbError> {
    sqlx::query("INSERT OR IGNORE INTO character_lore_link (character_id, lore_id) VALUES (?, ?)")
        .bind(character_id)
        .bind(lore_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn unlink(pool: &SqlitePool, character_id: i64, lore_id: i64) -> Result<(), DbError> {
    sqlx::query("DELETE FROM character_lore_link WHERE character_id = ? AND lore_id = ?")
        .bind(character_id)
        .bind(lore_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_all_links_flat(pool: &SqlitePool) -> Result<Vec<(i64, i64)>, DbError> {
    let rows = sqlx::query("SELECT character_id, lore_id FROM character_lore_link")
        .fetch_all(pool)
        .await?;

    use sqlx::Row;
    let mut results = Vec::new();
    for row in rows {
        results.push((row.get(0), row.get(1)));
    }
    Ok(results)
}
