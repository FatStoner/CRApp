#[derive(Default, Clone, Debug)]
pub struct ParsedCharacterData {
    pub name: String,
    pub title: String,
    pub personality: String,
    pub scenario: String,
    pub first_message: String,
    pub example_dialogue: String,
    pub external_tags: Vec<String>,
    pub app_tags: Vec<String>,
    pub urls: Vec<crate::models::CharacterUrl>,
}

enum ImportFormat {
    Profile,
    Edit,
    Unknown,
}

pub fn parse_clipboard(text: &str) -> ParsedCharacterData {
    let lines: Vec<&str> = text
        .lines()
        .map(|l| l.trim())
        // .filter(|l| !l.is_empty()) // Don't filter empty lines, we want to preserve them for multiline content
        .collect();

    if lines.is_empty() {
        return ParsedCharacterData::default();
    }

    match detect_format(&lines) {
        ImportFormat::Edit => parse_edit_view(&lines),
        ImportFormat::Profile => parse_profile_view(&lines),
        ImportFormat::Unknown => {
            // Fallback to Profile parser as it's more generic/loose
            parse_profile_view(&lines)
        }
    }
}

fn detect_format(lines: &[&str]) -> ImportFormat {
    // Edit view specific characteristic
    if lines
        .iter()
        .any(|l| *l == "Edit Chatbot" || *l == "Review Chatbot")
    {
        return ImportFormat::Edit;
    }

    // Also check for "Name" -> "*" -> <Value> pattern if "Edit Chatbot" header is missing (partial copy)
    // or checks for the token counter footer style "___/___ characters | ___ tokens"
    if lines
        .iter()
        .any(|l| l.contains("characters |") && l.contains("tokens"))
    {
        return ImportFormat::Edit;
    }

    // Profile view specific characteristic
    if lines
        .iter()
        .any(|l| *l == "avatar image" || *l == "Suggest Tag")
    {
        return ImportFormat::Profile;
    }

    ImportFormat::Unknown
}

fn parse_edit_view(lines: &[&str]) -> ParsedCharacterData {
    let mut data = ParsedCharacterData::default();
    let iter = lines.iter().enumerate();
    let mut current_section = "";

    for (i, &line) in iter {
        let lower = line.to_lowercase();

        // 1. Single Line Fields
        if lower == "name" {
            // Next line might be "*", then Value
            if let Some(val_idx) = find_next_value_index(lines, i) {
                data.name = lines[val_idx].to_string();
            }
            continue;
        }
        if lower == "title" {
            if let Some(val_idx) = find_next_value_index(lines, i) {
                data.title = lines[val_idx].to_string();
            }
            continue;
        }

        // 2. Multiline Sections
        // Starts with header, optionally "*", then content
        // Ends with token counter or next section

        if lower == "tags" {
            current_section = "tags";
            continue;
        }

        // --- Header Checks ---
        // If we are already in 'tags', avoid switching back to standard sections
        // merely because a tag happens to match a header keyword (e.g. "scenario").
        if current_section != "tags" {
            if lower == "greeting" {
                current_section = "greeting";
                continue;
            }
            if lower == "chatbot's personality" || lower == "personality" {
                current_section = "personality";
                continue;
            }
            if lower == "scenario" {
                current_section = "scenario";
                continue;
            }
            if lower == "example dialogue" {
                current_section = "example_dialogue";
                continue;
            }
        }

        // Stop Keywords
        if lower.starts_with("tokens:") || lower.contains("characters |") {
            current_section = "";
            continue;
        }
        if line == "*" {
            continue;
        }

        // Accumulate Content
        match current_section {
            "greeting" => {
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
            "tags" => {
                if lower.is_empty() {
                    continue;
                }
                // Stop if we hit the helper text
                if lower.contains("choose tags to help people discover your bot")
                    || lower == "advanced"
                    || lower.chars().all(|c| c.is_numeric() || c == '/')
                // e.g. "1/12"
                {
                    current_section = "";
                    continue;
                }
                data.external_tags.push(line.to_string());
            }
            _ => {}
        }
    }

    // Edit View Tags (often under "Tags" -> "Add Tags" -> ...)
    // But in the sample, tags don't appear clearly listed as values, just "0/12".
    // If they are listed, they might be just floating text.
    // For now, Edit View tags might be hard to parse unless we see a populated sample.
    // The provided sample has "0/12" implying no tags.
    // Leaving tags empty for Edit View for now unless identified.

    data.cleanup();
    data
}

fn find_next_value_index(lines: &[&str], current_index: usize) -> Option<usize> {
    for i in (current_index + 1)..lines.len() {
        if !lines[i].trim().is_empty() && lines[i] != "*" {
            return Some(i);
        }
    }
    None
}

fn parse_profile_view(lines: &[&str]) -> ParsedCharacterData {
    let mut data = ParsedCharacterData::default();

    // 1. ANCHORS & METADATA
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
            .take(10) // Was 3, increased to 10 to account for potential empty lines
            .position(|&l| l == "avatar image")
        {
            let name_idx = idx + offset + 1;
            // Find first non-empty line starting from name_idx
            if let Some(name_val) = lines.iter().skip(name_idx).find(|l| !l.trim().is_empty()) {
                data.name = name_val.to_string();
            }
        }
    }

    // Title & Tags extraction
    let content_start_idx = if let Some(end) = idx_suggest_tag {
        if let Some(start) = idx_share {
            if end > start {
                let range = &lines[(start + 1)..end];
                let candidates_with_indices: Vec<(usize, &str)> = range
                    .iter()
                    .enumerate()
                    .map(|(i, l)| (start + 1 + i, *l))
                    .filter(|(_, l)| {
                        let s = l.to_lowercase();

                        let is_numeric_ish = |c: char| c.is_numeric() || c == ',' || c == '.';
                        let is_usage_count = if let Some(last) = s.chars().last() {
                            if matches!(last, 'k' | 'm' | 'b') {
                                s[..s.len() - 1].chars().all(is_numeric_ish)
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        !s.chars().all(is_numeric_ish) && // pure numbers
                        !is_usage_count &&
                    !s.contains('%') &&
                    !s.contains("tokens") &&
                    !s.contains("chat now") &&
                    !s.is_empty() && // skip empty lines for candidate check
                    s != "share" &&
                    s != "favorite"
                    })
                    .collect();

                if !candidates_with_indices.is_empty() {
                    // Check for Lorebook pattern:
                    // If we have at least 2 candidates, checks the distance between 0 and 1.
                    // If distance is 2 (1 empty line gap), assume 0 is Lorebook and 1 is Title.
                    let (first_idx, first_val) = candidates_with_indices[0];
                    let start_index = if candidates_with_indices.len() >= 2 {
                        let (second_idx, _) = candidates_with_indices[1];
                        if second_idx - first_idx == 2 {
                            // Lorebook Detected! Skip the first one.
                            1
                        } else {
                            0
                        }
                    } else {
                        0
                    };

                    if start_index < candidates_with_indices.len() {
                        data.title = candidates_with_indices[start_index].1.to_string();
                        for (_, tag) in candidates_with_indices.iter().skip(start_index + 1) {
                            data.external_tags.push(tag.to_string());
                        }
                    }
                }
            }
        }
        end + 1
    } else {
        0
    };

    // 2. CONTENT SECTIONS
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

        // Catch Name if missed
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
            _ => {}
        }
    }

    data.cleanup();
    data
}

impl ParsedCharacterData {
    fn cleanup(&mut self) {
        self.name = self.name.trim().to_string();
        self.title = self.title.trim().to_string();
        self.personality = self.personality.trim().to_string();
        self.scenario = self.scenario.trim().to_string();
        self.first_message = self.first_message.trim().to_string();
        self.example_dialogue = self.example_dialogue.trim().to_string();

        // 1. Remove Advice Lines
        let advice_greeting = "What will they say to start a conversation.";
        let advice_personality = "In a few sentences, describe your chatbot's personality.";
        let advice_scenario = "Describe the current situation and context of the conversation";

        if self.first_message.ends_with(advice_greeting) {
            self.first_message = self
                .first_message
                .trim_end_matches(advice_greeting)
                .trim()
                .to_string();
        }
        if self.personality.ends_with(advice_personality) {
            self.personality = self
                .personality
                .trim_end_matches(advice_personality)
                .trim()
                .to_string();
        }
        if self.scenario.ends_with(advice_scenario) {
            self.scenario = self
                .scenario
                .trim_end_matches(advice_scenario)
                .trim()
                .to_string();
        }

        // 2. Remove Placeholders
        let placeholder_scenario = "Elara Nightshade stands in the center of a dimly lit room, a map of ancient ruins spread out before her. The faint glow from a nearby lantern reflects off the silver streaks in her dark hair as her piercing amber eyes scan the details, her enigmatic presence commanding the air of mystery surrounding the secrets she’s about to uncover.";
        let placeholder_dialogue = "{{User}}: Hey, what are you doing?\n{{Char}}: Greetings {{User}}! I am maintaining SpicyChat’s characters. Pleasure to meet you!\nExample conversations to define your Chatbot. This will impact how it talks.";

        if self.scenario == placeholder_scenario {
            self.scenario = String::new();
        }
        if self.example_dialogue.replace("\r\n", "\n").trim()
            == placeholder_dialogue.replace("\r\n", "\n").trim()
        {
            self.example_dialogue = String::new();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spicy_cleanup_advice_lines() {
        let mut data = ParsedCharacterData {
            first_message: "Hello user!\nWhat will they say to start a conversation.".to_string(),
            personality: "Kind bot.\nIn a few sentences, describe your chatbot's personality."
                .to_string(),
            scenario: "In a park.\nDescribe the current situation and context of the conversation"
                .to_string(),
            ..Default::default()
        };
        data.cleanup();
        assert_eq!(data.first_message, "Hello user!");
        assert_eq!(data.personality, "Kind bot.");
        assert_eq!(data.scenario, "In a park.");
    }

    #[test]
    fn test_spicy_cleanup_placeholders() {
        let mut data = ParsedCharacterData {
            scenario: "Elara Nightshade stands in the center of a dimly lit room, a map of ancient ruins spread out before her. The faint glow from a nearby lantern reflects off the silver streaks in her dark hair as her piercing amber eyes scan the details, her enigmatic presence commanding the air of mystery surrounding the secrets she’s about to uncover.".to_string(),
            example_dialogue: "{{User}}: Hey, what are you doing?\n{{Char}}: Greetings {{User}}! I am maintaining SpicyChat’s characters. Pleasure to meet you!\nExample conversations to define your Chatbot. This will impact how it talks.".to_string(),
            ..Default::default()
        };
        data.cleanup();
        assert_eq!(data.scenario, "");
        assert_eq!(data.example_dialogue, "");
    }

    #[test]
    fn test_multiline_preservation() {
        let raw_text = "Edit Chatbot\nName\nTestBot\nPersonality\nPara 1.\n\nPara 2.\ntokens: 100";
        let data = parse_clipboard(raw_text);
        assert_eq!(data.name, "TestBot");
        // We expect Para 1.\n\nPara 2.\n to be captured.
        // Note: The logic appends \n after each line.
        // So Para 1. -> Para 1.\n
        // Empty line -> \n
        // Para 2. -> Para 2.\n
        // Result: Para 1.\n\nPara 2.\n
        assert!(data.personality.contains("Para 1.\n\nPara 2."));
    }

    #[test]
    fn test_edit_view_with_empty_lines() {
        let raw_text = "Edit Chatbot\nName\n\nTestBot\nTitle\n\nThe Title\nPersonality\nDesc";
        let data = parse_clipboard(raw_text);
        assert_eq!(data.name, "TestBot");
        assert_eq!(data.title, "The Title");
    }

    #[test]
    fn test_profile_view_loose_structure_with_empty_name_gap() {
        let raw_text = "
        Back
        
        avatar image
        
        MyName
        Suggest Tag
        ";
        let data = parse_clipboard(raw_text);
        assert_eq!(data.name, "MyName");
    }

    #[test]
    fn test_profile_with_lorebook_spacing() {
        // Lorebook Name -> (1 empty line) -> Title -> (2 empty lines) -> Tag
        // Indices in the range:
        // Lorebook: i
        // (empty): i+1
        // Title: i+2 (delta = 2) -> SKIP Lorebook
        let raw_text = "
        Back
        
        avatar image
        BotName
        
        Share
        
        100 tokens
        
        MyLorebook
        
        MyTitle
        
        
        MyTag
        Suggest Tag
        ";
        let data = parse_clipboard(raw_text);
        assert_eq!(data.name, "BotName");
        assert_eq!(data.title, "MyTitle");
        assert_eq!(data.external_tags, vec!["MyTag"]);
    }

    #[test]
    fn test_profile_without_lorebook_spacing() {
        // Title -> (2 empty lines) -> Tag
        // Indices:
        // Title: i
        // (empty): i+1
        // (empty): i+2
        // Tag: i+3 (delta = 3) -> No Lorebook skip
        let raw_text = "
        Back
        
        avatar image
        BotName
        
        Share
        
        100 tokens
        
        MyTitle
        
        
        MyTag
        Suggest Tag
        ";
        let data = parse_clipboard(raw_text);
        assert_eq!(data.name, "BotName");
        assert_eq!(data.title, "MyTitle");
        assert_eq!(data.external_tags, vec!["MyTag"]);
    }

    #[test]
    fn test_edit_view_tags_with_empty_lines() {
        // "1/12" is usually the stop condition.
        // If we have empty lines between "Tags" and the tags, or between tags, it shouldn't reset.
        let raw_text = "
        Edit Chatbot
        Name
        Bot
        Tags
        
        Tag1
        
        Tag2
        1/12
        ";
        let data = parse_clipboard(raw_text);
        assert_eq!(data.name, "Bot");
        assert_eq!(data.external_tags, vec!["Tag1", "Tag2"]);
    }
}
