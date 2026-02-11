use super::ParsedCharacterData;
use crate::card_v2::TavernCardV2;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

pub fn parse_v2_card(json: &str) -> Result<ParsedCharacterData, String> {
    // 1. Try TavernCardV2 (Nested 'data' field)
    if let Ok(card) = serde_json::from_str::<TavernCardV2>(json) {
        return Ok(from_v2(card));
    }

    // 2. Try CharacterCardV2 (Flat structure, used by 'spicychat.ai (.json)' export)
    if let Ok(card) = serde_json::from_str::<crate::card_v2::CharacterCardV2>(json) {
        return Ok(from_char_v2(card));
    }

    // 3. Detailed Error
    // If both failed, we return the error from the primary format (TavernCardV2) usually,
    // or a generic "Unknown format".
    // Let's get the specific error for TavernCardV2 to help debug.
    let err = serde_json::from_str::<TavernCardV2>(json).unwrap_err();
    Err(format!("JSON Parse Error: {}", err))
}

pub fn parse_png_card(bytes: &[u8]) -> Result<ParsedCharacterData, String> {
    // 1. Decode PNG to find chunks
    let decoder = png::Decoder::new(bytes);
    let reader = match decoder.read_info() {
        Ok(r) => r,
        Err(e) => return Err(format!("PNG Decode Error: {}", e)),
    };
    let info = reader.info();

    // 2. Look for tEXt chunk with keyword "chara"
    for chunk in &info.uncompressed_latin1_text {
        if chunk.keyword == "chara" {
            // 3. Decode Base64
            let json_bytes = match BASE64.decode(&chunk.text) {
                Ok(b) => b,
                Err(e) => return Err(format!("Base64 Decode Error in 'chara' chunk: {}", e)),
            };

            let json_str = match String::from_utf8(json_bytes) {
                Ok(s) => s,
                Err(e) => return Err(format!("UTF-8 Error in 'chara' chunk: {}", e)),
            };

            // 4. Parse JSON
            return parse_v2_card(&json_str);
        }
    }

    // Debug: List found chunks if any
    let found_keywords: Vec<&str> = info
        .uncompressed_latin1_text
        .iter()
        .map(|c| c.keyword.as_str())
        .collect();
    Err(format!(
        "No 'chara' metadata found in PNG. Found chunks: {:?}",
        found_keywords
    ))
}

fn from_v2(card: TavernCardV2) -> ParsedCharacterData {
    ParsedCharacterData {
        name: card.data.name,
        title: card.data.personality,
        personality: card.data.description,
        scenario: card.data.scenario,
        first_message: card.data.first_mes,
        example_dialogue: card.data.mes_example,
        external_tags: card.data.tags,
        app_tags: Vec::new(),
        urls: Vec::new(),
        avatar_path: None,
    }
}

fn from_char_v2(card: crate::card_v2::CharacterCardV2) -> ParsedCharacterData {
    ParsedCharacterData {
        name: card.name,
        title: card.personality,
        personality: card.description,
        scenario: card.scenario,
        first_message: card.first_mes,
        example_dialogue: card.mes_example,
        external_tags: Vec::new(),
        // Checking struct... CharacterCardV2 in card_v2.rs has `metadata: CardMetadata`,
        // which has `source`, `tool`. No tags field in flat struct.
        app_tags: Vec::new(),
        urls: Vec::new(),
        avatar_path: None,
    }
}
