use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Character {
    pub id: i64,
    pub name: String,
    pub char_name: String,
    pub char_title: String,
    pub personality: String,
    pub first_message: String,
    pub author_notes: String,
    pub avatar_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
