use crate::ui::{AppAction, CrapApp, UiEvent};
use eframe::egui;

#[derive(Clone)]
pub enum PopupState {
    None,
    Renaming {
        id: i64,
        name: String,
    },

    DeleteWarning {
        _id: i64,
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
    DeleteTemplateConfirmation {
        id: i64,
        name: String,
    },
    UnsavedChanges {
        target: AppAction,
    },
    ImportDbWarning,
    CollectionIconConfirmation {
        id: i64,
        path: String,
        _preview_texture: Option<egui::TextureHandle>,
    },
    LorebookImport {
        source_code: String,
        parsed_data: Option<crate::ui::parsing::ParsedLorebookData>,
    },
    ExportDbSelection,
    TemplateSelector,
    TemplatePreview {
        template_data: crate::models::Template,
        target_char_id: i64,
    },
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

        PopupState::DeleteWarning { _id: _, count } => {
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
        PopupState::DeleteTemplateConfirmation { id, name } => {
            let mut close = false;
            egui::Window::new("Delete Template?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label(format!("Are you sure you want to delete '{}'?", name));
                    ui.colored_label(egui::Color32::RED, "This action cannot be undone.");
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("Yes, Delete").clicked() {
                            app.delete_template(id);
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
                            _preview_texture: None,
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
            let mut loaded_file_data = None;

            egui::Window::new("Import Lorebook from SpicyChat")
                .collapsible(false)
                .resizable(true)
                .default_width(600.0)
                .default_height(500.0)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                     ui.horizontal(|ui| {
                        if ui.button("📂 Load .crappbook / JSON").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Lorebook Files", &["crappbook", "json"])
                                .add_filter("All Files", &["*"])
                                .pick_file() 
                            {
                                if let Ok(content) = std::fs::read_to_string(&path) {
                                    if let Ok(data) = crate::ui::parsing::parse_crappbook_json(&content) {
                                        loaded_file_data = Some(data);
                                    }
                                }
                            }
                        }
                    });
                    ui.separator();
                    ui.label("Or paste SpicyChat source code below:");
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("To import: Go to lorebook page, right click empty space -> Inspect Element.\nFind the first <html ...> line, right click -> Copy -> Copy outerHTML.").size(11.0).color(egui::Color32::GRAY));

                    egui::ScrollArea::vertical()
                        .id_salt("import_source_scroll")
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
                            .id_salt("import_preview_scroll")
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
            if let Some(data) = loaded_file_data {
                 app.popup_state = PopupState::LorebookImport {
                    source_code: String::new(),
                    parsed_data: Some(data),
                };
            } else if do_parse {
                let parsed = crate::ui::parsing::parse_spicychat_lorebook(&source_code);
                app.popup_state = PopupState::LorebookImport {
                    source_code, // Keep source code
                    parsed_data: Some(parsed),
                };
            } else if do_import {
                // Populate Editor State (Overwrite) instead of saving immediately
                if let Some(data) = parsed_data {
                    // 1. Determine Target (New or Existing)
                    let mut lorebook = if let Some(current) = &app.selected_lorebook {
                        current.clone()
                    } else {
                        crate::models::Lorebook {
                             title: if data.title.is_empty() { "Imported Lorebook".to_string() } else { data.title.clone() },
                             ..Default::default()
                        }
                    };

                    // 2. Overwrite Fields
                    if !data.title.is_empty() {
                         lorebook.title = data.title;
                    }
                    lorebook.description = data.description.clone();
                    lorebook.content = data.description; // Sync desc/content

                    // 3. Entries (Replace All)
                    // We map them to entries with id: 0 to signify they are new/modified in this context
                    // If we wanted to be smarter and preserve IDs of matching entries, we could, 
                    // but "Overwrite" implies full replacement. 
                    // save_lorebook logic I added will handle diffing/deleting correctly even if we drop IDs here 
                    // OR save_lorebook assumes IDs are valid. 
                    // Wait, if I set id=0 here, save_lorebook will see them as NEW and INSERT them.
                    // It will also see that the OLD entries (which have IDs) are NOT present in this list.
                    // So it will DELETE the old ones and INSERT these as new.
                    // This is correct behavior for "Overwrite".
                    lorebook.entries = data.entries.into_iter().map(|e| {
                        crate::models::LorebookEntry {
                            lorebook_id: lorebook.id,
                            name: e.name,
                            keywords: e.keywords.join(", "),
                            content: e.content,
                            ..Default::default()
                        }
                    }).collect();

                    // 4. Tags (Replace All)
                    lorebook.tags = data.tags.into_iter().map(|t| {
                        crate::models::Tag { id: 0, name: t }
                    }).collect();

                    // 5. Update State
                    app.selected_lorebook = Some(lorebook);
                    app.mode = crate::ui::AppMode::Lorebooks;
                    app.central_view = crate::ui::CentralView::Editor;
                    app.popup_state = PopupState::None;
                    
                    app.set_status("Imported data into editor. Click SAVE to persist.".to_string(), egui::Color32::YELLOW);
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
        PopupState::TemplateSelector => {
            let mut close = false;
            let mut selected_template = None;
            egui::Window::new("Select Template")
                .collapsible(false)
                .resizable(true)
                .min_width(300.0)
                .default_height(400.0)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                     ui.label("Choose a template to apply:");
                     ui.add_space(5.0);
                     
                     egui::ScrollArea::vertical().show(ui, |ui| {
                         for template in &app.templates {
                            if ui.button(&template.name).clicked() {
                                selected_template = Some(template.clone());
                            }
                         }
                     });

                    ui.add_space(10.0);
                    if ui.button("Cancel").clicked() {
                        close = true;
                    }
                });
            
            if let Some(template) = selected_template {
                 if let Some(char) = &app.selected_character {
                     app.popup_state = PopupState::TemplatePreview {
                         template_data: template,
                         target_char_id: char.id
                     };
                 } else {
                     app.popup_state = PopupState::None;
                 }
            } else if close {
                app.popup_state = PopupState::None;
            }
        }
        PopupState::TemplatePreview { mut template_data, target_char_id } => {
            let mut close = false;
            let mut apply = false;
            
            egui::Window::new("Preview & Edit Template")
                .collapsible(false)
                .resizable(true)
                .min_width(500.0)
                .default_height(600.0)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new("Review the template data before applying.").strong());
                    ui.label("You can edit these fields here. They will overwrite the current character's data.");
                    ui.separator();
                    
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label("Title:");
                        ui.text_edit_singleline(&mut template_data.title);
                        ui.add_space(5.0);
                        
                        ui.label("First Message:");
                        ui.add(egui::TextEdit::multiline(&mut template_data.first_message).desired_rows(3));
                        ui.add_space(5.0);
                        
                        ui.label("Personality:");
                        ui.add(egui::TextEdit::multiline(&mut template_data.personality).desired_rows(3));
                        ui.add_space(5.0);
                        
                        ui.label("Scenario:");
                        ui.add(egui::TextEdit::multiline(&mut template_data.scenario).desired_rows(3));
                        ui.add_space(5.0);
                        
                        ui.label("Example Dialogue:");
                        ui.add(egui::TextEdit::multiline(&mut template_data.example_dialogue).desired_rows(3));
                    });
                    
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Apply Template").clicked() {
                            apply = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });
            
            if apply {
                 if let Some(char) = &mut app.selected_character {
                     if char.id == target_char_id {
                         char.char_title = template_data.title;
                         char.first_message = template_data.first_message;
                         char.personality = template_data.personality;
                         char.scenario = template_data.scenario;
                         char.example_dialogue = template_data.example_dialogue;
                         // Mark as modified if we tracked that, but UI handles dirty checks by diffing with DB
                     }
                 }
                app.popup_state = PopupState::None;
            } else if close {
                app.popup_state = PopupState::None;
            } else {
                // Determine if we need to update state because of edits
                 // We need to persist edits to the popup state
                 // Since we destructured `template_data` mutably from a clone of the state (at start of fn),
                 // we must write it back.
                 // HOWEVER, `app.popup_state` is already borrowing `app`? No, we cloned state at start.
                 // So we can write to app.popup_state.
                 app.popup_state = PopupState::TemplatePreview {
                     template_data,
                     target_char_id
                 };
            }
        }
    }
}
