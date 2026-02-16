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
                        crate::ui::components::CodeEditor::new(
                            &mut entry.name,
                            format!("lore_entry_name_{}", entry.id),
                        )
                        .single_line()
                        .highlight(app.editor_search_query.clone())
                        .show(
                            ui,
                            &mut app.cosmic_font_system,
                            &mut app.cosmic_swash_cache,
                            &mut app.cosmic_atlas,
                            &mut app.cosmic_editors,
                            &mut app.cosmic_clipboard,
                        );

                        ui.label("Keywords (comma separated)");
                        crate::ui::components::CodeEditor::new(
                            &mut entry.keywords,
                            format!("lore_entry_keywords_{}", entry.id),
                        )
                        .single_line()
                        .highlight(app.editor_search_query.clone())
                        .show(
                            ui,
                            &mut app.cosmic_font_system,
                            &mut app.cosmic_swash_cache,
                            &mut app.cosmic_atlas,
                            &mut app.cosmic_editors,
                            &mut app.cosmic_clipboard,
                        );

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
                            .id_source("entry_content_scroll")
                            .show(ui, |ui| {
                                // The provided snippet uses `book.entries[idx].content` and `idx` which are not available here.
                                // Assuming `entry.content` is the correct variable to modify.
                                // Also, the CodeEditor doesn't seem to take a layouter directly,
                                // and the search query highlighting might need to be handled differently or not at all by CodeEditor.
                                // For faithful replacement, we'll use CodeEditor directly.
                                crate::ui::components::CodeEditor::new(
                                    &mut entry.content,
                                    format!("lore_entry_content_{}", entry.id), // Using entry.id for unique ID
                                )
                                .desired_lines(15)
                                .highlight(app.editor_search_query.clone())
                                .show(
                                    ui,
                                    &mut app.cosmic_font_system,
                                    &mut app.cosmic_swash_cache,
                                    &mut app.cosmic_atlas,
                                    &mut app.cosmic_editors,
                                    &mut app.cosmic_clipboard,
                                );
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
                    .id_source("entries_list_scroll")
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
