use eframe::egui;
use egui_cosmic_text::{
    atlas::TextureAtlas,
    cosmic_text::{Attrs, Color, Cursor, Edit, Family, FontSystem, Selection, Shaping, SwashCache},
    widget::{CosmicEdit, EditorActions, FillWidth, HoverStrategy, Interactivity, LayoutMode, LineHeight},
};
use std::collections::HashMap;

pub struct CodeEditor<'a> {
    text: &'a mut String,
    id: String,
    desired_lines: usize,
    is_single_line: bool,
    search_query: Option<String>,
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
                        eprintln!("[CodeEditor] [ID: {}] Menu Paste: Inserting {} chars", self.id, text.chars().count());
                        editor.insert_string(text, font_system);
                        actions.scroll_to_cursor = true;
                        actions.focus = true;
                        *self.force_sync = true;
                    } else {
                        eprintln!("[CodeEditor] [ID: {}] Menu Paste: Clipboard text is empty", self.id);
                    }
                }
                Err(e) => {
                    eprintln!("[CodeEditor] [ID: {}] Menu Paste: Clipboard error: {:?}", self.id, e);
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
    pub fn new(text: &'a mut String, id: impl Into<String>) -> Self {
        Self {
            text,
            id: id.into(),
            desired_lines: 10,
            is_single_line: false,
            search_query: None,
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

    fn build_spans<'b>(
        &self, 
        text: &'b str, 
        default_attrs: Attrs<'b>, 
        highlight_attrs: Attrs<'b>
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
        let egui_text_color = visuals.text_color();
        let cosmic_color = Color::rgba(
            egui_text_color.r(),
            egui_text_color.g(),
            egui_text_color.b(),
            egui_text_color.a(),
        );

        let font_size = ui
            .style()
            .text_styles
            .get(&egui::TextStyle::Body)
            .map(|id| id.size)
            .unwrap_or(14.0);
        let line_height_val = font_size * 1.4;

        let default_attrs = Attrs::new().family(Family::SansSerif).color(cosmic_color);
        
        // Highlight attributes: Gold/Orange for visibility + Bold
        let highlight_color = Color::rgba(255, 215, 0, 255);
        let highlight_attrs = Attrs::new()
            .family(Family::SansSerif)
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
            widget.set_text(
                spans,
                default_attrs,
                Shaping::Advanced,
                font_system,
            );
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
        
        let last_model_text: Option<String> = ui.data(|d| d.get_temp(last_model_text_id));
        let last_search_query: Option<String> = ui.data(|d| d.get_temp(last_search_query_id));

        // Use string comparison for query to avoid TypeId issues with Option<String> persistence
        let current_query_str = self.search_query.as_deref().unwrap_or("");
        let last_query_str = last_search_query.as_deref().unwrap_or("");

        let query_changed = last_query_str != current_query_str;
        let text_changed = last_model_text.as_ref() != Some(self.text);

        // detect external change or query change
        if text_changed || query_changed {
            if query_changed {
                 ui.ctx().request_repaint();
            }

            let spans = self.build_spans(self.text.as_str(), default_attrs, highlight_attrs);
            cosmic_edit.set_text(
                spans,
                default_attrs, 
                Shaping::Advanced, 
                font_system
            );
            
            if text_changed {
                let mut editor = cosmic_edit.into_editor();
                editor.set_cursor(Cursor::new(0, 0));
                editor.set_selection(Selection::None);
                
                cosmic_edit = CosmicEdit::from_editor(
                    editor,
                    Interactivity::Enabled,
                    HoverStrategy::Widget,
                    FillWidth::default(),
                );
            }
        }

        let mut force_sync_back = false;

        // 4. Render Layout
        let response = ui.push_id(&self.id, |ui| {
            egui::Frame::canvas(ui.style())
                .fill(ui.visuals().extreme_bg_color)
                .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                .rounding(4.0)
                .inner_margin(egui::Margin::same(6.0))
                .show(ui, |ui| {
                    let min_height = self.desired_lines as f32 * line_height_val;
                    ui.set_min_height(min_height);

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

                    // Robust Keyboard Overrides
                    if resp.has_focus() {
                        ui.input_mut(|i| {
                            // Single Line Mode: Intercept Enter
                            if self.is_single_line && i.consume_key(egui::Modifiers::NONE, egui::Key::Enter) {
                                resp.surrender_focus();
                            }

                            // Manual Paste High-Reliability Bypass
                            if i.consume_key(egui::Modifiers::COMMAND, egui::Key::V) {
                                if let Ok(text) = clipboard.get_text() {
                                    if !text.is_empty() {
                                        // Filter newlines if single line
                                        let text = if self.is_single_line {
                                            text.replace('\n', " ").replace('\r', "")
                                        } else {
                                            text
                                        };
                                        eprintln!("[CodeEditor] [ID: {}] Key Paste: Inserting {} chars", self.id, text.chars().count());
                                        cosmic_edit.insert_string(text, font_system);
                                        force_sync_back = true;
                                    }
                                }
                            }
                            // Detect Ctrl+Z/Y - force sync back
                            if i.key_pressed(egui::Key::Z) && i.modifiers.command {
                                force_sync_back = true;
                            }
                            if i.key_pressed(egui::Key::Y) && i.modifiers.command {
                                force_sync_back = true;
                            }
                        });
                    }

                    // 5. Internal -> External Sync
                    if cosmic_edit.changed_this_frame() || force_sync_back {
                        let new_text = cosmic_edit.text();
                        let mut clean_new = new_text.trim_end_matches('\n').to_string();
                        
                        // Enforce single line
                        if self.is_single_line && clean_new.contains('\n') {
                            clean_new = clean_new.replace('\n', "");
                        }

                        if *self.text != clean_new {
                            // eprintln!("[CodeEditor] [ID: {}] Syncing Back.", self.id);
                            *self.text = clean_new;
                        }
                    }

                    if resp.clicked() {
                        resp.request_focus();
                    }

                    resp
                })
                .inner
        })
        .inner;

        // 6. Final State Recording
        ui.data_mut(|d| {
            d.insert_temp(last_model_text_id, self.text.clone());
            // Store as plain String to match retrieval type
            d.insert_temp(last_search_query_id, current_query_str.to_string());
        });
        
        editors.insert(self.id, cosmic_edit);

        response
    }
}
