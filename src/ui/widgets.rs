use arboard::Clipboard;
use eframe::egui;

// Helper struct for arbitrary data storage
#[derive(Clone, Copy, Debug, Default)]
struct TextSelection((usize, usize));

pub fn track_text_selection(ui: &egui::Ui, response: &egui::Response) {
    let id = response.id;
    if let Some(state) = egui::TextEdit::load_state(ui.ctx(), id) {
        if let Some(range) = state.cursor.char_range() {
            // If we have a selection (range width > 0)
            if range.primary.index != range.secondary.index {
                let sel = TextSelection((range.primary.index, range.secondary.index));
                ui.data_mut(|d| d.insert_temp(id, sel));
                eprintln!("DEBUG: Stored selection {:?}", sel);
            } else {
                // No selection (width 0).
                // We clear the cache ONLY if the user actively interacts to "clear" it:
                // 1. Left Click (re-position cursor freely)
                // 2. Typing/Change (modifying text clears selection context)
                // We DO NOT clear on Right Click (Secondary) or simple Focus (which might be transient).
                if response.clicked_by(egui::PointerButton::Primary) || response.changed() {
                    ui.data_mut(|d| d.remove_temp::<TextSelection>(id));
                    eprintln!("DEBUG: CLEARING cache (Primary Click or Text Change)");
                } else {
                    // eprintln!("DEBUG: Preserving cache (No disruptive action)");
                }
            }
        }
    }
}

pub fn text_context_menu(ui: &mut egui::Ui, text: &mut String, id: egui::Id) {
    // Try to get current realtime state
    let mut current_cursor_range = None;
    if let Some(state) = egui::TextEdit::load_state(ui.ctx(), id) {
        if let Some(range) = state.cursor.char_range() {
            current_cursor_range = Some((range.primary.index, range.secondary.index));
        }
    }

    eprintln!(
        "DEBUG: Menu Open. Realtime Range: {:?}",
        current_cursor_range
    );

    // If realtime state is just a cursor (no selection), check if we have a stored ("sticky") selection
    if let Some((start, end)) = current_cursor_range {
        if start == end {
            // active selection is empty (just cursor), check cache
            if let Some(stored) = ui.data(|d| d.get_temp::<TextSelection>(id)) {
                eprintln!("DEBUG: Using Cached Selection: {:?}", stored);
                current_cursor_range = Some(stored.0);
            } else {
                eprintln!("DEBUG: No Cached Selection found.");
            }
        } else {
            eprintln!("DEBUG: Using Realtime Selection.");
        }
    } else {
        // No cursor info at all? Check cache.
        if let Some(stored) = ui.data(|d| d.get_temp::<TextSelection>(id)) {
            eprintln!("DEBUG: No Realtime State, Using Cached: {:?}", stored);
            current_cursor_range = Some(stored.0);
        } else {
            eprintln!("DEBUG: No Realtime State, No Cache.");
        }
    }

    if ui.button("✂ Cut").clicked() {
        let mut clipboard = Clipboard::new().ok();

        if let Some((start, end)) = current_cursor_range {
            let (min, max) = if start < end {
                (start, end)
            } else {
                (end, start)
            };

            if min != max {
                // Selection exists
                if let Some((byte_start, _)) = text.char_indices().nth(min) {
                    if let Some((byte_end, _)) = text.char_indices().nth(max) {
                        let slice = &text[byte_start..byte_end];
                        if let Some(cb) = &mut clipboard {
                            let _ = cb.set_text(slice.to_string());
                        }
                        text.replace_range(byte_start..byte_end, "");
                    } else if max == text.chars().count() {
                        let slice = &text[byte_start..];
                        if let Some(cb) = &mut clipboard {
                            let _ = cb.set_text(slice.to_string());
                        }
                        text.replace_range(byte_start.., "");
                    }

                    // Restore focus and cursor
                    ui.memory_mut(|m| m.request_focus(id));
                    if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), id) {
                        state
                            .cursor
                            .set_char_range(Some(eframe::egui::text::CCursorRange::one(
                                eframe::egui::text::CCursor::new(min),
                            )));
                        state.store(ui.ctx(), id);
                    }
                }
            }
        }
        // Removed fallback "Cut All" to prevent accidents
        ui.close_menu();
    }

    if ui.button("📋 Copy").clicked() {
        let mut clipboard = Clipboard::new().ok();

        if let Some((start, end)) = current_cursor_range {
            let (min, max) = if start < end {
                (start, end)
            } else {
                (end, start)
            };

            if min != max {
                let chars: Vec<(usize, char)> = text.char_indices().collect();
                let byte_start = chars.get(min).map(|(i, _)| *i);
                let byte_end = chars.get(max).map(|(i, _)| *i).or_else(|| Some(text.len()));

                if let (Some(s), Some(e)) = (byte_start, byte_end) {
                    let slice = &text[s..e];
                    if let Some(cb) = &mut clipboard {
                        let _ = cb.set_text(slice.to_string());
                    }
                }
            }
        }
        // Removed fallback "Copy All"
        ui.close_menu();
    }

    if ui.button("📋 Paste").clicked() {
        let mut clipboard = Clipboard::new().ok();
        if let Some(cb) = &mut clipboard {
            if let Ok(content) = cb.get_text() {
                let content_len_chars = content.chars().count();
                let mut insert_index = 0;

                // For paste, we prefer the "current" cursor if it exists, even if it's just a point.
                // But if we have a stored SELECTION, we should probably overwrite it?
                // Standard behavior: If I have a selection, Paste replaces it.
                // So we prioritize selection range over cursor point.

                let target_range = current_cursor_range;

                if let Some((start, end)) = target_range {
                    let (min, max) = if start < end {
                        (start, end)
                    } else {
                        (end, start)
                    };
                    insert_index = min;

                    let chars: Vec<(usize, char)> = text.char_indices().collect();
                    let byte_start = chars.get(min).map(|(i, _)| *i).unwrap_or(text.len());
                    let byte_end = chars.get(max).map(|(i, _)| *i).unwrap_or(text.len());

                    text.replace_range(byte_start..byte_end, &content);
                } else {
                    // Fallback: Append
                    insert_index = text.chars().count();
                    text.push_str(&content);
                }

                ui.memory_mut(|m| m.request_focus(id));
                if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), id) {
                    let new_index = insert_index + content_len_chars;
                    state
                        .cursor
                        .set_char_range(Some(eframe::egui::text::CCursorRange::one(
                            eframe::egui::text::CCursor::new(new_index),
                        )));
                    state.store(ui.ctx(), id);
                }
            }
        }
        ui.close_menu();
    }
}

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
