use eframe::egui;

// Heuristic snippet extractor for search
pub fn extract_snippets(text: &str, query: &str) -> Vec<String> {
    let lower_text = text.to_lowercase();
    let lower_query = query.to_lowercase();
    let mut snippets = Vec::new();

    // Find all occurrences but limit processing to prevent performance issues
    let indices: Vec<_> = lower_text
        .match_indices(&lower_query)
        .map(|(i, _)| i)
        .take(10) // Limit matches processed for very common words
        .collect();

    for start_idx in indices {
        // Calculate desired range
        let snippet_start = start_idx.saturating_sub(20);
        let snippet_end = (start_idx + query.len() + 30).min(text.len());

        // Find UTF-8 safe boundaries to prevent panic
        let safe_start = text.floor_char_boundary(snippet_start);
        let safe_end = text.ceil_char_boundary(snippet_end);

        // Create safe slice
        let slice = &text[safe_start..safe_end];
        let display = format!("...{}...", slice.replace('\n', " "));
        snippets.push(display);

        if snippets.len() >= 3 {
            break;
        } // Limit snippets shown per field
    }

    snippets
}

pub fn paint_avatar_crop(ui: &mut egui::Ui, rect: egui::Rect, uri: &str, rounding: f32) {
    let texture_options = egui::TextureOptions::LINEAR;

    // Check cache/load status to get dimensions
    match ui.ctx().try_load_image(uri.into(), Default::default()) {
        Ok(poll) => {
            match poll {
                egui::load::ImagePoll::Ready { image } => {
                    let w = image.size[0] as f32;
                    let h = image.size[1] as f32;

                    // Calculate "Cover" UVs
                    let img_aspect = w / h;
                    let rect_aspect = rect.width() / rect.height();

                    let mut uv: egui::Rect;

                    if img_aspect > rect_aspect {
                        // Image is wider: keep height (0..1), crop width
                        // Anchor Left: start x at 0.0
                        let uv_w = rect_aspect / img_aspect;
                        uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(uv_w, 1.0));
                    } else {
                        // Image is taller: keep width (0..1), crop height
                        // Anchor Top: start y at 0.0
                        let uv_h = img_aspect / rect_aspect;
                        uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, uv_h));
                    }

                    // Apply 5% Zoom (Cut 5% from borders of the CROP)
                    let w_uv = uv.width();
                    let h_uv = uv.height();

                    uv.min.x += w_uv * 0.05;
                    uv.max.x -= w_uv * 0.05;
                    uv.min.y += h_uv * 0.05;
                    uv.max.y -= h_uv * 0.05;

                    // Render using egui::Image widget which handles rounding/painting correctly
                    egui::Image::new(uri)
                        .uv(uv)
                        .rounding(egui::Rounding::same(rounding))
                        .texture_options(texture_options)
                        .paint_at(ui, rect);
                }
                egui::load::ImagePoll::Pending { .. } => {
                    // Trigger load if not started? Image::new usually does it.
                    // But we used try_load which does trigger.
                    ui.painter().rect_filled(
                        rect,
                        egui::Rounding::same(rounding),
                        egui::Color32::from_gray(30),
                    );
                    ui.spinner();
                    // To ensure it refreshes:
                    ui.ctx().request_repaint();
                }
            }
        }
        Err(_) => {
            ui.painter().rect_filled(
                rect,
                egui::Rounding::same(rounding),
                egui::Color32::from_gray(60),
            );
            // '?' or error icon could go here
        }
    }
}
