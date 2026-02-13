use super::types::ImportFormat;

pub fn detect_format(lines: &[&str]) -> ImportFormat {
    // Crave specific check
    if lines.iter().any(|l| l.contains("CraveU AI")) {
        return ImportFormat::CraveEdit;
    }

    if lines
        .iter()
        .any(|l| l.contains("| AfterHours") || l.contains("afterhour.app"))
    {
        return ImportFormat::AfterHour;
    }

    if lines.iter().any(|l| l.contains("GirlfriendGPT")) {
        return ImportFormat::GirlfriendGpt;
    }

    if lines.iter().any(|l| l.to_lowercase().contains("janitor")) {
        if lines
            .iter()
            .any(|l| l.contains("Edit Character") || l.contains("Character Name*"))
        {
            return ImportFormat::JanitorEdit;
        }
        if lines
            .iter()
            .any(|l| l.contains("Analytics") || l.contains("Playing as"))
        {
            return ImportFormat::JanitorProfile;
        }
        // Fallback if unsure but says Janitor? Maybe Profile is safer default for view page
        return ImportFormat::JanitorProfile;
    }

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
