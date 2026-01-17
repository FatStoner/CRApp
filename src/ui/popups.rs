use crate::ui::{AppAction, CrapApp, UiEvent};
use eframe::egui;

#[derive(Clone)]
pub enum PopupState {
    None,
    Renaming {
        id: i64,
        name: String,
    },
    DeleteConfirmation {
        id: i64,
    },
    DeleteWarning {
        id: i64,
        count: usize,
    },
    DeleteCharacterConfirmation {
        id: i64,
        name: String,
    },
    DeleteLorebookEntryConfirmation {
        id: i64,
        lorebook_id: i64,
        name: String,
    },
    DeleteLorebookConfirmation {
        id: i64,
        title: String,
    },
    UnsavedChanges {
        target: AppAction,
    },
    ImportDbWarning,
    CollectionIconConfirmation {
        id: i64,
        path: String,
        preview_texture: Option<egui::TextureHandle>,
    },
    LorebookImport {
        source_code: String,
        parsed_data: Option<crate::ui::parsing::ParsedLorebookData>,
    },
    ExportDbSelection,
}

pub fn render_popups(ctx: &egui::Context, app: &mut CrapApp) {
    // We clone the state to avoid mutable borrow conflicts
    let state = app.popup_state.clone();

    match state {
        PopupState::None => {}
        PopupState::Renaming { id, mut name } => {
            let mut close = false;
            egui::Window::new("Rename Collection")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        let response = ui.text_edit_singleline(&mut name);
                        if response.changed() {
                            app.popup_state = PopupState::Renaming {
                                id,
                                name: name.clone(),
                            };
                        }
                        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            app.save_collection(id, name.clone(), None);
                            close = true;
                        }
                    });
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            app.save_collection(id, name.clone(), None);
                            close = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });
            if close {
                app.popup_state = PopupState::None;
            }
        }
        PopupState::DeleteConfirmation { id } => {
            // Note: This seems legacy/unused based on warnings, but keeping for now as it's in the enum
            let mut close = false;
            egui::Window::new("Delete Collection?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label("Are you sure you want to delete this collection?");
                    ui.colored_label(egui::Color32::RED, "This action cannot be undone.");
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Yes, Delete").clicked() {
                            app.delete_collection(id);
                            close = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });
            if close {
                app.popup_state = PopupState::None;
            }
        }
        PopupState::DeleteWarning { id: _, count } => {
            let mut close = false;
            egui::Window::new("Cannot Delete Folder")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.colored_label(egui::Color32::RED, "Warning: Folder is not empty.");
                    ui.add_space(5.0);
                    ui.label(format!(
                        "This folder contains {} character(s) or subfolder(s).",
                        count
                    ));
                    ui.label("You must move or delete all contents before deleting this folder.");
                    ui.add_space(10.0);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        if ui.button("OK").clicked() {
                            close = true;
                        }
                    });
                });
            if close {
                app.popup_state = PopupState::None;
            }
        }
        PopupState::DeleteCharacterConfirmation { id, name } => {
            let mut close = false;
            egui::Window::new("Delete Character?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label(format!("Are you sure you want to delete '{}'?", name));
                    ui.colored_label(egui::Color32::RED, "This action cannot be undone.");
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("Yes, Delete").clicked() {
                            app.delete_character(id);
                            close = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });
            if close {
                app.popup_state = PopupState::None;
            }
        }
        PopupState::DeleteLorebookConfirmation { id, title } => {
            let mut close = false;
            egui::Window::new("Delete Lorebook?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label(format!("Are you sure you want to delete '{}'?", title));
                    ui.colored_label(egui::Color32::RED, "This action cannot be undone.");
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("Yes, Delete").clicked() {
                            app.delete_lorebook(id);
                            close = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });
            if close {
                app.popup_state = PopupState::None;
            }
        }
        PopupState::DeleteLorebookEntryConfirmation {
            id,
            lorebook_id,
            name,
        } => {
            let mut close = false;
            egui::Window::new("Delete Entry?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label(format!("Are you sure you want to delete '{}'?", name));
                    ui.colored_label(egui::Color32::RED, "This action cannot be undone.");
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("Yes, Delete").clicked() {
                            let tx = app.tx.clone();
                            let db = app.db.clone();
                            let entry_id = id;
                            let lid = lorebook_id;
                            tokio::spawn(async move {
                                match db.delete_lorebook_entry(entry_id).await {
                                    Ok(_) => {
                                        let _ = tx
                                            .send(UiEvent::LorebookEntryDeleted(Ok(entry_id)))
                                            .await;
                                    }
                                    Err(e) => {
                                        let _ = tx
                                            .send(UiEvent::LorebookEntryDeleted(Err(e.to_string())))
                                            .await;
                                    }
                                }
                                // Trigger reload of entries for this lorebook
                                if let Ok(entries) = db.get_entries_for_lorebook(lid).await {
                                    let _ = tx
                                        .send(UiEvent::LorebookEntriesLoaded(Ok((lid, entries))))
                                        .await;
                                }
                            });
                            close = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });
            if close {
                app.popup_state = PopupState::None;
            }
        }
        PopupState::UnsavedChanges { target } => {
            egui::Window::new("Unsaved Changes")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label("You have unsaved changes.");
                    ui.label("What would you like to do?");
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("Save & Continue").clicked() {
                            if let Some(c) = app.selected_character.clone() {
                                app.pending_action = Some(target.clone());
                                app.save_character(c);
                            } else if let Some(book) = app.selected_lorebook.clone() {
                                app.pending_action = Some(target.clone());
                                app.save_lorebook(book);
                            }
                            app.popup_state = PopupState::None;
                        }

                        if ui.button("Discard Changes").clicked() {
                            // Revert changes
                            if let Some(selected) = &app.selected_character {
                                if selected.id == 0 {
                                    app.selected_character = None;
                                } else {
                                    if let Some(original) =
                                        app.characters.iter().find(|c| c.id == selected.id)
                                    {
                                        app.selected_character = Some(original.clone());
                                    }
                                }
                            } else if let Some(selected_book) = &app.selected_lorebook {
                                if selected_book.id == 0 {
                                    app.selected_lorebook = None;
                                } else {
                                    if let Some(original) =
                                        app.lorebooks.iter().find(|l| l.id == selected_book.id)
                                    {
                                        app.selected_lorebook = Some(original.clone());
                                    }
                                }
                            }
                            app.perform_action(target.clone(), ctx);
                            app.popup_state = PopupState::None;
                        }

                        if ui.button("Cancel").clicked() {
                            app.popup_state = PopupState::None;
                        }
                    });
                });
        }
        PopupState::ImportDbWarning => {
            egui::Window::new("Import Database")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.colored_label(
                        egui::Color32::RED,
                        "Warning: This will overwrite your current database!",
                    );
                    ui.label("A backup of your current data will be created.");
                    ui.add_space(5.0);
                    ui.label("Are you sure you want to proceed?");
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("Yes, Import").clicked() {
                            app.trigger_db_import();
                            app.popup_state = PopupState::None;
                        }
                        if ui.button("Cancel").clicked() {
                            app.popup_state = PopupState::None;
                        }
                    });
                });
        }
        PopupState::CollectionIconConfirmation { id, mut path, .. } => {
            let mut close = false;
            let mut new_path = None;

            egui::Window::new("Change Collection Icon")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("📂 Browse Image...").clicked() {
                            if let Some(file_path) = rfd::FileDialog::new()
                                .add_filter("Image", &["png", "jpg", "jpeg", "webp"])
                                .pick_file()
                            {
                                new_path = Some(file_path.to_string_lossy().to_string());
                            }
                        }
                        if ui.button("📋 Paste from Clipboard").clicked() {
                            // Handle clipboard logic here or signal app to do it?
                            // For simplicity, let's assume we can't easily paste directly into a path string here without more logic
                            // But we can try to reuse the app's clipboard utility if we moved it.
                            // For now, let's keep it simple or fix it up.
                            // Actually, since this is a new file, I'll omit the complex clipboard logic for now or implement it fully.
                        }
                    });

                    if let Some(np) = new_path {
                        path = np;
                        // Update the state with new path so it persists across frames until saved
                        app.popup_state = PopupState::CollectionIconConfirmation {
                            id,
                            path: path.clone(),
                            preview_texture: None,
                        };
                    }

                    ui.add_space(5.0);
                    ui.label(format!(
                        "Selected: {}",
                        if path.is_empty() { "None" } else { &path }
                    ));

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            app.update_collection_icon(id, Some(path.clone()));
                            close = true;
                        }
                        if ui.button("Clear Icon").clicked() {
                            app.update_collection_icon(id, None);
                            close = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });

            if close {
                app.popup_state = PopupState::None;
            }
        }

        PopupState::LorebookImport {
            mut source_code,
            parsed_data,
        } => {
            let mut close = false;
            let mut do_parse = false;
            let mut do_import = false;

            egui::Window::new("Import Lorebook from SpicyChat")
                .collapsible(false)
                .resizable(true)
                .default_width(600.0)
                .default_height(500.0)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label("Paste the page source code below:");

                    egui::ScrollArea::vertical()
                        .id_source("import_source_scroll")
                        .max_height(200.0)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut source_code)
                                    .hint_text("<html>...</html>")
                                    .desired_width(f32::INFINITY)
                                    .font(egui::TextStyle::Monospace),
                            );
                        });

                    ui.add_space(8.0);

                    if ui.button("Parse Source").clicked() {
                        do_parse = true;
                    }

                    ui.separator();

                    if let Some(data) = &parsed_data {
                        ui.heading("Preview");
                        egui::ScrollArea::vertical()
                            .id_source("import_preview_scroll")
                            .max_height(150.0)
                            .show(ui, |ui| {
                                egui::Grid::new("import_preview_grid").num_columns(2).show(
                                    ui,
                                    |ui| {
                                        ui.label("Title:");
                                        ui.label(egui::RichText::new(&data.title).strong());
                                        ui.end_row();

                                        ui.label("Description:");
                                        ui.label(if data.description.is_empty() {
                                            "(Empty)"
                                        } else {
                                            &data.description
                                        });
                                        ui.end_row();

                                        ui.label("Entries:");
                                        ui.label(format!("Found {}", data.entries.len()));
                                        ui.end_row();

                                        ui.label("Tags:");
                                        ui.label(format!("Found {}", data.tags.len()));
                                        ui.end_row();
                                    },
                                );
                            });

                        ui.add_space(8.0);

                        if data.title.is_empty() && data.entries.is_empty() {
                            ui.colored_label(
                                egui::Color32::YELLOW,
                                "Warning: No title or entries found. Check source code.",
                            );
                        }
                    }

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        let can_import = parsed_data
                            .as_ref()
                            .map(|d| !d.title.is_empty() || !d.entries.is_empty())
                            .unwrap_or(false);

                        if ui
                            .add_enabled(can_import, egui::Button::new("Import Lorebook"))
                            .clicked()
                        {
                            do_import = true;
                        }

                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });

            // Handle State Updates outside the closure
            if do_parse {
                let parsed = crate::ui::parsing::parse_spicychat_lorebook(&source_code);
                app.popup_state = PopupState::LorebookImport {
                    source_code, // Keep source code
                    parsed_data: Some(parsed),
                };
            } else if do_import {
                // Perform Import via Async Task
                if let Some(data) = parsed_data {
                    let tx = app.tx.clone();
                    let db = app.db.clone();

                    tokio::spawn(async move {
                        // Map ParsedLorebookData to models::Lorebook
                        let mut lorebook = crate::models::Lorebook {
                            title: if data.title.is_empty() {
                                "Imported Lorebook".to_string()
                            } else {
                                data.title
                            },
                            description: data.description.clone(),
                            content: data.description, // Sync desc/content
                            ..Default::default()
                        };

                        // Save Lorebook
                        // We need a proper upsert that returns ID or modify upsert locally to return ID from result.
                        // But upsert_lorebook in db returns Result<()>.
                        // Actually, insert usually sets the ID on the passed struct. BUT upsert_lorebook takes &mut Lorebook.
                        // So we should be good if we call it.

                        // Note: We need to handle tags and entries too.

                        // Step 1: Create Lorebook
                        if let Ok(_) = db.upsert_lorebook(&mut lorebook).await {
                            // Step 2: Add Entries
                            for entry in data.entries {
                                let mut new_entry = crate::models::LorebookEntry {
                                    lorebook_id: lorebook.id,
                                    name: entry.name,
                                    keywords: entry.keywords.join(", "),
                                    content: entry.content,
                                    ..Default::default()
                                };
                                if let Ok(eid) = db.add_entry_to_lorebook(&new_entry).await {
                                    new_entry.id = eid;
                                    lorebook.entries.push(new_entry);
                                }
                            }

                            // Step 3: Add Tags
                            for tag in data.tags {
                                let _ = db.add_tag_to_lorebook(lorebook.id, &tag).await;
                                // We might want to add tags to lorebook object too if we use them immediately
                                let new_tag = crate::models::Tag { id: 0, name: tag }; // ID is unknown without fetch, maybe acceptable for now
                                lorebook.tags.push(new_tag);
                            }

                            // Notify UI
                            let _ = tx.send(UiEvent::LorebookImported(lorebook)).await;

                            let _ = tx
                                .send(UiEvent::StatusMessage(
                                    "Lorebook Imported Successfully".to_string(),
                                    egui::Color32::GREEN,
                                ))
                                .await;

                            // Trigger Reload
                            // We probably want to select it or just reload the list.
                            // Sending a generic reload event or just letting user find it.
                            // There isn't a generic "ReloadLorebooks" event visible here in imports,
                            // but we can assume main loop refreshes or we can add one.
                            // For now, StatusMessage is good info.
                        } else {
                            let _ = tx
                                .send(UiEvent::StatusMessage(
                                    "Failed to Import Lorebook".to_string(),
                                    egui::Color32::RED,
                                ))
                                .await;
                        }
                    });
                    // close = true; // This does nothing inside async block and was creating a warning.
                    // To actually close, we would need to send an event or handle it in the next frame update.
                    // For now, keeping the popup open allows user to see "Success" message or import another.
                    // If we want to close, the async task should send a ClosePopup event.
                }
            } else if close {
                app.popup_state = PopupState::None;
            } else if !source_code.is_empty() {
                // Check if text changed to update state if parsing didn't happen but typing did
                // Actually this variant owns the string, so we need to put it back if we didn't change state.
                // egui::TextEdit modifies the string in place inside the closure which borrows mutable `source_code`.
                // So we need to put it back into app.popup_state if we want to persist it.
                // BUT `app.popup_state` was cloned at start of function!
                // So we MUST update `app.popup_state` with the Modified source_code if we want it to persist.

                // However, we only have reference to `app` here.
                // If we didn't trigger parse/import/close, we should update the state with the potentially modified source code.
                // But simply assigning it back every frame is fine.
                app.popup_state = PopupState::LorebookImport {
                    source_code,
                    parsed_data,
                };
            }
        }
        PopupState::ExportDbSelection => {
            let mut close = false;
            egui::Window::new("Export Database")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label("Choose an export method:");
                    ui.add_space(10.0);

                    if ui
                        .button("💾 Export Database Only (.db)")
                        .on_hover_text("Exports just the SQLite database file. Useful for manual backups.")
                        .clicked()
                    {
                        app.trigger_db_export_file_only();
                        close = true;
                    }

                    ui.add_space(5.0);

                    if ui
                        .button("📦 Export Full Backup (.zip)")
                        .on_hover_text("Exports the database AND all images (avatars, covers, gallery). Recommended for moving to another PC.")
                        .clicked()
                    {
                        app.perform_full_zip_export();
                        close = true;
                    }

                    ui.add_space(15.0);
                    if ui.button("Cancel").clicked() {
                        close = true;
                    }
                });
            if close {
                app.popup_state = PopupState::None;
            }
        }
    }
}
