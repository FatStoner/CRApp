use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(non_snake_case)]
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
    pub position: usize,
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
    pub enabled: bool,
    pub depth: usize,
    // Extensions and other fields observed in reference
    pub selectiveLogic: Option<usize>,
    pub addMemo: Option<bool>,
    pub displayIndex: Option<usize>,
    // "keys" is present in reference alongside "key"
    pub keys: Vec<String>,
    // "name" is present in reference
    pub name: String,
    // "id" is present in reference (seems duplicate of uid but u64?)
    pub id: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SillyTavernLorebook {
    pub name: Option<String>,
    pub description: Option<String>,
    pub scan_depth: Option<usize>,
    pub token_budget: Option<usize>,
    pub recursive_scanning: Option<bool>,
    pub extensions: Option<HashMap<String, serde_json::Value>>,
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
            position: 0,
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
            enabled: true,
            depth: 4,
            selectiveLogic: Some(0),
            addMemo: Some(true),
            displayIndex: Some(1),
            keys: vec![],
            name: "".to_string(),
            id: 0,
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
            id: idx, // reference has both
            key: keywords.clone(),
            keys: keywords, // reference has both
            comment: entry.name.clone(),
            name: entry.name.clone(),
            content: entry.content.clone(),
            ..Default::default()
        };

        entries.insert(idx.to_string(), st_entry);
    }

    SillyTavernLorebook {
        name: Some(lorebook.title.clone()),
        description: Some(lorebook.description.clone()),
        scan_depth: Some(4),
        token_budget: Some(2048),
        recursive_scanning: Some(false),
        extensions: Some(HashMap::new()),
        entries,
    }
}
