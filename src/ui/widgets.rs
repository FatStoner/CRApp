use eframe::egui;

// Heuristic snippet extractor for search
pub fn extract_snippets(text: &str, query: &str) -> Vec<String> {
    let lower_text = text.to_lowercase();
    let lower_query = query.to_lowercase();
    let mut snippets = Vec::new();
    
    // Naive finding of all occurrences
    let indices: Vec<_> = lower_text.match_indices(&lower_query).map(|(i, _)| i).collect();
    
    for start_idx in indices {
        let snippet_start = start_idx.saturating_sub(20);
        let snippet_end = (start_idx + query.len() + 30).min(text.len());
        
        // Clean up boundaries to avoid chopped words (simple heuristic: expand to space)
        let slice = &text[snippet_start..snippet_end];
        let display = format!("...{}...", slice.replace('\n', " "));
        snippets.push(display);
        
        if snippets.len() >= 3 { break; } // Limit per field
    }
    
    snippets
}
