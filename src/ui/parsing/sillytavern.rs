use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct SillyTavernEntry {
    pub uid: usize,
    pub key: Vec<String>,
    pub keysecondary: Vec<String>,
    pub comment: String,
    pub content: String,
    pub constant: bool,
    pub selective: bool,
    pub folder: String,
    pub order: usize,
    pub position: String,
    pub use_regex: bool,
    pub exclude_recursion: bool,
    pub prevent_recursion: bool,
    pub delay_until_recursion: bool,
    pub match_whole_words: Option<bool>,
    pub match_any_word: Option<bool>,
    pub case_sensitive: Option<bool>,
    pub scan_depth: Option<usize>,
    pub probability: usize,
    pub disable: bool,
    pub depth: usize,
} // Defaults will be handled in conversion

#[derive(Debug, Serialize, Deserialize)]
pub struct SillyTavernLorebook {
    pub entries: HashMap<String, SillyTavernEntry>,
}

impl Default for SillyTavernEntry {
    fn default() -> Self {
        Self {
            uid: 0,
            key: vec![],
            keysecondary: vec![],
            comment: "".to_string(),
            content: "".to_string(),
            constant: false,
            selective: true,
            folder: "".to_string(),
            order: 100,
            position: "before_char".to_string(),
            use_regex: false,
            exclude_recursion: false,
            prevent_recursion: false,
            delay_until_recursion: false,
            match_whole_words: None,
            match_any_word: None,
            case_sensitive: None,
            scan_depth: None,
            probability: 100,
            disable: false,
            depth: 4,
        }
    }
}

pub fn convert_to_sillytavern(lorebook: &crate::models::Lorebook) -> SillyTavernLorebook {
    let mut entries = HashMap::new();

    for (idx, entry) in lorebook.entries.iter().enumerate() {
        let keywords: Vec<String> = entry
            .keywords
            .split(',')
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
            .collect();

        let st_entry = SillyTavernEntry {
            uid: idx,
            key: keywords,
            comment: entry.name.clone(),
            content: entry.content.clone(),
            ..Default::default()
        };

        entries.insert(idx.to_string(), st_entry);
    }

    SillyTavernLorebook { entries }
}
