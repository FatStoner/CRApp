use crate::models::Template;
use sqlx::SqlitePool;

pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Template>, sqlx::Error> {
    sqlx::query_as::<_, Template>("SELECT * FROM templates ORDER BY name ASC")
        .fetch_all(pool)
        .await
}

pub async fn upsert(pool: &SqlitePool, template: &mut Template) -> Result<(), sqlx::Error> {
    template.updated_at = chrono::Utc::now();
    if template.id == 0 {
        // INSERT
        let id = sqlx::query(
            "INSERT INTO templates (name, title, first_message, personality, scenario, example_dialogue, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&template.name)
        .bind(&template.title)
        .bind(&template.first_message)
        .bind(&template.personality)
        .bind(&template.scenario)
        .bind(&template.example_dialogue)
        .bind(template.created_at)
        .bind(template.updated_at)
        .execute(pool)
        .await?
        .last_insert_rowid();

        template.id = id;
    } else {
        // UPDATE
        sqlx::query(
            "UPDATE templates SET name=?, title=?, first_message=?, personality=?, scenario=?, example_dialogue=?, updated_at=? WHERE id=?"
        )
        .bind(&template.name)
        .bind(&template.title)
        .bind(&template.first_message)
        .bind(&template.personality)
        .bind(&template.scenario)
        .bind(&template.example_dialogue)
        .bind(template.updated_at)
        .bind(template.id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM templates WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
