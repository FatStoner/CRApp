#[derive(Default, Clone)]
pub struct ParsedCharacterData {
    pub name: String,
    pub title: String,
    pub personality: String,
    pub scenario: String,
    pub first_message: String,
    pub example_dialogue: String,
    pub external_tags: Vec<String>,
}

pub fn parse_clipboard(text: &str) -> ParsedCharacterData {
    let mut data = ParsedCharacterData::default();

    // Pre-processing
    let lines: Vec<&str> = text
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    if lines.is_empty() {
        return data;
    }

    // -------------------------------------------------------------------------
    // 1. ANCHORS & METADATA
    // -------------------------------------------------------------------------
    let idx_back = lines.iter().position(|&l| l == "Back");
    let idx_share = lines
        .iter()
        .position(|&l| l == "Share")
        .or_else(|| lines.iter().position(|&l| l == "Favorite"));
    let idx_suggest_tag = lines
        .iter()
        .position(|&l| l.eq_ignore_ascii_case("suggest tag"));

    // Name extraction (Back -> avatar image -> Name)
    if let Some(idx) = idx_back {
        if let Some(offset) = lines
            .iter()
            .skip(idx)
            .take(3)
            .position(|&l| l == "avatar image")
        {
            let name_idx = idx + offset + 1;
            if name_idx < lines.len() {
                data.name = lines[name_idx].to_string();
            }
        }
    }

    // Title & Tags extraction (Share -> ... -> Suggest Tag)
    // We prioritize this block for tags to avoid reading section headers as tags or vice versa.
    let content_start_idx = if let Some(end) = idx_suggest_tag {
        if let Some(start) = idx_share {
            if end > start {
                let range = &lines[(start + 1)..end];
                let candidates: Vec<&str> = range
                    .iter()
                    .filter(|&&l| {
                        let s = l.to_lowercase();
                        !s.chars().all(|c| c.is_numeric() || c == ',' || c == '.') && // pure numbers
                    !s.contains('%') &&
                    !s.contains("tokens") &&
                    !s.contains("chat now") &&
                    s != "share" &&
                    s != "favorite"
                    })
                    .cloned()
                    .collect();

                if !candidates.is_empty() {
                    data.title = candidates[0].to_string();
                    for tag in candidates.iter().skip(1) {
                        data.external_tags.push(tag.to_string());
                    }
                }
            }
        }
        end + 1 // Start scanning content just after "Suggest Tag"
    } else {
        0 // Fallback: if no strict metadata block, scan whole file (less safe but necessary fallback)
    };

    // -------------------------------------------------------------------------
    // 2. CONTENT SECTIONS (Strict Scan)
    // -------------------------------------------------------------------------
    let mut current_section = "";

    for i in content_start_idx..lines.len() {
        let line = lines[i];
        let lower = line.to_lowercase();

        // Footer Stop
        if lower == "spicychat" || lower.starts_with("owned & operated by") {
            break;
        }

        // Headers
        if lower == "greeting" || lower == "first message" {
            current_section = "first_message";
            continue;
        }
        if lower == "personality" {
            current_section = "personality";
            continue;
        }
        if lower == "scenario" {
            current_section = "scenario";
            continue;
        }
        if lower == "example dialogues" || lower == "example dialogue" {
            current_section = "example_dialogue";
            continue;
        }
        if lower == "show less" {
            current_section = "ignore";
            continue;
        }

        // Key-Value checks (Only inside relevant sections or if valid)
        // Note: Name might be in Personality if we missed it earlier
        if current_section == "personality" && lower.starts_with("name:") && data.name.is_empty() {
            if let Some((_, val)) = line.split_once(':') {
                data.name = val.trim().to_string();
            }
        }

        match current_section {
            "first_message" => {
                data.first_message.push_str(line);
                data.first_message.push('\n');
            }
            "personality" => {
                data.personality.push_str(line);
                data.personality.push('\n');
            }
            "scenario" => {
                data.scenario.push_str(line);
                data.scenario.push('\n');
            }
            "example_dialogue" => {
                data.example_dialogue.push_str(line);
                data.example_dialogue.push('\n');
            }
            _ => {
                // Fallback catch for Name if purely unstructured and we are outside sections
                if i < 20
                    && data.name.is_empty()
                    && line.len() < 50
                    && !line.contains(':')
                    && !lower.starts_with('@')
                    && !lower.contains("tokens")
                {
                    // Only if we haven't found a name yet and we are early in the file
                    // data.name = line.to_string(); // Too risky with strict parsing?
                }
            }
        }
    }

    // Cleanup
    data.personality = data.personality.trim().to_string();
    data.scenario = data.scenario.trim().to_string();
    data.first_message = data.first_message.trim().to_string();
    data.example_dialogue = data.example_dialogue.trim().to_string();

    data
}
