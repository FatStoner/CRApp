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

impl Default for Character {
    fn default() -> Self {
        Self {
            id: 0, // 0 indicates a new, unsaved character
            name: "New Character".to_string(),
            char_name: "".to_string(),
            char_title: "".to_string(),
            personality: "".to_string(),
            first_message: "".to_string(),
            author_notes: "".to_string(),
            avatar_path: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
