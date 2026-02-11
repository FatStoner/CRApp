use super::{find_next_value_index, ParsedCharacterData};

pub fn parse_ggpt_view(lines: &[&str]) -> ParsedCharacterData {
    let mut data = ParsedCharacterData::default();
    let mut current_section = "";

    for (i, &line) in lines.iter().enumerate() {
        let lower = line.to_lowercase();
        let trimmed = line.trim();

        // One-line fields
        if lower == "character name" {
            if let Some(val_idx) = find_next_value_index(lines, i) {
                data.name = lines[val_idx].to_string();
            }
            continue;
        }

        if lower == "character age" {
            // Not currently stored in ParsedCharacterData, skipping
            continue;
        }

        if lower.starts_with("description") && lower.contains("tokens") {
            // Mapping Description -> Title as requested
            current_section = "title";
            continue;
        }

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

        if lower.starts_with("example conversation") {
            current_section = "example_dialogue";
            continue;
        }

        if lower == "character tags" {
            current_section = "tags";
            continue;
        }

        // Stop conditions
        if lower == "write a brief overview of your character."
            || lower == "describe your character's traits, behavior, and demeanor."
            || lower == "legacy"
            || lower.contains("visibility")
        {
            current_section = "";
            continue;
        }

        match current_section {
            "title" => {
                if !trimmed.is_empty() {
                    data.title.push_str(line);
                    data.title.push('\n');
                }
            }
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
                // The example sometimes has a warning "⚠️ Can cause unpredictable behavior..."
                if !line.contains("⚠️") {
                    data.example_dialogue.push_str(line);
                    data.example_dialogue.push('\n');
                }
            }
            "tags" => {
                if lower == "add tag" || lower.is_empty() {
                    continue;
                }
                // Stop if we hit descriptions beneath tags
                if lower.starts_with("assign tags that describes") {
                    current_section = "";
                    continue;
                }

                // Tags seem to be on their own lines
                if !trimmed.is_empty() {
                    data.external_tags.push(trimmed.to_string());
                }
            }
            _ => {}
        }
    }

    data.cleanup();
    data
}
