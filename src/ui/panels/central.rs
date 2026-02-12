use crate::ui::views::{render_browser_view, render_editor_view, render_lorebook_editor, render_template_editor};
use crate::ui::parsing::parse_clipboard;
use crate::models::Tag;
use crate::ui::views::render_options_window;
use crate::ui::{AppMode, CrapApp};
use eframe::egui;

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
                        ui.label("Supported Platforms:");
                        ui.label("• JanitorAI (Profile & Edit)");
                        ui.label("• CraveU AI (Edit)");
                        ui.label("• GirlfriendGPT");
                        ui.label("• SpicyChat (Chatbots & Lorebooks)");
                        ui.label(egui::RichText::new("1. Go to the character profile OR edit page on the service\n2. Select All (Ctrl+A)\n3. Copy (Ctrl+C)\n4. Paste here (Ctrl+V)").size(11.0).color(egui::Color32::GRAY));
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
                        
                        egui::ScrollArea::vertical()
                            .id_salt("review_parsed_data_scroll") // unique id
                            .auto_shrink([false, false]) // Don't shrink to content, fill window
                            .show(ui, |ui| {
                                let data = app.parsed_data.as_mut().unwrap(); // FIX: Define data inside the closure!

                                let avail_width = ui.available_width();
                                // We are in a grid with 2 columns.
                                // We need to estimate input width.
                                // If we don't know exact label width, we can use a reasonable default or layout differently.
                                // Actually, let's just use single column for small screens or keep grid but use relative sizing?
                                // If we use desired_width(input_width), we need to make sure grid respects it.
                                // Let's try to just subtract a fixed amount for label.
                                let input_width = (avail_width - 150.0).max(200.0);

                                egui::Grid::new("review_grid").striped(true).num_columns(2).min_col_width(100.0).show(ui, |ui| {
                                    ui.label("Name (File Name):");
                                    ui.add(egui::TextEdit::singleline(&mut data.name).desired_width(input_width));
                                    ui.end_row();

                                    ui.label("Character Name:");
                                    ui.add(egui::TextEdit::singleline(&mut data.char_name).desired_width(input_width));
                                    ui.end_row();
                                    
                                    ui.label("Title (Bio):");
                                    ui.add(egui::TextEdit::multiline(&mut data.title).desired_rows(3).desired_width(input_width));
                                    ui.end_row();
                                    
                                    ui.label("Personality:");
                                    ui.add(egui::TextEdit::multiline(&mut data.personality).desired_rows(10).desired_width(input_width));
                                    ui.end_row();
                                    
                                    ui.label("Scenario:");
                                    ui.add(egui::TextEdit::multiline(&mut data.scenario).desired_rows(8).desired_width(input_width));
                                    ui.end_row();
                                    
                                    ui.label("First Message:");
                                    ui.add(egui::TextEdit::multiline(&mut data.first_message).desired_rows(6).desired_width(input_width));
                                    ui.end_row();

                                    ui.label("Example Dialogue:");
                                    ui.add(egui::TextEdit::multiline(&mut data.example_dialogue).desired_rows(6).desired_width(input_width));
                                    ui.end_row();
                                
                                ui.label("External Tags:");
                                ui.vertical(|ui| {
                                    ui.set_max_width(input_width); // FIX: Constrain width for wrapping
                                    ui.horizontal_wrapped(|ui| {
                                        let mut tags_to_remove = Vec::new();
                                        for (i, tag) in data.external_tags.iter().enumerate() {
                                            egui::Frame::none().fill(egui::Color32::from_gray(80)).rounding(12.0).inner_margin(4.0).show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    ui.label(egui::RichText::new(tag).color(egui::Color32::WHITE).size(12.0));
                                                    if ui.small_button("x").clicked() {
                                                        tags_to_remove.push(i);
                                                    }
                                                });
                                            });
                                        }
                                        for i in tags_to_remove.iter().rev() {
                                            data.external_tags.remove(*i);
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        let response = ui.text_edit_singleline(&mut app.ext_tag_input);
                                        if (ui.button("Add").clicked() || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))) && !app.ext_tag_input.is_empty() {
                                            data.external_tags.push(app.ext_tag_input.clone());
                                            app.ext_tag_input.clear();
                                            response.request_focus();
                                        }
                                    });
                                });
                                ui.end_row();

                                ui.label("App Tags:");
                                ui.vertical(|ui| {
                                    ui.set_max_width(input_width); // FIX: Constrain width for wrapping
                                    ui.horizontal_wrapped(|ui| {
                                        let mut tags_to_remove = Vec::new();
                                        for (i, tag) in data.app_tags.iter().enumerate() {
                                            egui::Frame::none().fill(egui::Color32::from_rgb(50, 80, 150)).rounding(12.0).inner_margin(4.0).show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    ui.label(egui::RichText::new(tag).color(egui::Color32::WHITE).size(12.0));
                                                    if ui.small_button("x").clicked() {
                                                        tags_to_remove.push(i);
                                                    }
                                                });
                                            });
                                        }
                                        for i in tags_to_remove.iter().rev() {
                                            data.app_tags.remove(*i);
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        let response = ui.text_edit_singleline(&mut app.app_tag_input);
                                        if (ui.button("Add").clicked() || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))) && !app.app_tag_input.is_empty() {
                                            data.app_tags.push(app.app_tag_input.clone());
                                            app.app_tag_input.clear();
                                            response.request_focus();
                                        }
                                    });
                                });
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
                                    }
                                    if !d.char_name.is_empty() {
                                        c.char_name = d.char_name.clone();
                                    }
                                    if !d.title.is_empty() { c.char_title = d.title.clone(); }
                                    if !d.personality.is_empty() { c.personality = d.personality.clone(); }
                                    if !d.scenario.is_empty() { c.scenario = d.scenario; }
                                    if !d.first_message.is_empty() { c.first_message = d.first_message; }
                                    if !d.example_dialogue.is_empty() { c.example_dialogue = d.example_dialogue; }
                                    
                                    if c.id != 0 {
                                        // EXISTING CHARACTER
                                        // Update in-memory tags (Overwrite)
                                        c.external_tags.clear();
                                        for tag_name in &d.external_tags {
                                            c.external_tags.push(Tag { id: 0, name: tag_name.clone() });
                                        }

                                        c.app_tags.clear();
                                        for tag_name in &d.app_tags {
                                            c.app_tags.push(Tag { id: 0, name: tag_name.clone() });
                                        }

                                        status_update = Some(("Data applied. Click SAVE to persist.".to_string(), egui::Color32::GREEN));
                                        
                                        // Update URLs
                                        if !d.urls.is_empty() {
                                             c.urls = d.urls.clone();
                                        }
                                        // Update Avatar
                                        if let Some(path) = &d.avatar_path {
                                            c.avatar_path = Some(path.clone());
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
                                        // Update Avatar
                                        if let Some(path) = &d.avatar_path {
                                            c.avatar_path = Some(path.clone());
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

        if app.show_options_window {
            render_options_window(app, ctx);
        }

        // Global Search View
        if app.mode == AppMode::DeepSearch {
            crate::ui::views::search::render_deep_search(app, ui);
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
            AppMode::Templates => {
                render_template_editor(app, ui);
            },
            _ => {
                ui.label("Unknown mode");
            }
        }
    });
}
