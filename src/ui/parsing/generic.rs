use super::{find_next_value_index, ParsedCharacterData};

pub fn parse_edit_view(lines: &[&str]) -> ParsedCharacterData {
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
                data.char_name = data.name.clone();
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

pub fn parse_profile_view(lines: &[&str]) -> ParsedCharacterData {
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
                data.char_name = data.name.clone();
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
                    let (first_idx, _first_val) = candidates_with_indices[0];
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
                data.char_name = data.name.clone();
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
