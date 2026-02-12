use crate::models::{count_tokens, Lorebook, LorebookEntry};
use crate::ui::CrapApp;
use arboard::Clipboard;
use eframe::egui;

pub fn render_lorebook_entries(
    app: &mut CrapApp,
    ui: &mut egui::Ui,
    book: &mut Lorebook,
    entry_save_req: &mut Option<LorebookEntry>,
    entry_add_req: &mut bool,
    status_update: &mut Option<(String, egui::Color32)>,
) {
    // --- Master-Detail Entries View ---
    ui.allocate_ui(ui.available_size(), |ui| {
        ui.columns(2, |columns| {
            // ENTRY EDITOR (Now on Left)
            columns[0].vertical(|ui| {
                if let Some(entry) = &mut app.selected_entry {
                    // Ensure we are editing an entry belonging to this lorebook
                    if entry.lorebook_id == book.id {
                        ui.heading("Edit Entry");
                        ui.label("Name");
                        if app.editor_search_query.len() >= 3 {
                            let mut layouter = crate::ui::text_highlight::create_highlight_layouter(
                                app.editor_search_query.clone(),
                            );
                            ui.add(
                                egui::TextEdit::singleline(&mut entry.name).layouter(&mut layouter),
                            );
                        } else {
                            ui.text_edit_singleline(&mut entry.name);
                        }

                        ui.label("Keywords (comma separated)");
                        if app.editor_search_query.len() >= 3 {
                            let mut layouter = crate::ui::text_highlight::create_highlight_layouter(
                                app.editor_search_query.clone(),
                            );
                            ui.add(
                                egui::TextEdit::singleline(&mut entry.keywords)
                                    .layouter(&mut layouter),
                            );
                        } else {
                            ui.text_edit_singleline(&mut entry.keywords);
                        }

                        ui.horizontal(|ui| {
                            ui.label("Content");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "Tokens: {} | Chars: {}",
                                            count_tokens(&entry.content),
                                            entry.content.chars().count()
                                        ))
                                        .size(12.0)
                                        .color(egui::Color32::GRAY),
                                    );
                                },
                            );
                        });
                        egui::ScrollArea::vertical()
                            .id_salt("entry_content_scroll")
                            .show(ui, |ui| {
                                if app.editor_search_query.len() >= 3 {
                                    let mut layouter =
                                        crate::ui::text_highlight::create_highlight_layouter(
                                            app.editor_search_query.clone(),
                                        );
                                    ui.add(
                                        egui::TextEdit::multiline(&mut entry.content)
                                            .desired_width(f32::INFINITY)
                                            .layouter(&mut layouter),
                                    );
                                } else {
                                    ui.add(
                                        egui::TextEdit::multiline(&mut entry.content)
                                            .desired_width(f32::INFINITY),
                                    );
                                }
                            });

                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                                if ui.button("Save Entry").clicked() {
                                    *entry_save_req = Some(entry.clone());
                                }
                                if ui.button("📋 Copy").clicked() {
                                    if let Ok(mut clipboard) = Clipboard::new() {
                                        if let Ok(json) = serde_json::to_string(&entry) {
                                            if let Ok(_) = clipboard.set_text(json) {
                                                *status_update = Some((
                                                    "Entry copied to clipboard".to_string(),
                                                    egui::Color32::GREEN,
                                                ));
                                            }
                                        }
                                    }
                                }
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                if ui
                                    .button(egui::RichText::new("Delete").color(egui::Color32::RED))
                                    .clicked()
                                {
                                    // Trigger confirmation popup
                                    app.popup_state =
                                        crate::ui::PopupState::DeleteLorebookEntryConfirmation {
                                            id: entry.id,
                                            lorebook_id: book.id,
                                            name: entry.name.clone(),
                                        };
                                }
                            });
                        });
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label("Select an entry from this Lorebook.")
                        });
                    }
                } else {
                    ui.centered_and_justified(|ui| ui.label("Select an entry to edit."));
                }
            });

            // ENTRY LIST (Now on Right)
            columns[1].vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("Entries");
                    if ui
                        .button("📋")
                        .on_hover_text("Paste Entry from Clipboard")
                        .clicked()
                    {
                        if let Ok(mut clipboard) = Clipboard::new() {
                            if let Ok(text) = clipboard.get_text() {
                                if let Ok(mut new_entry) =
                                    serde_json::from_str::<LorebookEntry>(&text)
                                {
                                    new_entry.lorebook_id = book.id;
                                    new_entry.id = 0; // Ensure new ID
                                    app.add_specific_entry_to_lorebook(new_entry);
                                    *status_update =
                                        Some(("Entry pasted!".to_string(), egui::Color32::GREEN));
                                } else {
                                    *status_update = Some((
                                        "Clipboard does not contain valid entry data".to_string(),
                                        egui::Color32::RED,
                                    ));
                                }
                            }
                        }
                    }
                    if ui.small_button("+").clicked() {
                        *entry_add_req = true;
                    }
                });
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_salt("entries_list_scroll")
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        let mut switch_to_entry = None;
                        for entry in &book.entries {
                            let selected =
                                app.selected_entry.as_ref().map(|e| e.id) == Some(entry.id);

                            let is_match = if app.editor_search_query.len() >= 3 {
                                let q = app.editor_search_query.to_lowercase();
                                entry.name.to_lowercase().contains(&q)
                                    || entry.keywords.to_lowercase().contains(&q)
                                    || entry.content.to_lowercase().contains(&q)
                            } else {
                                false
                            };

                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 2.0;
                                if is_match {
                                    ui.label(
                                        egui::RichText::new("🔍")
                                            .size(10.0)
                                            .color(egui::Color32::from_rgb(255, 215, 0)),
                                    ); // Gold
                                } else {
                                    // Add empty space to align with matched items
                                    ui.add_space(12.0);
                                }

                                let label_text = if is_match {
                                    egui::RichText::new(&entry.name)
                                        .color(egui::Color32::from_rgb(255, 215, 0))
                                } else {
                                    egui::RichText::new(&entry.name)
                                };

                                if ui.selectable_label(selected, label_text).clicked() {
                                    switch_to_entry = Some(entry.clone());
                                }
                            });
                        }

                        if let Some(new_entry) = switch_to_entry {
                            // SYNC BEFORE SWITCH
                            app.push_history();
                            if let Some(current) = &app.selected_entry {
                                if let Some(existing) =
                                    book.entries.iter_mut().find(|e| e.id == current.id)
                                {
                                    *existing = current.clone();
                                }
                            }
                            app.selected_entry = Some(new_entry);
                        }
                    });
            });
        });
    });
}
