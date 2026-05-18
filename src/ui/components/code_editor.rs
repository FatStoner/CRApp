use eframe::egui;
use egui_cosmic_text::{
    atlas::TextureAtlas,
    cosmic_text::{Action, Attrs, Color, Edit, Family, FontSystem, Motion, Shaping, SwashCache},
    widget::{
        CosmicEdit, EditorActions, FillWidth, HoverStrategy, Interactivity, LayoutMode, LineHeight,
    },
};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

pub struct CodeEditor<'a> {
    text: &'a mut String,
    id: String,
    desired_lines: usize,
    max_lines: Option<usize>,
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
    spell_checker: &'a Option<std::sync::Arc<crate::ui::spell_check::SpellChecker>>,
    correction_action: &'a mut Option<(usize, usize, String)>,
}

impl<'a> egui_cosmic_text::widget::ContextMenu for SimpleContextMenu<'a> {
    fn ui<L: LayoutMode>(
        self,
        ui: &mut egui::Ui,
        editor: &mut CosmicEdit<L>,
        font_system: &mut FontSystem,
    ) -> EditorActions {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
        ui.set_min_width(0.0);
        ui.style_mut().spacing.button_padding = egui::vec2(4.0, 2.0);
        ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 2.0);

        let mut actions = EditorActions::default();

        // dictionary logic
        if self.spell_checker.is_some() {
            let target_word_id = egui::Id::new(&self.id).with("context_menu_word");
            let target_word: Option<(String, usize, usize, Vec<String>)> = ui.data(|d| d.get_temp(target_word_id));

            if target_word.is_none() {
                // SQUEEZE: Force the menu to be narrow if only Cut/Copy/Paste are here.
                ui.set_max_width(60.0);
            }

            if let Some((word, start, end, suggestions)) = target_word {
                for suggestion in suggestions {
                    if ui.button(egui::RichText::new(&suggestion).strong()).clicked() {
                        *self.correction_action = Some((start, end, suggestion));
                        let glitches_id = egui::Id::new(&self.id).with("glitches");
                        let target_word_id = egui::Id::new(&self.id).with("context_menu_word");
                        ui.data_mut(|d| {
                            d.remove_temp::<Arc<(Vec<(usize, usize)>, Vec<usize>)>>(glitches_id)
                        });
                        ui.data_mut(|d| d.remove_temp::<(String, usize, usize, Vec<String>)>(target_word_id));
                        ui.close_menu();
                        ui.ctx().request_repaint();
                    }
                }
                
                ui.separator();
                
                let display_word = if word.len() > 12 {
                    format!("{}...", &word[..12])
                } else {
                    word.clone()
                };

                if ui
                    .button(format!("➕ Add \"{}\" to Dictionary", display_word))
                    .clicked()
                {
                    if let Some(checker) = self.spell_checker.as_ref() {
                        checker.add_word(&word);
                        let glitches_id = egui::Id::new(&self.id).with("glitches");
                        // Clear cache to force re-check
                        ui.data_mut(|d| {
                            d.remove_temp::<Arc<(Vec<(usize, usize)>, Vec<usize>)>>(glitches_id)
                        });
                        ui.data_mut(|d| d.remove_temp::<(String, usize, usize, Vec<String>)>(target_word_id));
                        ui.close_menu();
                        ui.ctx().request_repaint();
                    }
                }
                ui.separator();
            }
        }

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
                Err(_e) => {
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
            max_lines: None,
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

    pub fn max_lines(mut self, lines: usize) -> Self {
        self.max_lines = Some(lines);
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

        // REVERT: self.font_family is already Family<'a>, no need to match
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

        let mut cosmic_edit = editors.remove(&self.id).unwrap_or_else(|| {
            // Recover from state loss: create a fresh editor
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
            widget
        });
        // Font Size Caching
        let last_font_size_id = ui.make_persistent_id(format!("{}_last_font_size", self.id));
        let last_font_size: Option<f32> = ui.data(|d| d.get_temp(last_font_size_id));

        if last_font_size != Some(scaled_font_size) {
            cosmic_edit.set_font_size(
                scaled_font_size,
                LineHeight::Absolute(scaled_line_height),
                font_system,
            );
            ui.data_mut(|d| d.insert_temp(last_font_size_id, scaled_font_size));
        }

        // 3. State Tracking (Golden Sync Logic) + Highlight Check
        let last_model_hash_id = ui.make_persistent_id(format!("{}_last_model_hash", self.id));
        let last_search_query_id = ui.make_persistent_id(format!("{}_last_search_query", self.id));
        let last_font_family_id = ui.make_persistent_id(format!("{}_last_font_family", self.id));
        let cursor_req_id = ui.make_persistent_id(format!("{}_cursor_req", self.id));

        let last_model_hash: Option<u64> = ui.data(|d| d.get_temp(last_model_hash_id));
        let last_search_query: Option<String> = ui.data(|d| d.get_temp(last_search_query_id));
        let last_font_family: Option<String> = ui.data(|d| d.get_temp(last_font_family_id));
        let last_bright_mode: Option<bool> =
            ui.data(|d| d.get_temp(ui.make_persistent_id(format!("{}_last_bright_mode", self.id))));
        let cursor_req: Option<bool> = ui.data(|d| d.get_temp(cursor_req_id));

        // Use string comparison for query/font to avoid TypeId issues with Option<String> persistence
        let current_query_str = self.search_query.clone().unwrap_or_default();
        let last_query_str = last_search_query.as_deref().unwrap_or("");

        let current_font_str = format!("{:?}", self.font_family);
        let last_font_str = last_font_family.as_deref().unwrap_or("");

        let query_changed = last_query_str != current_query_str;
        let font_changed = last_font_str != current_font_str;
        let bright_mode_changed = last_bright_mode != Some(self.bright_mode);

        // Calculate hash of current model text only if we are idle or length changed
        // OPTIMIZATION: Check length first. If length matches, probability of change is low (but not zero).
        // We accept this risk to inconsistent state for 1 frame to avoid hashing 100MB/frame.
        let last_len_id = ui.make_persistent_id(format!("{}_last_len", self.id));
        let last_len: Option<usize> = ui.data(|d| d.get_temp(last_len_id));
        let current_len = self.text.len();

        let text_changed = if last_len != Some(current_len) {
            let mut hasher = DefaultHasher::new();
            self.text.hash(&mut hasher);
            let current_model_hash = hasher.finish();
            ui.data_mut(|d| d.insert_temp(last_len_id, current_len));
            ui.data_mut(|d| d.insert_temp(last_model_hash_id, current_model_hash));
            last_model_hash != Some(current_model_hash)
        } else {
            false
        }; // --- CRITICAL: NORMALIZE LINE ENDINGS ---
           // CRLF (\r\n) causes 1-byte drift per line between zspell and cosmic-text.
           // We force all text to \n normalization before it touches CosmicEdit or zspell.
        if text_changed && self.text.contains('\r') {
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
            let spans = self.build_spans(self.text.as_str(), default_attrs, highlight_attrs);
            cosmic_edit.set_text(spans, default_attrs, Shaping::Advanced, font_system);
            ui.data_mut(|d| d.remove_temp::<bool>(cursor_req_id));
        }

        // --- CRITICAL: EXTERNAL vs INTERNAL SYNC ---
        // We only call set_text if the text changed externally.
        // If we just typed something, clean_editor_text will match the model, and we skip set_text to preserve undo history.
        //
        // OPTIMIZATION: Only fetch editor text (expensive allocation) if model hash changed.
        let external_change = if text_changed {
            let editor_text = cosmic_edit.text();
            let clean_editor_text = if self.is_single_line {
                editor_text.trim_end_matches('\n').replace('\n', "")
            } else {
                editor_text.trim_end_matches('\n').to_string()
            };
            *self.text != clean_editor_text
        } else {
            false
        };

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

        let id_clone = self.id.clone();
        let (response, cosmic_edit) = ui
            .push_id(id_clone, |ui| {
                egui::Frame::canvas(ui.style())
                    .fill(ui.visuals().extreme_bg_color)
                    .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                    .rounding(4.0)
                    .inner_margin(egui::Margin::same(6.0))
                    .show(ui, |ui| {
                        let min_height = self.desired_lines as f32 * line_height_val;
                        ui.set_min_height(min_height);
                        if self.is_single_line {
                            ui.set_max_height(min_height);
                        } else if let Some(max_l) = self.max_lines {
                            let max_height = max_l as f32 * line_height_val;
                            ui.set_max_height(max_height);
                        }

                        if self.is_single_line {
                            ui.set_max_height(min_height);
                        }
                        ui.set_width(ui.available_width());

                        let mut force_sync_back = false;
                        
                        // INTERCEPT ENTER EARLY for single-line to prevent cosmic-text from processing it
                        if self.is_single_line {
                            let resp_id = egui::Id::new(&self.id);
                            let has_focus = ui.memory(|m| m.has_focus(resp_id));
                            if has_focus {
                                if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
                                    ui.memory_mut(|m| m.surrender_focus(resp_id));
                                }
                            }
                        }

                        let mut correction_action: Option<(usize, usize, String)> = None;
                        
                        let resp = cosmic_edit.ui(
                            ui,
                            font_system,
                            swash_cache,
                            atlas,
                            SimpleContextMenu {
                                clipboard,
                                force_sync: &mut force_sync_back,
                                id: self.id.clone(),
                                spell_checker: &self.spell_checker,
                                correction_action: &mut correction_action,
                            },
                        );
                        force_sync_back_cell.set(force_sync_back);

                        if let Some((start, end, suggestion)) = correction_action {
                            if self.text.is_char_boundary(start) && self.text.is_char_boundary(end) {
                                self.text.replace_range(start..end, &suggestion);
                                let spans = self.build_spans(self.text.as_str(), default_attrs, highlight_attrs);
                                cosmic_edit.set_text(spans, default_attrs, Shaping::Advanced, font_system);
                                force_sync_back_cell.set(true);
                            }
                        }

                        // Capture changed status BEFORE any potential reconstruction
                        let internal_changed = cosmic_edit.changed_this_frame();

                        // --- SPELL CHECK RENDERING ---
                        if let Some(checker) = &self.spell_checker {
                            // Caching Logic
                            let glitches_id = egui::Id::new(&self.id).with("glitches");
                            let cached_glitches: Option<Arc<(Vec<(usize, usize)>, Vec<usize>)>> =
                                ui.data(|d| d.get_temp(glitches_id));

                            // Check if we need to re-run spell check
                            // - internal_changed: User typed something this frame
                            // - external_change: Model text changed from outside
                            // - cached_glitches.is_none(): First run or cache cleared
                            let need_check =
                                internal_changed || external_change || force_sync_back_cell.get() || cached_glitches.is_none();

                            // Access the internal buffer once to ensure consistent coordinates and indices
                            cosmic_edit.editor().with_buffer(|buffer| {
                                let (arc_glitches, arc_offsets) = if need_check {
                                    // 1. Calculate absolute byte offsets for each line
                                    // cosmic-text offsets in LayoutRun are relative to the logical line start.
                                    // zspell offsets are absolute to the whole string.
                                    let mut line_offsets = Vec::with_capacity(buffer.lines.len());
                                    let mut current_offset = 0;
                                    let mut full_text = String::with_capacity(current_offset); // heuristic? no current_offset is 0 here.

                                    for (i, line) in buffer.lines.iter().enumerate() {
                                        if i > 0 {
                                            current_offset += 1; // '\n'
                                            full_text.push('\n');
                                        }
                                        line_offsets.push(current_offset);
                                        let text = line.text();
                                        current_offset += text.len();
                                        full_text.push_str(text);
                                    }

                                    let glitches = checker.check(&full_text);
                                    let cache_data = Arc::new((glitches, line_offsets));

                                    ui.data_mut(|d| d.insert_temp(glitches_id, cache_data.clone()));
                                    (cache_data.0.clone(), cache_data.1.clone())
                                } else {
                                    let cache = cached_glitches.unwrap();
                                    (cache.0.clone(), cache.1.clone())
                                };

                                // Deref the Arc to get slice
                                let glitches = &arc_glitches;
                                let line_offsets = &arc_offsets;

                                if !glitches.is_empty() {
                                    let painter = ui.painter();
                                    let underline_stroke =
                                        egui::Stroke::new(1.0, egui::Color32::RED);
                                    let rect_min = resp.rect.min;
                                    let clip_rect = ui.clip_rect();

                                    // OPTIMIZATION: Linear scan O(N) since both layout runs and glitches are sorted
                                    let mut glitch_idx = 0;

                                    for run in buffer.layout_runs() {
                                        let line_y = run.line_y;

                                        // VISIBILITY CULLING
                                        // Check if this line is visible in the current clip rect
                                        // Margin to avoid flickering at edges
                                        let margin = 100.0;
                                        let line_top = rect_min.y + line_y / pixels_per_point;
                                        let line_bottom =
                                            line_top + run.line_height / pixels_per_point;

                                        if line_bottom < clip_rect.min.y - margin {
                                            continue; // Skip lines above view
                                        }
                                        if line_top > clip_rect.max.y + margin {
                                            break; // Stop processing lines below view (layout_runs are ordered)
                                        }

                                        let line_offset =
                                            line_offsets.get(run.line_i).cloned().unwrap_or(0);
                                        let line_end_offset = line_offset + run.text.len();

                                        // Skip glitches that end before this line starts
                                        while glitch_idx < glitches.len() {
                                            let (_, end) = glitches[glitch_idx];
                                            if end > line_offset {
                                                break;
                                            }
                                            glitch_idx += 1;
                                        }

                                        // Iterate glitches that might overlap this line
                                        let mut current_idx = glitch_idx;
                                        while current_idx < glitches.len() {
                                            let (start_byte, end_byte) = &glitches[current_idx];

                                            // Stop if this glitch starts after the line ends
                                            if *start_byte >= line_end_offset {
                                                break;
                                            }

                                            // Increment for next potential line check
                                            current_idx += 1;

                                            // Ignore single characters (length 1) to avoid tiny underlines on '-' or lone letters
                                            if *end_byte - *start_byte <= 1 {
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
                        // Click-to-focus on empty space
                        // Determine how much space is left to reach min_height (desired_lines)
                        // We use min_height because desired_lines dictates the minimum visual size.
                        // available_rect_before_wrap() might be large if max_height is set, so we constrain it manually.

                        let current_used_height = ui.min_rect().height();
                        // Note: current_used_height inside this Ui might just be cosmic_edit height + padding.

                        let target_min_height = if self.is_single_line {
                            0.0 // No filler for single line needed usually, but logic holds
                        } else {
                            self.desired_lines as f32 * line_height_val
                        };

                        let remaining_to_min = (target_min_height - current_used_height).max(0.0);

                        if remaining_to_min > 0.0 {
                            let (id, rect) = ui
                                .allocate_space(egui::vec2(ui.available_width(), remaining_to_min));
                            let filler_resp = ui.interact(rect, id, egui::Sense::click());
                            if filler_resp.clicked() {
                                resp.request_focus();
                                ui.data_mut(|d| d.insert_temp(cursor_req_id, true));
                                ui.ctx().request_repaint();
                            }
                        }

                        // Robust Keyboard Overrides
                        if resp.has_focus() {
                            ui.input_mut(|i| {
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

                                let mut hasher = DefaultHasher::new();
                                self.text.hash(&mut hasher);
                                let new_hash = hasher.finish();

                                ui.data_mut(|d| d.insert_temp(last_model_hash_id, new_hash));
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

        // Handle Context Menu Hit Testing on Right Click
        if response.secondary_clicked() {
            if self.spell_checker.is_some() {
                let glitches_id = egui::Id::new(&self.id).with("glitches");
                let cached_glitches: Option<Arc<(Vec<(usize, usize)>, Vec<usize>)>> =
                    ui.data(|d| d.get_temp(glitches_id));

                let target_word_id = egui::Id::new(&self.id).with("context_menu_word");
                let mut found_word = None;

                if let Some(data) = cached_glitches {
                    let (glitches, line_offsets) = &*data;

                    // 1. Mouse Hit Test
                    if let Some(mouse_pos) = response.interact_pointer_pos() {
                        ui.ctx().request_repaint(); // Ensure menu shows up with data next frame

                        let pixels_per_point = ui.ctx().pixels_per_point();
                        let rect_min = response.rect.min;

                        cosmic_edit.editor().with_buffer(|buffer| {
                            'hit_test: for run in buffer.layout_runs() {
                                let line_y = run.line_y;
                                let line_top = rect_min.y + line_y / pixels_per_point;
                                let line_height = run.line_height / pixels_per_point;
                                let line_bottom = line_top + line_height;

                                // Expanded Y-bounds for easier clicking
                                let margin = line_height * 0.5;
                                if mouse_pos.y < line_top - margin
                                    || mouse_pos.y > line_bottom + margin
                                {
                                    continue;
                                }

                                let line_offset = if run.line_i < line_offsets.len() {
                                    line_offsets[run.line_i]
                                } else {
                                    continue;
                                };

                                for (start, end) in glitches {
                                    let start = *start;
                                    let end = *end;

                                    for glyph in run.glyphs {
                                        let abs_glyph_start = line_offset + glyph.start;
                                        let abs_glyph_end = line_offset + glyph.end;

                                        if abs_glyph_start < end && abs_glyph_end > start {
                                            let x = rect_min.x + glyph.x / pixels_per_point;
                                            let w = glyph.w / pixels_per_point;

                                            // Standard X bounds
                                            if mouse_pos.x >= x && mouse_pos.x <= x + w {
                                                // Extract word
                                                let mut full_text = String::new();
                                                for (i, line) in buffer.lines.iter().enumerate() {
                                                    if i > 0 {
                                                        full_text.push('\n');
                                                    }
                                                    full_text.push_str(line.text());
                                                }
                                                if end <= full_text.len() {
                                                    let word = full_text[start..end].to_string();
                                                    let suggestions = self.spell_checker.as_ref().unwrap().suggest(&word);
                                                    found_word = Some((word, start, end, suggestions));
                                                    break 'hit_test;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        });
                    }

                    // 2. Cursor Hit Test Fallback
                    if found_word.is_none() {
                        let cursor = cosmic_edit.editor().cursor();
                        cosmic_edit.editor().with_buffer(|buffer| {
                            if cursor.line < buffer.lines.len() && cursor.line < line_offsets.len()
                            {
                                let abs_cursor = line_offsets[cursor.line] + cursor.index;
                                // Check if cursor is strictly inside a glitch
                                if let Some((start, end)) = glitches
                                    .iter()
                                    .find(|(s, e)| abs_cursor >= *s && abs_cursor <= *e)
                                {
                                    let mut full_text = String::new();
                                    for (i, line) in buffer.lines.iter().enumerate() {
                                        if i > 0 {
                                            full_text.push('\n');
                                        }
                                        full_text.push_str(line.text());
                                    }
                                    if *end <= full_text.len() {
                                        let word = full_text[*start..*end].to_string();
                                        let suggestions = self.spell_checker.as_ref().unwrap().suggest(&word);
                                        found_word = Some((word, *start, *end, suggestions));
                                    }
                                }
                            }
                        });
                    }
                }

                if let Some(word_data) = found_word {
                    ui.data_mut(|d| d.insert_temp(target_word_id, word_data));
                } else {
                    ui.data_mut(|d| d.remove_temp::<(String, usize, usize, Vec<String>)>(target_word_id));
                }
            }
        }

        // 6. Final State Recording
        ui.data_mut(|d| {
            // Recalculate hash for final state (it might have changed during render/sync)
            let mut hasher = DefaultHasher::new();
            self.text.hash(&mut hasher);
            let final_hash = hasher.finish();

            d.insert_temp(last_model_hash_id, final_hash);
            // Store as plain String to match retrieval type
            d.insert_temp(last_search_query_id, current_query_str);
            d.insert_temp(last_font_family_id, current_font_str);
            d.insert_temp(
                ui.make_persistent_id(format!("{}_last_bright_mode", self.id)),
                self.bright_mode,
            );
        });

        editors.insert(self.id, cosmic_edit);

        response
    }
}
