use crate::ui::CrapApp;
use eframe::egui;

/// Render editing/modification popups
pub fn render_editing_popups(ctx: &egui::Context, app: &mut CrapApp, state: &super::PopupState) {
    match state {
        super::PopupState::Renaming { id, name } => {
            let mut name = name.clone();
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
                            app.popup_state = super::PopupState::Renaming {
                                id: *id,
                                name: name.clone(),
                            };
                        }
                        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            app.save_collection(*id, name.clone(), None);
                            close = true;
                        }
                    });
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            app.save_collection(*id, name.clone(), None);
                            close = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });
            if close {
                app.popup_state = super::PopupState::None;
            }
        }

        super::PopupState::UnsavedChanges { target } => {
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
                            app.popup_state = super::PopupState::None;
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
                            app.popup_state = super::PopupState::None;
                        }

                        if ui.button("Cancel").clicked() {
                            app.popup_state = super::PopupState::None;
                        }
                    });
                });
        }

        super::PopupState::CollectionIconConfirmation { id, path, .. } => {
            let mut path = path.clone();
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
                    });

                    if let Some(np) = new_path {
                        path = np;
                        app.popup_state = super::PopupState::CollectionIconConfirmation {
                            id: *id,
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
                            app.update_collection_icon(*id, Some(path.clone()));
                            close = true;
                        }
                        if ui.button("Clear Icon").clicked() {
                            app.update_collection_icon(*id, None);
                            close = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });

            if close {
                app.popup_state = super::PopupState::None;
            }
        }

        _ => {}
    }
}
