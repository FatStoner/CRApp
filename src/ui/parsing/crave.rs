use super::{find_next_value_index, ParsedCharacterData};

pub fn parse_crave_edit_view(lines: &[&str]) -> ParsedCharacterData {
    let mut data = ParsedCharacterData::default();
    let mut current_section = "";

    for (i, &line) in lines.iter().enumerate() {
        let lower = line.to_lowercase();

        if lower == "character name*" {
            if let Some(val_idx) = find_next_value_index(lines, i) {
                data.name = lines[val_idx].to_string();
            }
            continue;
        }

        if lower == "introduction*" {
            current_section = "title";
            continue;
        }

        if lower == "personality*" {
            current_section = "personality";
            continue;
        }

        if lower == "tags*" {
            current_section = "tags";
            continue;
        }

        if lower == "initial message (greeting)*" {
            current_section = "greeting";
            continue;
        }

        if lower == "scenario" {
            current_section = "scenario";
            continue;
        }

        // Stop keywords or next section triggers
        if lower == "type in tags..." {
            current_section = "";
            continue;
        }

        if lower.contains("tokens") {
            // Check if it's a standalone token line.
            if line.chars().all(|c| {
                c.is_numeric()
                    || c.is_whitespace()
                    || c == 't'
                    || c == 'o'
                    || c == 'k'
                    || c == 'e'
                    || c == 'n'
                    || c == 's'
            }) {
                current_section = "";
                continue;
            }
        }

        match current_section {
            "title" => {
                if lower == "import html code" {
                    continue;
                }
                if lower.contains("markdown and html code supported!") {
                    current_section = "";
                    continue;
                }
                data.title.push_str(line);
                data.title.push('\n');
            }
            "personality" => {
                data.personality.push_str(line);
                data.personality.push('\n');
            }
            "greeting" => {
                data.first_message.push_str(line);
                data.first_message.push('\n');
            }
            "scenario" => {
                if lower.contains("outline the context and setting") {
                    current_section = "";
                    continue;
                }
                data.scenario.push_str(line);
                data.scenario.push('\n');
            }
            "tags" => {
                if !line.is_empty() && lower != "type in tags..." && lower != "tags*" {
                    // Check if it is a single word or comma separated
                    if line.contains(',') {
                        for t in line.split(',') {
                            let trimmed = t.trim();
                            if !trimmed.is_empty() {
                                data.external_tags.push(trimmed.to_string());
                            }
                        }
                    } else {
                        data.external_tags.push(line.to_string());
                    }
                }
            }
            _ => {}
        }
    }

    data.cleanup();
    data
}
