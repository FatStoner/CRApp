use crate::models::Tag;
use sqlx::SqlitePool;

pub async fn get_for_character(
    pool: &SqlitePool,
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
        .fetch_all(pool)
        .await
}

pub async fn add_to_character(
    pool: &SqlitePool,
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
    let insert_tag_query = format!("INSERT OR IGNORE INTO {} (name) VALUES (?)", tag_table);
    sqlx::query(&insert_tag_query)
        .bind(tag_name)
        .execute(pool)
        .await?;

    // 2. Get Tag ID
    let get_id_query = format!("SELECT id FROM {} WHERE name = ?", tag_table);
    let tag_id: i64 = sqlx::query_scalar(&get_id_query)
        .bind(tag_name)
        .fetch_one(pool)
        .await?;

    // 3. Link
    let link_query = format!(
        "INSERT OR IGNORE INTO {} (character_id, tag_id) VALUES (?, ?)",
        join_table
    );
    sqlx::query(&link_query)
        .bind(char_id)
        .bind(tag_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn remove_from_character(
    pool: &SqlitePool,
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
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn remove_all_from_character(
    pool: &SqlitePool,
    char_id: i64,
    is_external: bool,
) -> Result<(), sqlx::Error> {
    let join_table = if is_external {
        "character_external_tags"
    } else {
        "character_tags"
    };

    let query = format!("DELETE FROM {} WHERE character_id = ?", join_table);
    sqlx::query(&query).bind(char_id).execute(pool).await?;

    Ok(())
}

// Returns (character_id, Tag) for all tags of a specific type
pub async fn get_all_flat(
    pool: &SqlitePool,
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

    let rows = sqlx::query(&query).fetch_all(pool).await?;

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

pub async fn search_matching(
    pool: &SqlitePool,
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

    let rows = sqlx::query(q)
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(pool)
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
