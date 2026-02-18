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
    pub collection_id: Option<i64>,     // For filtering by folder
    pub matches: Vec<(String, String)>, // (Field Name, Snippet)
    pub index: usize,                   // For restoring original order
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

impl std::fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeMode::System => write!(f, "System"),
            ThemeMode::Light => write!(f, "Light"),
            ThemeMode::Dark => write!(f, "Dark"),
        }
    }
}

impl std::str::FromStr for ThemeMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Light" => Ok(ThemeMode::Light),
            "Dark" => Ok(ThemeMode::Dark),
            _ => Ok(ThemeMode::System),
        }
    }
}

#[derive(Debug, Clone, FromRow, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct CharacterUrl {
    pub id: i64,
    pub character_id: i64,
    pub url: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, FromRow, PartialEq, Serialize, Deserialize)]
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
    #[sqlx(default)]
    pub quick_notes: String,
    #[sqlx(default)]
    pub is_favorite: bool,
    #[sqlx(default)]
    pub is_nsfw: bool,
    #[sqlx(default)]
    pub blur_avatar: bool,
    pub spell_check_overrides_json: Option<String>,
    #[sqlx(skip)]
    pub spell_check_overrides: std::collections::HashSet<String>,
    #[sqlx(skip)]
    pub app_tags: Vec<Tag>,
    #[sqlx(skip)]
    pub external_tags: Vec<Tag>,
    #[sqlx(skip)]
    pub urls: Vec<CharacterUrl>,
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
            quick_notes: "".to_string(),
            is_favorite: false,
            is_nsfw: false,
            blur_avatar: false,
            spell_check_overrides_json: None,
            spell_check_overrides: std::collections::HashSet::new(),
            app_tags: Vec::new(),
            external_tags: Vec::new(),
            urls: Vec::new(),
        }
    }
}

impl Character {
    pub fn content_eq(&self, other: &Character) -> bool {
        self.name == other.name
            && self.char_name == other.char_name
            && self.char_title == other.char_title
            && self.personality == other.personality
            && self.scenario == other.scenario
            && self.example_dialogue == other.example_dialogue
            && self.first_message == other.first_message
            && self.author_notes == other.author_notes
            && self.avatar_path == other.avatar_path
            && self.collection_id == other.collection_id
            && self.quick_notes == other.quick_notes
            && self.is_favorite == other.is_favorite
            && self.is_nsfw == other.is_nsfw
            && self.blur_avatar == other.blur_avatar
            && self.spell_check_overrides == other.spell_check_overrides
            && self
                .urls
                .iter()
                .filter(|u| !u.url.trim().is_empty())
                .eq(other.urls.iter().filter(|u| !u.url.trim().is_empty()))
    }

    pub fn post_load(&mut self) {
        if let Some(json) = &self.spell_check_overrides_json {
            if let Ok(set) = serde_json::from_str(json) {
                self.spell_check_overrides = set;
            }
        }
    }
}

pub fn count_tokens(text: &str) -> usize {
    use std::sync::OnceLock;
    use tiktoken_rs::CoreBPE;

    static BPE: OnceLock<CoreBPE> = OnceLock::new();

    let bpe = BPE
        .get_or_init(|| tiktoken_rs::cl100k_base().expect("Failed to load cl100k_base tokenizer"));

    bpe.encode_with_special_tokens(text).len()
}

#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct Lorebook {
    pub id: i64,
    pub title: String,
    pub description: String,
    #[sqlx(default)]
    pub content: String, // Added handling for content column
    pub cover_path: Option<String>,
    #[sqlx(skip)]
    pub tags: Vec<crate::models::Tag>,
    #[sqlx(skip)]
    pub entries: Vec<LorebookEntry>,
}

#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct Collection {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    #[sqlx(default)]
    pub display_order: i64,
    pub image_path: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow, PartialEq)]
pub struct LorebookEntry {
    pub id: i64,
    pub lorebook_id: i64,
    pub name: String,
    pub keywords: String,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Default for LorebookEntry {
    fn default() -> Self {
        Self {
            id: 0,
            lorebook_id: 0,
            name: "New Entry".to_string(),
            keywords: String::new(),
            content: String::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}

impl Default for Lorebook {
    fn default() -> Self {
        Self {
            id: 0,
            title: "New Lorebook".to_string(),
            description: "".to_string(),
            content: "".to_string(),
            cover_path: None,
            tags: Vec::new(),
            entries: Vec::new(),
        }
    }
}
impl Lorebook {
    pub fn content_eq(&self, other: &Lorebook) -> bool {
        self.title == other.title
            && self.description == other.description
            && self.content == other.content
            && self.cover_path == other.cover_path
            && self.tags == other.tags
            && self.entries == other.entries
    }
}

#[derive(Debug, Clone, FromRow, PartialEq, Serialize, Deserialize)]
pub struct Template {
    pub id: i64,
    pub name: String,
    pub title: String,
    pub first_message: String,
    pub personality: String,
    pub scenario: String,
    pub example_dialogue: String,
    #[sqlx(default)]
    pub created_at: DateTime<Utc>,
    #[sqlx(default)]
    pub updated_at: DateTime<Utc>,
}

impl Default for Template {
    fn default() -> Self {
        Self {
            id: 0,
            name: "New Template".to_string(),
            title: "".to_string(),
            first_message: "".to_string(),
            personality: "".to_string(),
            scenario: "".to_string(),
            example_dialogue: "".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl Template {
    pub fn content_eq(&self, other: &Template) -> bool {
        self.name == other.name
            && self.title == other.title
            && self.first_message == other.first_message
            && self.personality == other.personality
            && self.scenario == other.scenario
            && self.example_dialogue == other.example_dialogue
    }
}
