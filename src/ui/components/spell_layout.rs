use crate::ui::spell_check::SpellChecker;
use eframe::egui;
use std::sync::Arc;

pub fn create_spell_check_layouter(
    spell_checker: Option<Arc<SpellChecker>>,
    search_query: String,
) -> Box<dyn Fn(&egui::Ui, &str, f32) -> Arc<egui::Galley>> {
    Box::new(move |ui: &egui::Ui, text: &str, wrap_width: f32| {
        let mut layout_job = egui::text::LayoutJob::default();

        let font_id = egui::TextStyle::Body.resolve(ui.style());
        let default_format = egui::TextFormat {
            font_id: font_id.clone(),
            color: ui.style().visuals.text_color(),
            ..Default::default()
        };

        // If no spell checker and no search, efficient return
        if spell_checker.is_none() && search_query.len() < 3 {
            layout_job.append(text, 0.0, default_format);
            layout_job.wrap.max_width = wrap_width;
            return ui.fonts(|f| f.layout_job(layout_job));
        }

        // We need to merge ranges from Spell Checker and Search Highlight.
        // Or handle them hierarchically. Search Highlight usually takes precedence (yellow bg).
        // Spell Check is usually a red wavy underline.
        // egui::TextFormat supports `underline: Stroke`.

        // 1. Get Spell Check Ranges
        let spell_errors = if let Some(checker) = &spell_checker {
            checker.check(text)
        } else {
            Vec::new()
        };

        // 2. Get Search Ranges
        let mut search_matches = Vec::new();
        if search_query.len() >= 3 {
            let search_lower = search_query.to_lowercase();
            let text_lower = text.to_lowercase();
            for (start, _) in text_lower.match_indices(&search_lower) {
                search_matches.push(start..start + search_query.len());
            }
        }

        // 3. Build Layout Job
        // We can just iterate char by char or construct ranges.
        // Since we have two independent sets of ranges, overlap is possible.
        // "MisspelledWord" containing "SearchQuery" or vice versa.
        // Merging ranges is tricky.

        // Simpler approach:
        // Iterate over text indices. For each segment, determine style.
        // Points of interest: starts and ends of all ranges.

        let mut points = Vec::new();
        points.push(0);
        points.push(text.len());

        for range in &spell_errors {
            points.push(range.0);
            points.push(range.1);
        }
        for range in &search_matches {
            points.push(range.start);
            points.push(range.end);
        }

        points.sort();
        points.dedup();

        for i in 0..points.len() - 1 {
            let start = points[i];
            let end = points[i + 1];
            if start >= end {
                continue;
            }

            let slice = &text[start..end];
            let mut format = default_format.clone();

            // Check Search match
            let is_search_match = search_matches.iter().any(|r| r.contains(&start));
            if is_search_match {
                format.background = egui::Color32::from_rgb(255, 255, 0); // Yellow
                format.color = egui::Color32::BLACK;
            }

            // Check Spell error

            if spell_errors.iter().any(|r| start >= r.0 && end <= r.1) {
                format.underline = egui::Stroke::new(1.0, egui::Color32::RED);
            }

            layout_job.append(slice, 0.0, format);
        }

        layout_job.wrap.max_width = wrap_width;
        ui.fonts(|f| f.layout_job(layout_job))
    })
}
