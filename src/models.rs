use chrono::{DateTime, Utc};
use sqlx::FromRow;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum SearchResultKind {
    Character,
    Lorebook,
}

#[derive(Debug, Clone)]
pub struct DeepSearchResult {
    pub id: i64,
    pub kind: SearchResultKind,
    pub display_name: String,
    pub matches: Vec<(String, String)>, // (Field Name, Snippet)
}

#[derive(Debug, Clone, FromRow, Default, PartialEq, Eq, Hash)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct Character {
    pub id: i64,
    pub name: String,
    pub char_name: String,
    pub char_title: String,
    pub personality: String,
    pub scenario: String,
    pub example_dialogue: String,
    pub first_message: String,
    pub author_notes: String,
    pub avatar_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub collection_id: Option<i64>,
    #[sqlx(skip)]
    pub app_tags: Vec<Tag>,
    #[sqlx(skip)]
    pub external_tags: Vec<Tag>,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            id: 0, // 0 indicates a new, unsaved character
            name: "New Character".to_string(),
            char_name: "".to_string(),
            char_title: "".to_string(),
            personality: "".to_string(),
            scenario: "".to_string(),
            example_dialogue: "".to_string(),
            first_message: "".to_string(),
            author_notes: "".to_string(),
            avatar_path: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            collection_id: None,
            app_tags: Vec::new(),
            external_tags: Vec::new(),
        }
    }
}

pub fn count_tokens(text: &str) -> usize {
    use std::sync::OnceLock;
    use tiktoken_rs::CoreBPE;

    static BPE: OnceLock<CoreBPE> = OnceLock::new();
    
    let bpe = BPE.get_or_init(|| {
        tiktoken_rs::cl100k_base().expect("Failed to load cl100k_base tokenizer")
    });
    
    bpe.encode_with_special_tokens(text).len()
}

#[derive(Debug, Clone, FromRow)]
pub struct Lorebook {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub cover_path: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Collection {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
}

impl Default for Lorebook {
    fn default() -> Self {
        Self {
            id: 0,
            title: "New Lorebook".to_string(),
            description: "".to_string(),
            cover_path: None,
        }
    }
}
