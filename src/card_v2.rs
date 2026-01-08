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
