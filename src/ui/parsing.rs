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

#[derive(Default, Clone, Debug)]
pub struct ParsedLorebookEntry {
    pub name: String,
    pub keywords: Vec<String>,
    pub content: String,
}

#[derive(Default, Clone, Debug)]
pub struct ParsedLorebookData {
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub entries: Vec<ParsedLorebookEntry>,
}

enum ImportFormat {
    Profile,
    Edit,
    LorebookHtml,
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
        // For LorebookHtml, we need a separate entry point or return type.
        // Since parse_clipboard returns ParsedCharacterData, we might need a separate function for Lorebooks.
        // See parse_spicychat_lorebook below.
        _ => {
            // Fallback to Profile parser as it's more generic/loose
            parse_profile_view(&lines)
        }
    }
}

pub fn parse_spicychat_lorebook(html: &str) -> ParsedLorebookData {
    // Dispatch based on content markers
    if html.contains("text-mobile-heading-3") {
        parse_spicychat_lorebook_profile_view(html)
    } else {
        parse_spicychat_lorebook_edit_view(html)
    }
}

fn extract_spicychat_text(full_html: &str, start_marker: &str, end_marker: &str) -> Option<String> {
    if let Some(start_idx) = full_html.find(start_marker) {
        // Find the end of the opening tag from the marker
        if let Some(tag_end) = full_html[start_idx..].find('>') {
            let content_start = start_idx + tag_end + 1;
            if let Some(end_idx) = full_html[content_start..].find(end_marker) {
                return Some(
                    full_html[content_start..content_start + end_idx]
                        .trim()
                        .to_string(),
                );
            }
        }
    }
    None
}

fn parse_spicychat_lorebook_entries(html: &str) -> Vec<ParsedLorebookEntry> {
    let mut entries = Vec::new();
    let entry_marker = "hover:bg-gray-4";
    let mut current_pos = 0;

    while let Some(marker_offset) = html[current_pos..].find(entry_marker) {
        let entry_start = current_pos + marker_offset;
        current_pos = entry_start + entry_marker.len(); // Advance past this marker

        // Limit search scope to reasonable length to avoid finding next entry's data
        // 5000 chars should be enough for one entry?
        let search_limit = std::cmp::min(current_pos + 5000, html.len());
        let entry_region = &html[current_pos..search_limit];

        // Check if we actually have entry data in this region (it might be some other button with same class)
        // Entry Name: text-gray-12 line-clamp-2
        // Keywords: text-gray-11 line-clamp-1
        // Content: -webkit-line-clamp: 2

        let mut entry = ParsedLorebookEntry::default();
        let mut found_any = false;

        // Optimization: Quick check if this region is actually an entry
        // Entries usually have "line-clamp-2" for name or content. Navigation buttons do NOT.
        if !entry_region.contains("line-clamp-2") {
            continue;
        }

        if let Some(name) = extract_spicychat_text(entry_region, "text-gray-12", "</p>") {
            entry.name = name;
            found_any = true;
        }

        if let Some(kws) = extract_spicychat_text(entry_region, "text-gray-11 line-clamp-1", "</p>")
        {
            entry.keywords = kws.split(',').map(|s| s.trim().to_string()).collect();
        }

        // Content sometimes has style attribute with line-clamp
        if let Some(_content_idx) = entry_region.find("-webkit-line-clamp: 2") {
            // We need to find the closes closing tag after this style declaration?
            // Actually extract_text helper searches for '>' after marker.
            // The marker is inside the style attribute.
            // <p ... style="... -webkit-line-clamp: 2 ...">CONTENT</p>
            if let Some(content) =
                extract_spicychat_text(entry_region, "-webkit-line-clamp: 2", "</p>")
            {
                entry.content = content;
            }
        }

        if found_any && !entry.name.is_empty() {
            entries.push(entry);
        }
    }
    entries
}

fn parse_spicychat_lorebook_edit_view(html: &str) -> ParsedLorebookData {
    let mut data = ParsedLorebookData::default();

    // 1. Extract Title
    // Marker: "Edit Lorebook" -> Look for next text-label-lg
    if let Some(edit_idx) = html.find("Edit Lorebook") {
        let search_region = &html[edit_idx..];
        // The title usually has "text-label-lg" or "mt-sm"
        // Let's look for the class "text-label-lg" which is unique to the title in that area
        if let Some(title) = extract_spicychat_text(search_region, "text-label-lg", "</p>") {
            data.title = title;
        } else if let Some(title) = extract_spicychat_text(search_region, "mt-sm", "</p>") {
            data.title = title;
        }
    }

    // 2. Extract Entries
    data.entries = parse_spicychat_lorebook_entries(html);

    data
}

fn parse_spicychat_lorebook_profile_view(html: &str) -> ParsedLorebookData {
    let mut data = ParsedLorebookData::default();

    // 1. Extract Title
    // Marker: text-mobile-heading-3
    let mut title_end_idx = 0;
    if let Some(start_idx) = html.find("text-mobile-heading-3") {
        if let Some(title) =
            extract_spicychat_text(&html[start_idx..], "text-mobile-heading-3", "</p>")
        {
            data.title = title;
            title_end_idx = start_idx;
        }
    }

    // 2. Extract Description
    // Search *after* the title to avoid picking up navbar elements.
    let search_scope = if title_end_idx > 0 {
        &html[title_end_idx..]
    } else {
        html
    };
    let mut current_pos = 0;

    while let Some(idx) = search_scope[current_pos..].find("text-label-lg") {
        let start = current_pos + idx;
        current_pos = start + "text-label-lg".len(); // advance

        if let Some(content) =
            extract_spicychat_text(&search_scope[start..], "text-label-lg", "</p>")
        {
            let is_author = content.starts_with('@');
            let is_title = content == data.title;
            // List of known nav items to exclude if description matches them
            // (though scoping after title should prevent most of these)
            let is_nav_item = [
                "Home",
                "Chats",
                "My Personas",
                "Create",
                "Chatbot",
                "Lorebook",
                "Group",
                "My Creations",
                "Chatbots",
                "Groups",
                "Favorites",
                "Recommendations",
                "Leaderboard",
                "Blocked Creators",
                "Subscribe",
                "Help",
                "Sign Out",
                "Back",
                "Share",
                "History",
            ]
            .contains(&content.as_str());

            if !is_author && !is_title && !is_nav_item {
                data.description = content;
                break;
            }
        }
    }

    // 3. Extract Entries
    // Find "Entries" header to safely skip navbar and other buttons
    let entries_start_idx = if let Some(idx) = html.find("text-heading-5") {
        idx
    } else {
        0
    };

    // Safety check: ensure entries start after title if title exists
    let safe_start = std::cmp::max(entries_start_idx, title_end_idx);

    data.entries = parse_spicychat_lorebook_entries(&html[safe_start..]);

    data
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
    #[test]
    fn test_parse_spicychat_lorebook() {
        let html = r#"
            <div class="w-full mx-auto max-w-[750px] bg-gray-1 dark:bg-gray-3 rounded-lg p-3 md:p-4 border border-solid border-gray-5">
                <div class="flex justify-between max-mob:flex-col ">
                    <div class="flex items-center gap-sm">
                        <p class="font-sans text-decoration-skip-ink-none text-underline-position-from-font text-heading-5 font-bold text-left flex gap-sm items-center">Edit Lorebook</p>
                    </div>
                </div>
                <p class="font-sans text-decoration-skip-ink-none text-underline-position-from-font text-label-lg font-medium text-left text-gray-11 mt-sm">Test Lorebook #01</p>
                <div class="py-xl flex flex-col gap-xl pb-0 pt-lg">
                    <div class="flex flex-col">
                        <button type="button" class="w-full flex items-center justify-between rounded-lg cursor-pointer transition-colors duration-200 bg-transparent border border-solid border-transparent gap-2 py-md px-[13px] min-h-auto hover:bg-gray-4">
                            <div class="flex items-center gap-md flex-1">
                                <div class="flex flex-col min-w-0 flex-1 gap-0.5">
                                    <p class="font-sans text-decoration-skip-ink-none text-underline-position-from-font text-label-md font-regular text-left text-gray-12 line-clamp-2">Example entry 1</p>
                                    <div class="flex items-center gap-1">
                                        <p class="font-sans text-decoration-skip-ink-none text-underline-position-from-font text-label-sm font-regular text-left text-gray-11 line-clamp-1">keyword_example1</p>
                                    </div>
                                    <p class="font-sans text-decoration-skip-ink-none text-underline-position-from-font text-label-sm font-regular text-left" style="display: -webkit-box; -webkit-box-orient: vertical; overflow: hidden; -webkit-line-clamp: 2; text-overflow: ellipsis;">Lorem ipsum dolor sit amet</p>
                                </div>
                            </div>
                        </button>
                        <button type="button" class="w-full flex items-center justify-between rounded-lg cursor-pointer transition-colors duration-200 bg-transparent border border-solid border-transparent gap-2 py-md px-[13px] min-h-auto hover:bg-gray-4">
                            <div class="flex items-center gap-md flex-1">
                                <div class="flex flex-col min-w-0 flex-1 gap-0.5">
                                    <p class="font-sans text-decoration-skip-ink-none text-underline-position-from-font text-label-md font-regular text-left text-gray-12 line-clamp-2">Example entry 2</p>
                                    <div class="flex items-center gap-1">
                                        <p class="font-sans text-decoration-skip-ink-none text-underline-position-from-font text-label-sm font-regular text-left text-gray-11 line-clamp-1">2example_keyword</p>
                                    </div>
                                    <p class="font-sans text-decoration-skip-ink-none text-underline-position-from-font text-label-sm font-regular text-left" style="display: -webkit-box; -webkit-box-orient: vertical; overflow: hidden; -webkit-line-clamp: 2; text-overflow: ellipsis;">Second entry content</p>
                                </div>
                            </div>
                        </button>
                    </div>
                </div>
            </div>
            "#;

        let parsed = parse_spicychat_lorebook(html);

        assert_eq!(parsed.title, "Test Lorebook #01");
        assert_eq!(parsed.entries.len(), 2);

        let entry1 = &parsed.entries[0];
        assert_eq!(entry1.name, "Example entry 1");
        assert_eq!(entry1.keywords, vec!["keyword_example1"]);
        assert_eq!(entry1.content, "Lorem ipsum dolor sit amet");

        let entry2 = &parsed.entries[1];
        assert_eq!(entry2.name, "Example entry 2");
        assert_eq!(entry2.keywords, vec!["2example_keyword"]);
        assert_eq!(entry2.content, "Second entry content");
    }

    #[test]
    fn test_parse_spicychat_lorebook_profile_view() {
        let html = r#"
            <div class="some-container">
                <p class="text-mobile-heading-3 font-bold">Public Lorebook Title</p>
                <div class="author-section">
                    <p class="text-label-lg">@AuthorName</p>
                </div>
                <div class="desc-section">
                    <p class="text-label-lg">This is a description of the lorebook.</p>
                </div>
                
                <!-- Navbar element that mimics entry button but lacks line-clamp-2 -->
                <button class="hover:bg-gray-4 flex items-center">
                    <div class="flex items-center gap-2">
                         <p class="text-label-lg">Home</p>
                    </div>
                </button>

                <div class="w-full">
                    <p class="text-heading-5">Entries</p>
                    <div class="entries-list">
                        <button class="hover:bg-gray-4 flex items-center">
                            <div class="entry-content">
                                <p class="text-gray-12 text-label-md line-clamp-2">Profile Entry 1</p>
                                <p class="text-gray-11 line-clamp-1">profile_kw1, profile_kw2</p>
                                <p class="style-clamp" style="-webkit-line-clamp: 2">Profile Content 1</p>
                            </div>
                        </button>
                    </div>
                </div>
            </div>
        "#;

        let parsed = parse_spicychat_lorebook(html);

        assert_eq!(parsed.title, "Public Lorebook Title");
        assert_eq!(parsed.description, "This is a description of the lorebook.");

        // Should only find 1 entry, ignoring the Navbar element
        assert_eq!(parsed.entries.len(), 1);
        assert_eq!(parsed.entries[0].name, "Profile Entry 1");
        assert_eq!(
            parsed.entries[0].keywords,
            vec!["profile_kw1", "profile_kw2"]
        );
        assert_eq!(parsed.entries[0].content, "Profile Content 1");
    }
}
