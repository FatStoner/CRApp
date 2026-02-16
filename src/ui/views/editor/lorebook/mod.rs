use crate::ui::CrapApp;
use eframe::egui;

mod characters;
mod entries;
mod metadata;

use characters::render_lorebook_characters;
use entries::render_lorebook_entries;
use metadata::render_lorebook_metadata;

pub fn render_lorebook_editor(app: &mut CrapApp, ui: &mut egui::Ui) {
    if let Some(mut book) = app.selected_lorebook.take() {
        // --- SYNC START: Sync selected_entry back to book for accurate dirty check ---
        if let Some(selected) = &app.selected_entry {
            if let Some(existing) = book.entries.iter_mut().find(|e| e.id == selected.id) {
                *existing = selected.clone();
            }
        }
        // --- SYNC END ---

        let mut save_lore_req = None;
        let mut tag_add_request = None;
        let mut tag_remove_request = None;
        let mut entry_save_req = None;
        let mut back_history_req = false;
        let mut entry_add_req = false;
        let mut status_update: Option<(String, egui::Color32)> = None;

        // Check Dirty State
        let is_dirty = if book.id == 0 {
            true // Always dirty if new
        } else if let Some(original) = app.lorebooks.iter().find(|b| b.id == book.id) {
            !book.content_eq(original)
        } else {
            false
        };

        // Check for Ctrl+S (Global scope for editor)
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::S)) {
            // Sync before save!
            if let Some(selected) = &app.selected_entry {
                if let Some(existing) = book.entries.iter_mut().find(|e| e.id == selected.id) {
                    *existing = selected.clone();
                }
            }
            save_lore_req = Some(book.clone());
        }

        ui.horizontal(|ui| {
            let back_btn = ui.button("⬅ Back");
            if back_btn.clicked() {
                back_history_req = true;
            }
            back_btn.context_menu(|ui| {
                ui.label("Navigation History");
                ui.separator();
                let history_len = app.navigation_history.len();
                let start_index = history_len.saturating_sub(5);
                let history_items: Vec<(usize, String)> = app
                    .navigation_history
                    .iter()
                    .enumerate()
                    .skip(start_index)
                    .rev()
                    .map(|(i, state)| (i, app.describe_state(state)))
                    .collect();

                for (i, label) in history_items {
                    if ui.button(label).clicked() {
                        if is_dirty {
                            app.popup_state = crate::ui::PopupState::UnsavedChanges {
                                target: crate::ui::AppAction::GoToHistory(i),
                            };
                        } else {
                            app.go_to_history(i);
                        }
                        ui.close_menu();
                    }
                }
                if history_len == 0 {
                    ui.label(egui::RichText::new("No history").italics().weak());
                }
            });

            // Handle Esc key for Back navigation
            if ui.memory(|m| m.focused().is_none())
                && ui.input(|i| i.key_pressed(egui::Key::Escape))
            {
                back_history_req = true;
            }

            ui.heading("Edit Lorebook");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // EXPORT
                if ui.button("EXPORT").clicked() {
                    if let Some(result) = app.export_lorebook(&book) {
                        status_update = Some(result);
                    }
                }

                // IMPORT
                if ui.button("IMPORT").clicked() {
                    app.popup_state = crate::ui::PopupState::LorebookImport {
                        source_code: String::new(),
                        parsed_data: None,
                    };
                }

                // SAVE
                let mut save_color = ui.visuals().widgets.inactive.bg_fill;
                if is_dirty {
                    save_color = egui::Color32::from_rgb(200, 100, 50); // Orange/Red
                }

                if ui
                    .add(egui::Button::new(egui::RichText::new("SAVE").strong()).fill(save_color))
                    .clicked()
                {
                    // Sync before save!
                    if let Some(selected) = &app.selected_entry {
                        if let Some(existing) =
                            book.entries.iter_mut().find(|e| e.id == selected.id)
                        {
                            *existing = selected.clone();
                        }
                    }
                    save_lore_req = Some(book.clone());
                }

                // Status Notification
                if let Some((msg, color)) = &app.status_message {
                    if msg.contains("Saved") {
                        ui.label(egui::RichText::new(msg).color(*color).italics());
                    }
                }
            });
        });
        ui.add_space(4.0);

        // --- In-editor search ---
        ui.horizontal(|ui| {
            ui.label("🔍 Search:");
            let response = ui.add(
                egui::TextEdit::singleline(&mut app.editor_search_query)
                    .id_source("editor_search_field")
                    .hint_text("Type 3+ chars to highlight/jump...")
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

        // --- Auto-selection logic ---
        if app.editor_search_query.len() >= 3 {
            let query_lower = app.editor_search_query.to_lowercase();
            let metadata_match = book.title.to_lowercase().contains(&query_lower)
                || book.content.to_lowercase().contains(&query_lower);

            if !metadata_match {
                let current_match = if let Some(e) = &app.selected_entry {
                    e.name.to_lowercase().contains(&query_lower)
                        || e.keywords.to_lowercase().contains(&query_lower)
                        || e.content.to_lowercase().contains(&query_lower)
                } else {
                    false
                };

                if !current_match {
                    let matching_entry_idx = book.entries.iter().position(|e| {
                        e.name.to_lowercase().contains(&query_lower)
                            || e.keywords.to_lowercase().contains(&query_lower)
                            || e.content.to_lowercase().contains(&query_lower)
                    });

                    if let Some(idx) = matching_entry_idx {
                        if let Some(entry) = book.entries.get(idx).cloned() {
                            // SYNC BEFORE AUTO-SWITCH
                            if let Some(current) = &app.selected_entry {
                                if let Some(existing) =
                                    book.entries.iter_mut().find(|e| e.id == current.id)
                                {
                                    *existing = current.clone();
                                }
                            }

                            // Restore ownership temporarily for history
                            app.selected_lorebook = Some(book.clone());
                            app.push_history();
                            // Take it back
                            book = app.selected_lorebook.take().unwrap();

                            app.selected_entry = Some(entry);
                            app.active_lorebook_tab = crate::ui::LorebookTab::Entries;
                        }
                    }
                }
            }
        }

        render_lorebook_metadata(
            app,
            ui,
            &mut book,
            &mut status_update,
            &mut tag_add_request,
            &mut tag_remove_request,
        );

        ui.separator();

        // --- Tabs ---
        let linked_char_count = app
            .characters
            .iter()
            .filter(|c| {
                app.char_lore_map
                    .get(&c.id)
                    .map(|l| l.contains(&book.id))
                    .unwrap_or(false)
            })
            .count();

        ui.horizontal(|ui| {
            let entries_label = format!("Entries ({})", book.entries.len());
            if ui
                .selectable_label(
                    app.active_lorebook_tab == crate::ui::LorebookTab::Entries,
                    entries_label,
                )
                .clicked()
            {
                app.active_lorebook_tab = crate::ui::LorebookTab::Entries;
            }
            let chars_label = format!("Characters ({})", linked_char_count);
            if ui
                .selectable_label(
                    app.active_lorebook_tab == crate::ui::LorebookTab::Characters,
                    chars_label,
                )
                .clicked()
            {
                app.active_lorebook_tab = crate::ui::LorebookTab::Characters;
            }
        });
        ui.separator();

        match app.active_lorebook_tab {
            crate::ui::LorebookTab::Entries => {
                render_lorebook_entries(
                    app,
                    ui,
                    &mut book,
                    &mut entry_save_req,
                    &mut entry_add_req,
                    &mut status_update,
                );
            }
            crate::ui::LorebookTab::Characters => {
                // To avoid history issues if a character is opened, we'll handle ownership restoration inside mod.rs or child.
                // We'll restore it here temporarily before calling characters tab if we want to be safe,
                // but characters tab only restore it when 'OpenCharacter' is clicked.
                // Wait, it doesn't even HAVE a way to restore it easily.

                // Let's pass the book by value or just restore it right before the potential navigation.
                render_lorebook_characters(app, ui, &book);

                // If the characters tab triggered a navigation, app.selected_lorebook might be Some(book) or None.
                // Actually, if it triggered a navigation, we should probably return early.
            }
        }

        // --- Final Event Handling ---
        if let Some((msg, color)) = status_update {
            app.set_status(msg, color);
        }
        if let Some(l) = save_lore_req {
            app.save_lorebook(l);
        }
        if let Some((lid, name)) = tag_add_request {
            app.add_tag_to_lorebook(lid, name);
        }
        if let Some((lid, tid)) = tag_remove_request {
            app.remove_tag_from_lorebook(lid, tid);
        }

        if entry_add_req {
            app.add_entry_to_lorebook(book.id);
        }
        if let Some(e) = entry_save_req {
            app.save_lorebook_entry(e);
        }

        // Restore ownership
        if let Some(current) = &app.selected_entry {
            if let Some(existing) = book.entries.iter_mut().find(|e| e.id == current.id) {
                *existing = current.clone();
            }
        }
        if app.central_view == crate::ui::CentralView::Editor && app.selected_lorebook.is_none() {
            app.selected_lorebook = Some(book);
        }

        if back_history_req {
            app.request_back();
        }
    } else {
        ui.label("Select a lorebook.");
    }
}
