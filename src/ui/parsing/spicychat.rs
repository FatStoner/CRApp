use super::{ParsedLorebookData, ParsedLorebookEntry};

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
