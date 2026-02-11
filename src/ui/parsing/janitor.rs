use super::{find_next_value_index, ParsedCharacterData};

pub fn parse_janitor_edit(lines: &[&str]) -> ParsedCharacterData {
    let mut data = ParsedCharacterData::default();
    let mut current_section = "";

    for (i, &line) in lines.iter().enumerate() {
        let lower = line.to_lowercase();
        let trimmed = line.trim();

        if lower.starts_with("character name") {
            if let Some(val_idx) = find_next_value_index(lines, i) {
                data.name = lines[val_idx].trim().to_string();
            }
            continue;
        }

        // Title Mapping: "Character Bio" -> "Paragraph"
        if lower.starts_with("character bio") || lower == "paragraph" {
            current_section = "title";
            continue;
        }

        if lower.starts_with("personality") && lower.contains("*") {
            current_section = "personality";
            continue;
        }

        if lower.starts_with("scenario") {
            current_section = "scenario";
            continue;
        }

        if lower.starts_with("initial messages") || lower.starts_with("first message") {
            current_section = "first_message";
            continue;
        }

        if lower.starts_with("example dialogs") {
            current_section = "example_dialogue";
            continue;
        }

        if lower.starts_with("character tags") {
            current_section = "";
            continue;
        }

        // Stop conditions
        if lower.starts_with("character settings")
            || lower.starts_with("publish settings")
            || lower.starts_with("content rating")
            || lower.starts_with("definition visibility")
            || lower.contains("tokens")
        {
            if lower.contains("tokens") {
                // Often "595 tokens" appears after a section, stopping it.
                continue;
            }
            current_section = "";
        }

        match current_section {
            "title" => {
                if !trimmed.is_empty()
                    && lower != "paragraph"
                    && !lower.starts_with("this will be displayed")
                {
                    data.title.push_str(line);
                    data.title.push('\n');
                }
            }
            "personality" => {
                // Skip the line that just says "Personality*"
                if !trimmed.is_empty() && !lower.starts_with("personality") {
                    data.personality.push_str(line);
                    data.personality.push('\n');
                }
            }
            "scenario" => {
                if !trimmed.is_empty() && !lower.starts_with("scenario") {
                    data.scenario.push_str(line);
                    data.scenario.push('\n');
                }
            }
            "first_message" => {
                // Skip UI text
                if lower.contains("provide a lengthy first message")
                    || lower.contains("click the + icon")
                    || lower.contains("use the up/down arrow")
                {
                    continue;
                }
                if !trimmed.is_empty() {
                    data.first_message.push_str(line);
                    data.first_message.push('\n');
                }
            }
            "example_dialogue" => {
                if lower.contains("provide example conversations")
                    || lower.contains("best practise example")
                {
                    continue;
                }
                if !trimmed.is_empty() {
                    data.example_dialogue.push_str(line);
                    data.example_dialogue.push('\n');
                }
            }
            _ => {}
        }
    }

    data.cleanup();
    data
}

pub fn parse_janitor_profile(lines: &[&str]) -> ParsedCharacterData {
    let mut data = ParsedCharacterData::default();
    let mut current_section = "";

    // 1. Basic Metadata Extraction (Name, Title)
    // Name is usually line 10 or 11 in the provided examples, after "beta".
    // Or we can search for the line before "0" (cnt) or "by:".

    // Attempt to find "by:"
    if let Some(by_idx) = lines.iter().position(|l| l.trim().starts_with("by:")) {
        // Name is usually a few lines above "by:"
        // Example:
        // 10: Silent Storm
        // 11: Silent Storm
        // 12: 0
        // 13:
        // 14: 0
        // 15:
        // 16: by:

        // Let's try to grab the line 6 lines above by: ?
        // Or just scan from top for non-empty lines after "janitor" "beta"

        // Generic approach: Scan lines until we hit "by:", take the first non-empty "heading-like" lines as name?
        // In "janitor_profile_page.md":
        // 7: Analytics
        // 8: beta
        // 10: Saira

        // In "janitor_profile_page2.md":
        // 7: Analytics
        // 7: Analytics
        // 8: beta
        // 10: Silent Storm

        // So scanning after "Analytics" -> "beta" seems robust.
        if let Some(analytics_idx) = lines.iter().position(|l| l.trim() == "Analytics") {
            for i in (analytics_idx + 1)..by_idx {
                let t = lines[i].trim();
                if !t.is_empty()
                    && t != "beta"
                    && t != "janitor"
                    && !t.chars().all(char::is_numeric)
                {
                    data.name = t.to_string();
                    break;
                }
            }
        }
    }

    // Title / Description
    // Often starts with a tag block like [Tag1, Tag2] or just text.
    // It ends before "Created " or "Updated "

    let start_desc_idx = lines
        .iter()
        .position(|l| l.trim().starts_with("by:"))
        .map(|i| i + 2)
        .unwrap_or(0);
    let end_desc_idx = lines
        .iter()
        .position(|l| l.trim().starts_with("Created "))
        .unwrap_or(lines.len());

    if start_desc_idx < end_desc_idx {
        for i in start_desc_idx..end_desc_idx {
            let line = lines[i].trim();
            if line.is_empty() || line.starts_with("@") {
                continue;
            } // Skip author handle

            data.title.push_str(lines[i]);
            data.title.push('\n');
        }
    }

    // 2. Sections
    for line in lines {
        let lower = line.to_lowercase();
        let _trimmed = line.trim();

        if lower.starts_with("personality") && lower.contains("tokens") {
            current_section = "personality";
            continue;
        }
        if lower.starts_with("scenario") && lower.contains("tokens") {
            current_section = "scenario";
            continue;
        }
        if lower.starts_with("first message") && lower.contains("tokens") {
            current_section = "first_message";
            continue;
        }
        if lower.starts_with("example dialogs") {
            current_section = "example_dialogue";
            continue;
        }

        if lower == "0" || lower == "comments" {
            current_section = "";
            continue;
        }

        match current_section {
            "personality" => {
                data.personality.push_str(line);
                data.personality.push('\n');
            }
            "scenario" => {
                data.scenario.push_str(line);
                data.scenario.push('\n');
            }
            "first_message" => {
                data.first_message.push_str(line);
                data.first_message.push('\n');
            }
            "example_dialogue" => {
                if !lower.contains("login to view") {
                    data.example_dialogue.push_str(line);
                    data.example_dialogue.push('\n');
                }
            }
            _ => {}
        }
    }

    // Extract Tags from Title if present?
    // Janitor profile tags look like [Tag1, Tag2, Tag3] at the start of description often.
    if let Some(start_bracket) = data.title.find('[') {
        if let Some(end_bracket) = data.title.find(']') {
            if end_bracket > start_bracket {
                let tags_str = &data.title[start_bracket + 1..end_bracket];
                for t in tags_str.split(',') {
                    data.external_tags.push(t.trim().to_string());
                }
                // Optional: Remove tags from title? User might want to keep them.
                // Let's keep them for now as it's part of the "Bio/Description".
            }
        }
    }

    data.cleanup();
    data
}
