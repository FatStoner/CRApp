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

// SillyTavern V3 export card — built from ST-specific fields only
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SillyTavernCard {
    pub name: String,
    pub description: String,
    pub personality: String,
    pub scenario: String,
    pub first_mes: String,
    pub mes_example: String,
    pub creatorcomment: String,
    pub avatar: String,
    pub talkativeness: String,
    pub fav: bool,
    pub tags: Vec<String>,
    pub spec: String,
    pub spec_version: String,
    pub data: SillyTavernCardData,
    pub create_date: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SillyTavernCardData {
    pub name: String,
    pub description: String,
    pub personality: String,
    pub scenario: String,
    pub first_mes: String,
    pub mes_example: String,
    pub creator_notes: String,
    pub system_prompt: String,
    pub post_history_instructions: String,
    pub tags: Vec<String>,
    pub creator: String,
    pub character_version: String,
    pub alternate_greetings: Vec<String>,
    pub extensions: SillyTavernExtensions,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SillyTavernExtensions {
    pub talkativeness: String,
    pub fav: bool,
    pub world: String,
    pub depth_prompt: SillyTavernDepthPrompt,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SillyTavernDepthPrompt {
    pub prompt: String,
    pub depth: i64,
    pub role: String,
}

impl SillyTavernCard {
    pub fn from_character(character: &crate::models::Character) -> Self {
        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let talkativeness_str = format!("{:.1}", character.st_talkativeness);
        let card_name = if character.st_name.trim().is_empty() {
            character.name.clone()
        } else {
            character.st_name.clone()
        };
        let tags: Vec<String> = character
            .app_tags
            .iter()
            .chain(character.external_tags.iter())
            .map(|t| t.name.clone())
            .collect();

        SillyTavernCard {
            name: card_name.clone(),
            description: character.st_description.clone(),
            personality: character.st_personality.clone(),
            scenario: character.st_scenario.clone(),
            first_mes: character.st_first_mes.clone(),
            mes_example: character.st_mes_example.clone(),
            creatorcomment: character.st_creator_notes.clone(),
            avatar: "none".to_string(),
            talkativeness: talkativeness_str.clone(),
            fav: false,
            tags: tags.clone(),
            spec: "chara_card_v3".to_string(),
            spec_version: "3.0".to_string(),
            data: SillyTavernCardData {
                name: card_name,
                description: character.st_description.clone(),
                personality: character.st_personality.clone(),
                scenario: character.st_scenario.clone(),
                first_mes: character.st_first_mes.clone(),
                mes_example: character.st_mes_example.clone(),
                creator_notes: character.st_creator_notes.clone(),
                system_prompt: "".to_string(),
                post_history_instructions: "".to_string(),
                tags,
                creator: character.st_creator.clone(),
                character_version: character.st_character_version.clone(),
                alternate_greetings: character.st_alternate_greetings.clone(),
                extensions: SillyTavernExtensions {
                    talkativeness: talkativeness_str,
                    fav: false,
                    world: character.st_world.clone(),
                    depth_prompt: SillyTavernDepthPrompt {
                        prompt: character.st_depth_prompt.clone(),
                        depth: character.st_depth_prompt_depth,
                        role: character.st_depth_prompt_role.clone(),
                    },
                },
            },
            create_date: now,
        }
    }
}
