use eframe::egui;
use crate::models::Tag;
use crate::ui::{CrapApp, AppMode, UiEvent};
use super::parsing::parse_clipboard;
use super::browser::render_browser_view;
use super::editor::{render_editor_view, render_lorebook_editor};

// ----------------------------------------------------------------------------
// Rendering Logic
// ----------------------------------------------------------------------------

pub fn render_central_panel(app: &mut CrapApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
         // Modals (Import)
        if app.show_import_modal {
            let screen_rect = ctx.screen_rect();
            egui::Window::new("Import from Clipboard")
                .collapsible(false)
                .resizable(true)
                .pivot(egui::Align2::CENTER_CENTER)
                .fixed_pos(screen_rect.center())
                .default_width(screen_rect.width() * 0.6)
                .max_height(screen_rect.height() * 0.8)
                .show(ctx, |ui| {
                    if app.parsed_data.is_none() {
                        // Phase 1: Input
                        ui.label("Import from spicychat.ai:");
                        ui.label(egui::RichText::new("1. Go to the character profile on spicychat.ai\n2. Select All (Ctrl+A)\n3. Copy (Ctrl+C)\n4. Paste here (Ctrl+V)").size(11.0).color(egui::Color32::GRAY));
                        ui.add_space(4.0);
                        
                        let footer_height = 50.0;
                        let scroll_height = ui.available_height() - footer_height;
                        
                        egui::ScrollArea::vertical()
                            .max_height(scroll_height)
                            .show(ui, |ui| {
                                ui.add_sized(
                                    egui::Vec2::new(ui.available_width(), scroll_height.max(200.0)),
                                    egui::TextEdit::multiline(&mut app.import_text)
                                        .hint_text("Paste here...")
                                        .desired_width(f32::INFINITY)
                                        .lock_focus(false)
                                );
                            });
                        
                        ui.add_space(10.0);
                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button("Analyze").clicked() {
                                let data = parse_clipboard(&app.import_text);
                                app.parsed_data = Some(data);
                            }
                            if ui.button("Cancel").clicked() {
                                app.show_import_modal = false;
                            }
                        });
                    } else {
                        // Phase 2: Review
                        ui.heading("Review Parsed Data");
                        ui.separator();
                        
                        let data = app.parsed_data.as_mut().unwrap();
                        
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            egui::Grid::new("review_grid").striped(true).num_columns(2).show(ui, |ui| {
                                ui.label("Name:");
                                ui.text_edit_singleline(&mut data.name);
                                ui.end_row();
                                
                                ui.label("Title:");
                                ui.text_edit_singleline(&mut data.title);
                                ui.end_row();
                                
                                ui.label("Personality:");
                                ui.add(egui::TextEdit::multiline(&mut data.personality).desired_rows(10).desired_width(f32::INFINITY));
                                ui.end_row();
                                
                                ui.label("Scenario:");
                                ui.add(egui::TextEdit::multiline(&mut data.scenario).desired_rows(8).desired_width(f32::INFINITY));
                                ui.end_row();
                                
                                ui.label("First Message:");
                                ui.add(egui::TextEdit::multiline(&mut data.first_message).desired_rows(6).desired_width(f32::INFINITY));
                                ui.end_row();

                                ui.label("Example Dialogue:");
                                ui.add(egui::TextEdit::multiline(&mut data.example_dialogue).desired_rows(6).desired_width(f32::INFINITY));
                                ui.end_row();
                                
                                ui.label("External Tags:");
                                ui.label(data.external_tags.join(", "));
                                ui.end_row();

                                ui.label("App Tags:");
                                ui.label(data.app_tags.join(", "));
                                ui.end_row();

                                ui.label("URLs:");
                                ui.vertical(|ui| {
                                    for url in &data.urls {
                                        ui.horizontal(|ui| {
                                            if let Some(lbl) = &url.label {
                                                ui.label(egui::RichText::new(format!("{}:", lbl)).strong());
                                            }
                                            ui.label(&url.url);
                                        });
                                    }
                                });
                                ui.end_row();
                            });
                        });
                        
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if ui.button("Apply to Character").clicked() {
                                let mut status_update = None;

                                if let Some(c) = &mut app.selected_character {
                                    let d = app.parsed_data.take().unwrap();
                                    
                                    if !d.name.is_empty() { 
                                        c.name = d.name.clone(); 
                                        c.char_name = d.name.clone();
                                    }
                                    if !d.title.is_empty() { c.char_title = d.title.clone(); }
                                    if !d.personality.is_empty() { c.personality = d.personality.clone(); }
                                    if !d.scenario.is_empty() { c.scenario = d.scenario; }
                                    if !d.first_message.is_empty() { c.first_message = d.first_message; }
                                    if !d.example_dialogue.is_empty() { c.example_dialogue = d.example_dialogue; }
                                    
                                    if c.id != 0 {
                                        // EXISTING CHARACTER
                                        let tx_clone = app.tx.clone();
                                        let db_clone = app.db.clone();
                                        let cid = c.id;
                                        
                                        // Tags
                                        let ext_tags = d.external_tags.clone();
                                        let app_tags = d.app_tags.clone();
                                        
                                        tokio::spawn(async move {
                                            for tag_name in ext_tags {
                                                let _ = db_clone.add_tag_to_character(cid, &tag_name, true).await;
                                            }
                                            for tag_name in app_tags {
                                                 let _ = db_clone.add_tag_to_character(cid, &tag_name, false).await;
                                            }
                                            let _ = tx_clone.send(UiEvent::TagOperationFinished(Ok(()))).await;
                                        });

                                        status_update = Some(("Data updated. Tags being added.".to_string(), egui::Color32::GREEN));
                                        
                                        // Update URLs
                                        if !d.urls.is_empty() {
                                             c.urls = d.urls.clone();
                                        }
                                    } else {
                                        // NEW CHARACTER
                                        for tag_name in d.external_tags {
                                            c.external_tags.push(Tag { id: 0, name: tag_name });
                                        }
                                        for tag_name in d.app_tags {
                                            c.app_tags.push(Tag { id: 0, name: tag_name });
                                        }
                                        if !d.urls.is_empty() {
                                            c.urls = d.urls.clone();
                                            // Reset IDs for new urls
                                            for u in &mut c.urls {
                                                u.id = 0;
                                                u.character_id = 0;
                                            }
                                        }
                                        status_update = Some(("Import applied to New Character (Unsaved).".to_string(), egui::Color32::YELLOW));
                                    }
                                }

                                if let Some((msg, color)) = status_update {
                                    app.set_status(msg, color);
                                }

                                app.show_import_modal = false;
                            }
                            
                            if ui.button("Back").clicked() {
                                app.parsed_data = None;
                            }
                            
                            if ui.button("Cancel").clicked() {
                                app.show_import_modal = false;
                            }
                        });
                    }
                });
            return; // Modal blocking
        }

        // Global Search View
        if app.mode == AppMode::DeepSearch {
            super::global_search::render_deep_search(app, ui);
            return;
        }

        // Main Content
        match app.mode {
            AppMode::Characters => {
                match app.central_view {
                    crate::ui::CentralView::Browser => {
                        render_browser_view(app, ui);
                    },
                    crate::ui::CentralView::Editor => {
                        render_editor_view(app, ui);
                    }
                }
            },
            AppMode::Lorebooks => {
                render_lorebook_editor(app, ui);
            },
            _ => {
                ui.label("Unknown mode");
            }
        }
    });
}
