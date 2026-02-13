use super::types::ParsedCharacterData;
use super::utils::find_next_value_index;

pub fn parse_afterhour_view(lines: &[&str]) -> ParsedCharacterData {
    let mut data = ParsedCharacterData::default();
    let mut current_section = "";

    // Buffers for multi-line sections
    let mut scenario_buffer = String::new();
    let mut model_instructions_buffer = String::new();
    let mut personality_buffer = String::new();
    let mut first_message_buffer = String::new();
    let mut title_buffer = String::new();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();

        if line.is_empty() {
            i += 1;
            continue;
        }

        // Section Detection
        match line {
            "Name" => {
                if let Some(next_idx) = find_next_value_index(lines, i) {
                    data.char_name = lines[next_idx].to_string();
                    data.name = lines[next_idx].to_string(); // Use Char Name as file name default
                    i = next_idx;
                }
                i += 1;
                continue;
            }
            "Title" => {
                current_section = "Title";
                i += 1;
                continue;
            }
            "Greeting" => {
                current_section = "Greeting";
                i += 1;
                continue;
            }
            "Personality" => {
                current_section = "Personality";
                i += 1;
                continue;
            }
            "Scenario" => {
                current_section = "Scenario";
                i += 1;
                continue;
            }
            "Model Instructions" => {
                current_section = "Model Instructions";
                i += 1;
                continue;
            }
            "Tags" => {
                current_section = "Tags";
                i += 1;
                continue;
            }
            "Visibility"
            | "Definition Visibility"
            | "Advanced Configuration"
            | "Lorebook"
            | "Create Character"
            | "Creator Notes" => {
                // Stop parsing current multi-line section if we hit these headers
                current_section = "";
                i += 1;
                continue;
            }
            _ => {}
        }

        // Content Extraction based on current section
        match current_section {
            "Title" => {
                if !title_buffer.is_empty() {
                    title_buffer.push('\n');
                }
                title_buffer.push_str(line);
            }
            "Greeting" => {
                // Ignore lines that are just numbers (token counts)
                if line.chars().all(|c| c.is_numeric()) {
                    i += 1;
                    continue;
                }

                if !first_message_buffer.is_empty() {
                    first_message_buffer.push('\n');
                }
                first_message_buffer.push_str(line);
            }
            "Personality" => {
                // Personality in example seems to be a token count line then the block?
                // Example:
                // 40: Personality
                // 41: name: Alexandra Jones ...
                // 42: 557
                // Actually the line 41 is the content.
                // We should append all lines until next section.
                // Also ignore numeric lines that look like token counts if they are alone?
                // In example: "557" is on its own line after the block.

                // Heuristic: if line is just digits, ignore it? Or maybe it's part of the text.
                // Let's just append everything for now, user can edit.
                // Wait, "401" was before Personality line in example 1.
                // "557" after.

                if line.chars().all(|c| c.is_numeric()) {
                    // likely token count
                } else {
                    if !personality_buffer.is_empty() {
                        personality_buffer.push('\n');
                    }
                    personality_buffer.push_str(line);
                }
            }
            "Scenario" => {
                if line.chars().all(|c| c.is_numeric()) {
                    // likely token count
                } else {
                    if !scenario_buffer.is_empty() {
                        scenario_buffer.push('\n');
                    }
                    scenario_buffer.push_str(line);
                }
            }
            "Model Instructions" => {
                if line.chars().all(|c| c.is_numeric()) {
                    // likely token count
                } else {
                    if !model_instructions_buffer.is_empty() {
                        model_instructions_buffer.push('\n');
                    }
                    model_instructions_buffer.push_str(line);
                }
            }
            "Tags" => {
                // Tags seem to be one per line?
                // Example:
                // 47: Tags
                // 48: Roommate
                // 49: Female
                // ...
                // 62:
                // 63: Private (part of Visibility?)
                // Actually "Visibility" is next section.
                data.external_tags.push(line.to_string());
            }
            _ => {}
        }

        i += 1;
    }

    data.title = title_buffer.trim().to_string();
    data.first_message = first_message_buffer.trim().to_string();
    data.personality = personality_buffer.trim().to_string();

    // Combine Scenario and Model Instructions
    let mut combined_scenario = String::new();
    if !scenario_buffer.trim().is_empty() {
        combined_scenario.push_str(scenario_buffer.trim());
    }
    if !model_instructions_buffer.trim().is_empty() {
        if !combined_scenario.is_empty() {
            combined_scenario.push_str("\n\n");
        }
        combined_scenario.push_str(model_instructions_buffer.trim());
    }
    data.scenario = combined_scenario;

    data.cleanup();
    data
}

#[cfg(test)]
#[path = "afterhour_tests.rs"]
mod tests;
