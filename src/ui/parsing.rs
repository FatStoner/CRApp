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
        .filter(|l| !l.is_empty())
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
        if lines[i] != "*" {
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
            .take(3)
            .position(|&l| l == "avatar image")
        {
            let name_idx = idx + offset + 1;
            if name_idx < lines.len() {
                data.name = lines[name_idx].to_string();
            }
        }
    }

    // Title & Tags extraction
    let content_start_idx = if let Some(end) = idx_suggest_tag {
        if let Some(start) = idx_share {
            if end > start {
                let range = &lines[(start + 1)..end];
                let candidates: Vec<&str> = range
                    .iter()
                    .filter(|&&l| {
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
    }
}

// ----------------------------------------------------------------------------
// TESTS
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format_edit() {
        let text = r#"
Spicychat
...
Edit Chatbot
...
Name
*
Silent Storm
"#;
        let lines: Vec<&str> = text.lines().map(|l| l.trim()).collect();
        match detect_format(&lines) {
            ImportFormat::Edit => {}
            _ => panic!("Failed to detect Edit format"),
        }
    }

    #[test]
    fn test_detect_format_profile() {
        let text = r#"
Spicychat
...
Back
avatar image
SomeName
...
Suggest Tag
"#;
        let lines: Vec<&str> = text.lines().map(|l| l.trim()).collect();
        match detect_format(&lines) {
            ImportFormat::Profile => {}
            _ => panic!("Failed to detect Profile format"),
        }
    }

    #[test]
    fn test_parse_edit_tags() {
        let text = r#"
Edit Chatbot
Tags


Female


Choose tags to help people discover your bot
1/12
"#;
        let data = parse_clipboard(text);
        assert_eq!(data.external_tags, vec!["Female"]);
    }

    #[test]
    fn test_parse_edit_tags_edge_case() {
        let text = r#"
Edit Chatbot
Scenario
Some scenario content.
Example Dialogue
Some dialogue.
Tags


scenario


Choose tags to help people discover your bot
1/12
"#;
        let data = parse_clipboard(text);
        assert_eq!(data.scenario.trim(), "Some scenario content.");
        assert_eq!(data.external_tags, vec!["scenario"]);
    }

    #[test]
    fn test_parse_profile_high_usage() {
        let text = r#"
Back
avatar image
Eve
@fatstoner
Favorite
Share
9.6k
0%
1,607 tokens
Months after meeting a bunyip...
Futanari
NSFW
Suggest Tag
"#;
        let data = parse_clipboard(text);
        assert_eq!(data.name, "Eve");
        // "9.6k" should be ignored, so title should be the next line or empty/handled.
        // In this snippet, "Months after meeting..." acts as title based on position if "9.6k" is ignored.
        // If "9.6k" is NOT ignored, it becomes the title.
        assert_ne!(data.title, "9.6k", "Failed to ignore high usage count 9.6k");
        assert!(
            data.title.starts_with("Months after"),
            "Title detection failed: got '{}'",
            data.title
        );
        assert!(data.external_tags.contains(&"Futanari".to_string()));
    }
}
