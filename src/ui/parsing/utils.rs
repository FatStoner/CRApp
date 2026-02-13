use super::detection::detect_format;
use super::types::{ImportFormat, ParsedCharacterData, ParsedLorebookData};
use super::{afterhour, crave, generic, girlfriendgpt, janitor};

pub fn parse_crappbook_json(json: &str) -> Result<ParsedLorebookData, serde_json::Error> {
    serde_json::from_str(json)
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
        ImportFormat::Edit => generic::parse_edit_view(&lines),
        ImportFormat::CraveEdit => crave::parse_crave_edit_view(&lines),
        ImportFormat::AfterHour => afterhour::parse_afterhour_view(&lines),
        ImportFormat::GirlfriendGpt => girlfriendgpt::parse_ggpt_view(&lines),
        ImportFormat::JanitorEdit => janitor::parse_janitor_edit(&lines),
        ImportFormat::JanitorProfile => janitor::parse_janitor_profile(&lines),
        ImportFormat::Profile => generic::parse_profile_view(&lines),
        // For LorebookHtml, we need a separate entry point or return type.
        // Since parse_clipboard returns ParsedCharacterData, we might need a separate function for Lorebooks.
        // See parse_spicychat_lorebook below.
        _ => {
            // Fallback to Profile parser as it's more generic/loose
            generic::parse_profile_view(&lines)
        }
    }
}

pub(crate) fn find_next_value_index(lines: &[&str], current_index: usize) -> Option<usize> {
    for i in (current_index + 1)..lines.len() {
        if !lines[i].trim().is_empty() && lines[i] != "*" {
            return Some(i);
        }
    }
    None
}
