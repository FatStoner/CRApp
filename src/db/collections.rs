use crate::models::Collection;
use sqlx::SqlitePool;

pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Collection>, sqlx::Error> {
    sqlx::query_as::<_, Collection>("SELECT * FROM collections ORDER BY display_order ASC")
        .fetch_all(pool)
        .await
}

pub async fn upsert(pool: &SqlitePool, collection: &Collection) -> Result<i64, sqlx::Error> {
    if collection.id == 0 {
        // New Collection: Determine display_order (Max in siblings + 1)
        let max_order: Option<i64> = if let Some(pid) = collection.parent_id {
            sqlx::query_scalar("SELECT MAX(display_order) FROM collections WHERE parent_id = ?")
                .bind(pid)
                .fetch_optional(pool)
                .await?
        } else {
            sqlx::query_scalar("SELECT MAX(display_order) FROM collections WHERE parent_id IS NULL")
                .fetch_optional(pool)
                .await?
        };

        // If no siblings, start huge or start at id? Better start at current MAX + 1.
        // If table empty, 0.
        let next_order = max_order.unwrap_or(0) + 1;

        let id = sqlx::query(
            "INSERT INTO collections (name, parent_id, display_order, image_path) VALUES (?, ?, ?, ?)",
        )
        .bind(&collection.name)
        .bind(collection.parent_id)
        .bind(next_order)
        .bind(&collection.image_path)
        .execute(pool)
        .await?
        .last_insert_rowid();
        Ok(id)
    } else {
        sqlx::query("UPDATE collections SET name=?, parent_id=?, image_path=? WHERE id=?")
            .bind(&collection.name)
            .bind(collection.parent_id)
            .bind(&collection.image_path)
            .bind(collection.id)
            .execute(pool)
            .await?;
        Ok(collection.id)
    }
}

pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    // Orphan children
    sqlx::query("UPDATE collections SET parent_id = NULL WHERE parent_id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    sqlx::query("UPDATE characters SET collection_id = NULL WHERE collection_id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    // Delete
    sqlx::query("DELETE FROM collections WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn reorder(pool: &SqlitePool, id: i64, move_up: bool) -> Result<(), sqlx::Error> {
    // 1. Get current item info
    let current: Collection = sqlx::query_as("SELECT * FROM collections WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;

    // 2. Find swap target
    let op = if move_up { "<" } else { ">" };
    let sort = if move_up { "DESC" } else { "ASC" };

    // Handle parent_id null checks
    let query = if let Some(_pid) = current.parent_id {
        format!(
            "SELECT * FROM collections WHERE parent_id = ? AND display_order {} ? ORDER BY display_order {} LIMIT 1",
            op, sort
        )
    } else {
        format!(
            "SELECT * FROM collections WHERE parent_id IS NULL AND display_order {} ? ORDER BY display_order {} LIMIT 1",
            op, sort
        )
    };

    let mut q = sqlx::query_as::<_, Collection>(&query);
    if let Some(pid) = current.parent_id {
        q = q.bind(pid);
    }
    q = q.bind(current.display_order);

    let target = q.fetch_optional(pool).await?;

    if let Some(other) = target {
        // Swap orders
        // Use transaction for safety
        let mut tx = pool.begin().await?;

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
