use eframe::egui;
use egui_cosmic_text::{
    atlas::TextureAtlas,
    cosmic_text::{Action, Attrs, Color, Edit, Family, FontSystem, Motion, Shaping, SwashCache},
    widget::{
        CosmicEdit, EditorActions, FillWidth, HoverStrategy, Interactivity, LayoutMode, LineHeight,
    },
};
use std::collections::HashMap;

pub struct CodeEditor<'a> {
    text: &'a mut String,
    id: String,
    desired_lines: usize,
    is_single_line: bool,
    search_query: Option<String>,
    font_family: Family<'a>,
    spell_checker: Option<std::sync::Arc<crate::ui::spell_check::SpellChecker>>,
    font_size_offset: f32,
    bright_mode: bool,
}

/// Custom context menu that strictly handles Cut, Copy, and Paste.
/// Optimized for reliability and immediate synchronization.
struct SimpleContextMenu<'a> {
    clipboard: &'a mut arboard::Clipboard,
    force_sync: &'a mut bool,
    id: String,
}

impl<'a> egui_cosmic_text::widget::ContextMenu for SimpleContextMenu<'a> {
    fn ui<L: LayoutMode>(
        self,
        ui: &mut egui::Ui,
        editor: &mut CosmicEdit<L>,
        font_system: &mut FontSystem,
    ) -> EditorActions {
        let mut actions = EditorActions::default();

        if ui.button("✂ Cut").clicked() {
            if editor.cut(ui, font_system) {
                actions.scroll_to_cursor = true;
                actions.focus = true;
                *self.force_sync = true;
            }
            ui.close_menu();
        }

        if ui.button("📋 Copy").clicked() {
            editor.copy(ui);
            ui.close_menu();
        }

        if ui.button("📋 Paste").clicked() {
            // High-Reliability fix: Create a fresh clipboard handle on click.
            let mut fresh_cb = arboard::Clipboard::new().ok();
            let cb = fresh_cb.as_mut().unwrap_or(self.clipboard);

            match cb.get_text() {
                Ok(text) => {
                    if !text.is_empty() {
                        eprintln!(
                            "[CodeEditor] [ID: {}] Menu Paste: Inserting {} chars",
                            self.id,
                            text.chars().count()
                        );
                        editor.insert_string(text, font_system);
                        actions.scroll_to_cursor = true;
                        actions.focus = true;
                        *self.force_sync = true;
                    } else {
                        eprintln!(
                            "[CodeEditor] [ID: {}] Menu Paste: Clipboard text is empty",
                            self.id
                        );
                    }
                }
                Err(e) => {
                    eprintln!(
                        "[CodeEditor] [ID: {}] Menu Paste: Clipboard error: {:?}",
                        self.id, e
                    );
                    // Fallback
                    if let Ok(text) = self.clipboard.get_text() {
                        if !text.is_empty() {
                            editor.insert_string(text, font_system);
                            actions.scroll_to_cursor = true;
                            actions.focus = true;
                            *self.force_sync = true;
                        }
                    }
                }
            }
            ui.close_menu();
        }

        actions
    }

    fn enabled(&self) -> bool {
        true
    }
}

impl<'a> CodeEditor<'a> {
    pub fn new(text: &'a mut String, id: impl Into<String>, font_family: Family<'a>) -> Self {
        Self {
            text,
            id: id.into(),
            desired_lines: 10,
            is_single_line: false,
            search_query: None,
            font_family,
            spell_checker: None,
            font_size_offset: 0.0,
            bright_mode: true,
        }
    }

    pub fn desired_lines(mut self, lines: usize) -> Self {
        self.desired_lines = lines;
        self
    }

    pub fn single_line(mut self) -> Self {
        self.is_single_line = true;
        self.desired_lines = 1;
        self
    }

    pub fn highlight(mut self, query: impl Into<String>) -> Self {
        let q = query.into();
        if q.len() >= 3 {
            self.search_query = Some(q);
        }
        self
    }

    pub fn spell_check(
        mut self,
        checker: Option<std::sync::Arc<crate::ui::spell_check::SpellChecker>>,
    ) -> Self {
        self.spell_checker = checker;
        self
    }

    pub fn font_size_offset(mut self, offset: f32) -> Self {
        self.font_size_offset = offset;
        self
    }

    pub fn bright_mode(mut self, enabled: bool) -> Self {
        self.bright_mode = enabled;
        self
    }

    fn build_spans<'b>(
        &self,
        text: &'b str,
        default_attrs: Attrs<'b>,
        highlight_attrs: Attrs<'b>,
    ) -> Vec<(&'b str, Attrs<'b>)> {
        let mut spans = Vec::new();

        if let Some(query) = &self.search_query {
            let query_lower = query.to_lowercase();
            let text_lower = text.to_lowercase();

            let mut last_end = 0;

            // Note: simple ASCII case-insensitive matching for now to match index mapping
            // This works because we are slicing the original str based on byte indices from match_indices on lowercased version.
            // This assumes 1:1 byte mapping between lower and upper which is mostly true for what we care about here,
            // but effectively we should rely on the match_indices.

            for (start, _) in text_lower.match_indices(&query_lower) {
                // Pre-match segment
                if start > last_end {
                    spans.push((&text[last_end..start], default_attrs));
                }

                // Match segment
                let end = start + query.len();
                spans.push((&text[start..end], highlight_attrs));

                last_end = end;
            }

            // Remaining segment
            if last_end < text.len() {
                spans.push((&text[last_end..], default_attrs));
            }

            if spans.is_empty() {
                spans.push((text, default_attrs));
            }
        } else {
            spans.push((text, default_attrs));
        }

        spans
    }

    pub fn show(
        self,
        ui: &mut egui::Ui,
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
        atlas: &mut TextureAtlas,
        editors: &mut HashMap<String, CosmicEdit<FillWidth>>,
        clipboard: &mut arboard::Clipboard,
    ) -> egui::Response {
        let pixels_per_point = ui.ctx().pixels_per_point();

        // 1. Aesthetics
        let visuals = ui.visuals();
        let egui_text_color = if self.bright_mode {
            visuals.strong_text_color()
        } else {
            visuals.text_color()
        };

        let cosmic_color = Color::rgba(
            egui_text_color.r(),
            egui_text_color.g(),
            egui_text_color.b(),
            egui_text_color.a(),
        );

        let mut font_size = ui
            .style()
            .text_styles
            .get(&egui::TextStyle::Monospace)
            .map(|id| id.size)
            .unwrap_or(14.0);

        font_size += self.font_size_offset;
        let line_height_val = font_size * 1.4;

        let default_attrs = Attrs::new().family(self.font_family).color(cosmic_color);

        // Highlight attributes: Gold/Orange for visibility + Bold
        let highlight_color = Color::rgba(255, 215, 0, 255);
        let highlight_attrs = Attrs::new()
            .family(self.font_family)
            .color(highlight_color)
            .weight(egui_cosmic_text::cosmic_text::Weight::BOLD);

        let scaled_font_size = (font_size * pixels_per_point).round();
        let scaled_line_height = (line_height_val * pixels_per_point).round();

        // 2. Persistent Widget Management
        if !editors.contains_key(&self.id) {
            let mut widget = CosmicEdit::new(
                scaled_font_size,
                LineHeight::Absolute(scaled_line_height),
                Interactivity::Enabled,
                HoverStrategy::Widget,
                FillWidth::default(),
                font_system,
            );

            let spans = self.build_spans(self.text.as_str(), default_attrs, highlight_attrs);
            widget.set_text(spans, default_attrs, Shaping::Advanced, font_system);
            editors.insert(self.id.clone(), widget);
        }

        let mut cosmic_edit = editors.remove(&self.id).unwrap();
        cosmic_edit.set_font_size(
            scaled_font_size,
            LineHeight::Absolute(scaled_line_height),
            font_system,
        );

        // 3. State Tracking (Golden Sync Logic) + Highlight Check
        let last_model_text_id = ui.make_persistent_id(format!("{}_last_model_text", self.id));
        let last_search_query_id = ui.make_persistent_id(format!("{}_last_search_query", self.id));
        let last_font_family_id = ui.make_persistent_id(format!("{}_last_font_family", self.id));
        let cursor_req_id = ui.make_persistent_id(format!("{}_cursor_req", self.id));

        let last_model_text: Option<String> = ui.data(|d| d.get_temp(last_model_text_id));
        let last_search_query: Option<String> = ui.data(|d| d.get_temp(last_search_query_id));
        let last_font_family: Option<String> = ui.data(|d| d.get_temp(last_font_family_id));
        let last_bright_mode: Option<bool> =
            ui.data(|d| d.get_temp(ui.make_persistent_id(format!("{}_last_bright_mode", self.id))));
        let cursor_req: Option<bool> = ui.data(|d| d.get_temp(cursor_req_id));

        // Use string comparison for query/font to avoid TypeId issues with Option<String> persistence
        let current_query_str = self.search_query.as_deref().unwrap_or("");
        let last_query_str = last_search_query.as_deref().unwrap_or("");

        let current_font_str = format!("{:?}", self.font_family);
        let last_font_str = last_font_family.as_deref().unwrap_or("");

        let query_changed = last_query_str != current_query_str;
        let font_changed = last_font_str != current_font_str;
        let bright_mode_changed = last_bright_mode != Some(self.bright_mode);
        let text_changed = last_model_text.as_ref() != Some(self.text);

        // --- CRITICAL: NORMALIZE LINE ENDINGS ---
        // CRLF (\r\n) causes 1-byte drift per line between zspell and cosmic-text.
        // We force all text to \n normalization before it touches CosmicEdit or zspell.
        if self.text.contains('\r') {
            *self.text = self.text.replace("\r\n", "\n").replace('\r', "\n");
        }

        // Handle deferred cursor request (rare, triggered by focus clicks)
        if cursor_req.unwrap_or(false) {
            let mut editor = cosmic_edit.into_editor();
            editor.action(font_system, Action::Motion(Motion::BufferEnd));
            cosmic_edit = CosmicEdit::from_editor(
                editor,
                Interactivity::Enabled,
                HoverStrategy::Widget,
                FillWidth::default(),
            );
            ui.data_mut(|d| d.remove_temp::<bool>(cursor_req_id));
        }

        // --- CRITICAL: EXTERNAL vs INTERNAL SYNC ---
        // We only call set_text if the text changed externally.
        // If we just typed something, clean_editor_text will match the model, and we skip set_text to preserve undo history.
        let editor_text = cosmic_edit.text();
        let clean_editor_text = if self.is_single_line {
            editor_text.trim_end_matches('\n').replace('\n', "")
        } else {
            editor_text.trim_end_matches('\n').to_string()
        };

        let external_change = text_changed && (*self.text != clean_editor_text);

        if external_change || query_changed || font_changed || bright_mode_changed {
            if query_changed || font_changed || bright_mode_changed {
                ui.ctx().request_repaint();
            }
            let spans = self.build_spans(self.text.as_str(), default_attrs, highlight_attrs);
            cosmic_edit.set_text(spans, default_attrs, Shaping::Advanced, font_system);
        }

        // 4. Interaction & Rendering
        let line_height_val = line_height_val;
        let force_sync_back_cell = std::cell::Cell::new(false);

        let (response, cosmic_edit) = ui
            .push_id(&self.id, |ui| {
                egui::Frame::canvas(ui.style())
                    .fill(ui.visuals().extreme_bg_color)
                    .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                    .rounding(4.0)
                    .inner_margin(egui::Margin::same(6.0))
                    .show(ui, |ui| {
                        let min_height = self.desired_lines as f32 * line_height_val;
                        ui.set_min_height(min_height);

                        let mut force_sync_back = false;
                        let resp = cosmic_edit.ui(
                            ui,
                            font_system,
                            swash_cache,
                            atlas,
                            SimpleContextMenu {
                                clipboard,
                                force_sync: &mut force_sync_back,
                                id: self.id.clone(),
                            },
                        );
                        force_sync_back_cell.set(force_sync_back);

                        // Capture changed status BEFORE any potential reconstruction
                        let internal_changed = cosmic_edit.changed_this_frame();

                        // --- SPELL CHECK RENDERING ---
                        if let Some(checker) = &self.spell_checker {
                            // Access the internal buffer once to ensure consistent coordinates and indices
                            cosmic_edit.editor().with_buffer(|buffer| {
                                // 1. Calculate absolute byte offsets for each line
                                // cosmic-text offsets in LayoutRun are relative to the logical line start.
                                // zspell offsets are absolute to the whole string.
                                let mut line_offsets = Vec::with_capacity(buffer.lines.len());
                                let mut full_text = String::new();
                                for (i, line) in buffer.lines.iter().enumerate() {
                                    if i > 0 {
                                        full_text.push('\n');
                                    }
                                    line_offsets.push(full_text.len());
                                    full_text.push_str(line.text());
                                }

                                let glitches = checker.check(&full_text);

                                if !glitches.is_empty() {
                                    let painter = ui.painter();
                                    let underline_stroke =
                                        egui::Stroke::new(1.0, egui::Color32::RED);
                                    let rect_min = resp.rect.min;

                                    for run in buffer.layout_runs() {
                                        let line_y = run.line_y;
                                        let line_offset =
                                            line_offsets.get(run.line_i).cloned().unwrap_or(0);

                                        for (start_byte, end_byte) in &glitches {
                                            // Ignore single characters (length 1) to avoid tiny underlines on '-' or lone letters
                                            if end_byte - start_byte <= 1 {
                                                continue;
                                            }

                                            for glyph in run.glyphs {
                                                // Convert line-relative indices to absolute indices
                                                let abs_start = line_offset + glyph.start;
                                                let abs_end = line_offset + glyph.end;

                                                // Robust overlap check for alignment
                                                if abs_start < *end_byte && abs_end > *start_byte {
                                                    // CRITICAL: cosmic-text works in pixels (scaled by ppp).
                                                    // egui painter works in logical points.
                                                    // We must divide both position and width.
                                                    let x = rect_min.x + glyph.x / pixels_per_point;
                                                    let y = rect_min.y
                                                        + (line_y + 1.0) / pixels_per_point;
                                                    let w = glyph.w / pixels_per_point;

                                                    painter.line_segment(
                                                        [egui::pos2(x, y), egui::pos2(x + w, y)],
                                                        underline_stroke,
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            });
                        }

                        // Click-to-focus on empty space
                        let available = ui.available_rect_before_wrap();
                        if available.height() > 0.0 {
                            let filler_resp = ui.allocate_rect(available, egui::Sense::click());
                            if filler_resp.clicked() {
                                resp.request_focus();
                                ui.data_mut(|d| d.insert_temp(cursor_req_id, true));
                                ui.ctx().request_repaint();
                            }
                        }

                        // Robust Keyboard Overrides
                        if resp.has_focus() {
                            ui.input_mut(|i| {
                                if self.is_single_line
                                    && i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                                {
                                    resp.surrender_focus();
                                }

                                if i.consume_key(egui::Modifiers::COMMAND, egui::Key::V) {
                                    if let Ok(text) = clipboard.get_text() {
                                        if !text.is_empty() {
                                            let text = if self.is_single_line {
                                                text.replace('\n', " ").replace('\r', "")
                                            } else {
                                                text.replace("\r\n", "\n").replace('\r', "\n")
                                            };
                                            cosmic_edit.insert_string(text, font_system);
                                            force_sync_back_cell.set(true);
                                        }
                                    }
                                }

                                // Explicit Undo/Redo handling
                                // FIX: Use key_pressed instead of consume_key to prevent rapid-fire undo on hold.
                                // We check 'key_pressed' which triggers once per press (mostly), then consume it.
                                if i.key_pressed(egui::Key::Z) && i.modifiers.command {
                                    if i.modifiers.shift {
                                        cosmic_edit.redo();
                                    } else {
                                        cosmic_edit.undo();
                                    }
                                    force_sync_back_cell.set(true);
                                    i.consume_key(egui::Modifiers::COMMAND, egui::Key::Z);
                                    if i.modifiers.shift {
                                        i.consume_key(
                                            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                                            egui::Key::Z,
                                        );
                                    }
                                }

                                // Redo alternative (Ctrl+Y)
                                if i.key_pressed(egui::Key::Y) && i.modifiers.command {
                                    cosmic_edit.redo();
                                    force_sync_back_cell.set(true);
                                    i.consume_key(egui::Modifiers::COMMAND, egui::Key::Y);
                                }
                            });
                        }

                        // Final Sync to model
                        let force_sync = force_sync_back_cell.get();
                        if internal_changed || force_sync {
                            let new_text = cosmic_edit.text();
                            let clean_new = if self.is_single_line {
                                new_text.trim_end_matches('\n').replace('\n', "")
                            } else {
                                new_text.trim_end_matches('\n').to_string()
                            };

                            if *self.text != clean_new {
                                *self.text = clean_new;
                                ui.data_mut(|d| {
                                    d.insert_temp(last_model_text_id, self.text.clone())
                                });
                            }
                        }

                        if resp.clicked() {
                            resp.request_focus();
                        }

                        (resp, cosmic_edit)
                    })
                    .inner
            })
            .inner;

        // 6. Final State Recording
        ui.data_mut(|d| {
            d.insert_temp(last_model_text_id, self.text.clone());
            // Store as plain String to match retrieval type
            d.insert_temp(last_search_query_id, current_query_str.to_string());
            d.insert_temp(last_font_family_id, current_font_str);
            d.insert_temp(
                ui.make_persistent_id(format!("{}_last_bright_mode", self.id)),
                Some(self.bright_mode),
            );
        });

        editors.insert(self.id, cosmic_edit);

        response
    }
}
