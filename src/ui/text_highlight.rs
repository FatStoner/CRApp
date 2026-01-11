use eframe::egui;

/// Creates a boxed layouter function that highlights search matches in text
pub fn create_highlight_layouter(
    search_query: String,
) -> Box<dyn Fn(&egui::Ui, &str, f32) -> std::sync::Arc<egui::Galley>> {
    Box::new(move |ui: &egui::Ui, text: &str, wrap_width: f32| {
        let mut layout_job = egui::text::LayoutJob::default();

        let font_id = egui::TextStyle::Body.resolve(ui.style());
        let default_format = egui::TextFormat {
            font_id: font_id.clone(),
            ..Default::default()
        };

        if search_query.len() >= 3 {
            let search_lower = search_query.to_lowercase();
            let text_lower = text.to_lowercase();

            let mut last_end = 0;

            // Find all matches
            for (start, _) in text_lower.match_indices(&search_lower) {
                let end = start + search_query.len();

                // Add text before match (normal formatting)
                if start > last_end {
                    layout_job.append(&text[last_end..start], 0.0, default_format.clone());
                }

                // Add highlighted match
                layout_job.append(
                    &text[start..end],
                    0.0,
                    egui::TextFormat {
                        font_id: font_id.clone(),
                        background: egui::Color32::from_rgb(255, 255, 0), // Yellow
                        color: egui::Color32::BLACK,
                        ..Default::default()
                    },
                );

                last_end = end;
            }

            // Add remaining text
            if last_end < text.len() {
                layout_job.append(&text[last_end..], 0.0, default_format);
            }
        } else {
            // No search or too short - use default formatting
            layout_job.append(text, 0.0, default_format);
        }

        layout_job.wrap.max_width = wrap_width;
        ui.fonts(|f| f.layout_job(layout_job))
    })
}
