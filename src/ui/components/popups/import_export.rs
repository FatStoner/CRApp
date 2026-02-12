use crate::ui::CrapApp;
use eframe::egui;

/// Render import/export popups
pub fn render_import_export_popups(ctx: &egui::Context, app: &mut CrapApp, state: &super::PopupState) {
    match state {
        super::PopupState::ImportDbWarning => {
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
                            app.popup_state = super::PopupState::None;
                        }
                        if ui.button("Cancel").clicked() {
                            app.popup_state = super::PopupState::None;
                        }
                    });
                });
        }

        super::PopupState::ExportDbSelection => {
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
                app.popup_state = super::PopupState::None;
            }
        }

        super::PopupState::LorebookImport {
            source_code,
            parsed_data,
        } => {
            render_lorebook_import(ctx, app, source_code.clone(), parsed_data.clone());
        }

        _ => {}
    }
}

fn render_lorebook_import(
    ctx: &egui::Context,
    app: &mut CrapApp,
    mut source_code: String,
    parsed_data: Option<crate::ui::parsing::ParsedLorebookData>,
) {
    use super::PopupState;
    
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
            source_code,
            parsed_data: Some(parsed),
        };
    } else if do_import {
        if let Some(data) = parsed_data {
            let mut lorebook = if let Some(current) = &app.selected_lorebook {
                current.clone()
            } else {
                crate::models::Lorebook {
                     title: if data.title.is_empty() { "Imported Lorebook".to_string() } else { data.title.clone() },
                     ..Default::default()
                }
            };

            if !data.title.is_empty() {
                 lorebook.title = data.title;
            }
            lorebook.description = data.description.clone();
            lorebook.content = data.description;

            lorebook.entries = data.entries.into_iter().map(|e| {
                crate::models::LorebookEntry {
                    lorebook_id: lorebook.id,
                    name: e.name,
                    keywords: e.keywords.join(", "),
                    content: e.content,
                    ..Default::default()
                }
            }).collect();

            lorebook.tags = data.tags.into_iter().map(|t| {
                crate::models::Tag { id: 0, name: t }
            }).collect();

            app.selected_lorebook = Some(lorebook);
            app.mode = crate::ui::AppMode::Lorebooks;
            app.central_view = crate::ui::CentralView::Editor;
            app.popup_state = PopupState::None;
            
            app.set_status("Imported data into editor. Click SAVE to persist.".to_string(), egui::Color32::YELLOW);
        }
    } else if close {
        app.popup_state = PopupState::None;
    } else if !source_code.is_empty() {
        app.popup_state = PopupState::LorebookImport {
            source_code,
            parsed_data,
        };
    }
}
