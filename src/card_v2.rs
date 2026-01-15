use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CharacterCardV2 {
    pub name: String,
    pub description: String,
    pub personality: String,
    pub scenario: String,
    pub first_mes: String,
    pub mes_example: String,
    pub metadata: CardMetadata,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CardMetadata {
    pub version: u32,
    pub created: u64,
    pub modified: u64,
    pub source: Option<String>,
    pub tool: Option<String>,
}

impl CharacterCardV2 {
    pub fn new(
        name: String,
        description: String,
        personality: String,
        scenario: String,
        first_mes: String,
        mes_example: String,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            name,
            description,
            personality,
            scenario,
            first_mes,
            mes_example,
            metadata: CardMetadata {
                version: 1,
                created: now,
                modified: now,
                source: None,
                tool: Some("CRAP (Character Repository App)".to_string()),
            },
        }
    }
}

// TavernAI V2 Spec compliant structure for PNG Metadata ONLY
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TavernCardV2 {
    pub spec: String,
    pub spec_version: String,
    pub data: CharacterCardData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CharacterCardData {
    pub name: String,
    pub description: String,
    pub personality: String,
    pub scenario: String,
    pub first_mes: String,
    pub mes_example: String,
    pub creator_notes: String,
    pub system_prompt: String,
    pub post_history_instructions: String,
    pub alternate_greetings: Vec<String>,
    pub character_book: Option<serde_json::Value>,
    pub tags: Vec<String>,
    pub creator: String,
    pub character_version: String,
    pub extensions: serde_json::Map<String, serde_json::Value>,
}

impl TavernCardV2 {
    pub fn new(
        name: String,
        description: String,
        personality: String,
        scenario: String,
        first_mes: String,
        mes_example: String,
    ) -> Self {
        Self {
            spec: "chara_card_v2".to_string(),
            spec_version: "2.0".to_string(),
            data: CharacterCardData {
                name,
                description,
                personality,
                scenario,
                first_mes,
                mes_example,
                creator_notes: "".to_string(),
                system_prompt: "".to_string(),
                post_history_instructions: "".to_string(),
                alternate_greetings: Vec::new(),
                character_book: None,
                tags: Vec::new(),
                creator: "".to_string(),
                character_version: "".to_string(),
                extensions: serde_json::Map::new(),
            },
        }
    }
}
