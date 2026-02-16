use crate::ui::{CharacterTab, CrapApp};
use eframe::egui;

mod gallery;
mod lorebooks;
mod main_data;
mod notes;

use gallery::render_gallery_tab;
use lorebooks::render_lorebooks_tab;
use main_data::render_main_data_tab;
use notes::render_notes_tab;

pub fn render_editor_view(app: &mut CrapApp, ui: &mut egui::Ui) {
    let mut trigger_import = false;

    let mut save_req = None;
    let mut toggle_requests = Vec::<(i64, i64, bool)>::new();
    let mut status_update = None;
    let mut back_history_req = false;
    let mut back_req = None;

    // Prepare collection options
    let collection_options: Vec<(i64, String)> = app
        .collections
        .iter()
        .map(|c| (c.id, app.get_collection_path(c.id)))
        .collect();

    // Take ownership of selected_character to allow mutable access to it AND app simultaneously
    if let Some(mut character) = app.selected_character.take() {
        // Helper for dirty check locally
        let is_dirty = if character.id == 0 {
            true
        } else {
            if let Some(original) = app.characters.iter().find(|c| c.id == character.id) {
                !character.content_eq(original)
            } else {
                true
            }
        };

        // Render toolbar and handle its actions
        let toolbar_action =
            super::toolbar::render_toolbar(ui, app, &character, is_dirty, &mut trigger_import);

        if toolbar_action.template_requested {
            app.popup_state = crate::ui::PopupState::TemplateSelector;
        }
        if toolbar_action.save_requested {
            save_req = Some(character.clone());
        }
        if toolbar_action.back_history_requested {
            back_history_req = true;
        }
        if let Some(target) = toolbar_action.back_to_collection {
            back_req = Some(target);
        }

        ui.horizontal(|ui| {
            ui.label("Collection:");
            let current_col_name = character
                .collection_id
                .and_then(|id| {
                    collection_options
                        .iter()
                        .find(|(cid, _)| *cid == id)
                        .map(|(_, name)| name.clone())
                })
                .unwrap_or_else(|| "Uncategorized".to_string());

            egui::ComboBox::from_id_source("collection_combo")
                .selected_text(current_col_name)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut character.collection_id, None, "Uncategorized");
                    for (id, name) in &collection_options {
                        ui.selectable_value(&mut character.collection_id, Some(*id), name);
                    }
                });

            ui.add_space(8.0);
            let fav_btn = if character.is_favorite {
                egui::Button::new(
                    egui::RichText::new("\u{2764} Favorite").color(egui::Color32::WHITE),
                )
                .fill(egui::Color32::from_rgb(200, 50, 50))
            } else {
                egui::Button::new("\u{2764} Favorite")
            };

            if ui.add(fav_btn).clicked() {
                character.is_favorite = !character.is_favorite;
            }
        });

        // In-editor search
        ui.horizontal(|ui| {
            ui.label("🔍 Search:");
            let response = ui.add(
                egui::TextEdit::singleline(&mut app.editor_search_query)
                    .id_source("editor_search_field")
                    .hint_text("Type 3+ chars to highlight...")
                    .desired_width(200.0),
            );

            if app.focus_search_field {
                response.request_focus();
                app.focus_search_field = false;
            }

            if !app.editor_search_query.is_empty() {
                if ui.small_button("✖").clicked() {
                    app.editor_search_query.clear();
                }

                ui.label(
                    egui::RichText::new(format!("Highlighting: '{}'", app.editor_search_query))
                        .size(11.0)
                        .color(egui::Color32::GRAY),
                );
            }
        });
        ui.separator();

        // Tabs
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(
                &mut app.active_char_tab,
                CharacterTab::MainData,
                "Main Data",
            );
            ui.selectable_value(&mut app.active_char_tab, CharacterTab::Notes, "Notes");
            ui.selectable_value(
                &mut app.active_char_tab,
                CharacterTab::Lorebooks,
                "Lorebooks",
            );
            ui.selectable_value(&mut app.active_char_tab, CharacterTab::Gallery, "Gallery");
        });
        ui.separator();

        // Handle Drag and Drop for Avatar
        let dropped_files = ui.input(|i| i.raw.dropped_files.clone());
        if !dropped_files.is_empty() {
            for dropped in dropped_files {
                if let Some(path) = dropped.path {
                    let ext = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    if ["png", "jpg", "jpeg", "webp"].contains(&ext.as_str()) {
                        let dest_dir = std::path::Path::new("data/avatars");
                        let _ = std::fs::create_dir_all(dest_dir);
                        if let Some(name) = path.file_name() {
                            let dest = dest_dir.join(name);
                            if let Ok(_) = std::fs::copy(&path, &dest) {
                                character.avatar_path = Some(dest.to_string_lossy().to_string());
                                status_update = Some((
                                    "Avatar loaded from dropped file!".to_string(),
                                    egui::Color32::GREEN,
                                ));
                            }
                        }
                    }
                }
            }
        }

        let mut tag_add_request: Option<(i64, String, bool)> = None;
        let mut tag_remove_request: Option<(i64, i64, bool)> = None;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| match app.active_char_tab {
                CharacterTab::MainData => {
                    render_main_data_tab(
                        app,
                        ui,
                        &mut character,
                        &mut status_update,
                        &mut tag_add_request,
                        &mut tag_remove_request,
                    );
                }
                CharacterTab::Notes => {
                    render_notes_tab(app, ui, &mut character);
                }
                CharacterTab::Lorebooks => {
                    render_lorebooks_tab(app, ui, &mut character, &mut toggle_requests);
                }
                CharacterTab::Gallery => {
                    render_gallery_tab(app, ui, &mut character, &mut status_update);
                }
            });

        if let Some((msg, color)) = status_update {
            app.set_status(msg, color);
        }

        // Handle events
        if trigger_import {
            app.show_import_modal = true;
            app.import_text.clear();
            app.parsed_data = None;
        }

        // Execute deferred tag operations
        if let Some((cid, name, is_ext)) = tag_add_request {
            app.add_tag(cid, name, is_ext);
        }
        if let Some((cid, tid, is_ext)) = tag_remove_request {
            app.remove_tag(cid, tid, is_ext);
        }

        // Restore ownership
        if app.central_view == crate::ui::CentralView::Editor && app.selected_character.is_none() {
            app.selected_character = Some(character);
        }
        if back_history_req {
            app.request_back();
        }
    } else {
        ui.label("Select a character to edit.");
    }

    // Process Toggle Requests
    for (cid, lid, linked) in toggle_requests {
        app.toggle_lore_link(cid, lid, linked);
    }

    if let Some(c) = save_req {
        app.save_character(c);
    }

    if let Some(target) = back_req {
        app.request_collection_switch(target);
    }
}
