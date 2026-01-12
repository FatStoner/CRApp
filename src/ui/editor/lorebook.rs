use eframe::egui;
use crate::ui::CrapApp;

pub fn render_lorebook_editor(app: &mut CrapApp, ui: &mut egui::Ui) {
                if let Some(mut book) = app.selected_lorebook.take() {
                    let mut save_lore_req = None;
                    let mut tag_add_request = None;
                    let mut tag_remove_request = None;
                    let mut entry_save_req = None;
                    let mut back_history_req = false;

                    let mut entry_add_req = false;
                    
                    let mut status_update: Option<(String, egui::Color32)> = None;
                    
                     // Check Dirty State
                    let is_dirty = if book.id == 0 {
                         !book.content_eq(&crate::models::Lorebook::default())
                    } else {
                         if let Some(original) = app.lorebooks.iter().find(|b| b.id == book.id) {
                             !book.content_eq(original)
                         } else {
                             false
                         }
                    };

                    // Check for Ctrl+S (Global scope for editor)
                    if ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::S)) {
                         save_lore_req = Some(book.clone());
                    }

                    ui.horizontal(|ui| {
                        if ui.button("⬅ Back").clicked() {
                           // Use app.request_back() to handle checks
                           back_history_req = true;
                        }
                        
                        // Handle Esc key for Back navigation
                        if ui.memory(|m| m.focused().is_none()) && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            back_history_req = true;
                        }

                        ui.heading("Edit Lorebook");

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // EXPORT (Placeholder)
                            ui.add_enabled(false, egui::Button::new("EXPORT"));
                            
                            // IMPORT (Placeholder)
                            ui.add_enabled(false, egui::Button::new("IMPORT"));

                            // SAVE
                            let mut save_color = ui.visuals().widgets.inactive.bg_fill;
                            if is_dirty {
                                 save_color = egui::Color32::from_rgb(200, 100, 50); // Orange/Red
                            }
                            
                            if ui.add(egui::Button::new(egui::RichText::new("SAVE").strong()).fill(save_color)).clicked() {
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
                        ui.add(
                            egui::TextEdit::singleline(&mut app.editor_search_query)
                                .hint_text("Type 3+ chars to highlight/jump...")
                                .desired_width(200.0),
                        );

                        if !app.editor_search_query.is_empty() {
                            if ui.small_button("✖").clicked() {
                                app.editor_search_query.clear();
                            }

                            ui.label(
                                egui::RichText::new(format!(
                                    "Highlighting: '{}'",
                                    app.editor_search_query
                                ))
                                .size(11.0)
                                .color(egui::Color32::GRAY),
                            );
                        }
                    });
                    ui.separator();

                    // --- Auto-selection logic ---
                    if app.editor_search_query.len() >= 3 {
                        let query_lower = app.editor_search_query.to_lowercase();

                        // 1. Check if metadata already matches (stay where we are)
                        let metadata_match = book.title.to_lowercase().contains(&query_lower)
                            || book.content.to_lowercase().contains(&query_lower);

                        if !metadata_match {
                            // 2. Check if currently selected entry matches
                            let current_match = if let Some(e) = &app.selected_entry {
                                e.name.to_lowercase().contains(&query_lower)
                                    || e.keywords.to_lowercase().contains(&query_lower)
                                    || e.content.to_lowercase().contains(&query_lower)
                            } else {
                                false
                            };

                            if !current_match {
                                // 3. Find first matching entry
                                if let Some(matching_entry) = book.entries.iter().find(|e| {
                                    e.name.to_lowercase().contains(&query_lower)
                                        || e.keywords.to_lowercase().contains(&query_lower)
                                        || e.content.to_lowercase().contains(&query_lower)
                                }) {
                                    app.selected_entry = Some(matching_entry.clone());
                                    // Switch to Entries tab if not already there
                                    app.active_lorebook_tab = crate::ui::LorebookTab::Entries;
                                }
                            }
                        }
                    }
                    
                    // --- Top Section: Metadata ---
                    egui::ScrollArea::vertical()
                        .max_height(ui.available_height() * 0.45)
                        .id_salt("lorebook_metadata_scroll")
                        .show(ui, |ui| {
                            let total_width = ui.available_width();
                            let right_width = (total_width * 0.35).max(160.0).min(300.0);
                            let left_width = total_width - right_width - ui.spacing().item_spacing.x;

                            ui.horizontal_top(|ui| {
                                // Left Column: Basic Data
                                ui.allocate_ui_with_layout(egui::vec2(left_width, 10.0), egui::Layout::top_down(egui::Align::Min), |ui| {
                                    ui.label("Title");
                                    if app.editor_search_query.len() >= 3 {
                                        let mut layouter = crate::ui::text_highlight::create_highlight_layouter(app.editor_search_query.clone());
                                        ui.add(egui::TextEdit::singleline(&mut book.title).desired_width(f32::INFINITY).layouter(&mut layouter));
                                    } else {
                                        ui.add(egui::TextEdit::singleline(&mut book.title).desired_width(f32::INFINITY));
                                    }
                                    ui.add_space(8.0);
                                    
                                    ui.label("Description");
                                    if app.editor_search_query.len() >= 3 {
                                        let mut layouter = crate::ui::text_highlight::create_highlight_layouter(app.editor_search_query.clone());
                                        ui.add(egui::TextEdit::multiline(&mut book.content).desired_width(f32::INFINITY).layouter(&mut layouter));
                                    } else {
                                        ui.add(egui::TextEdit::multiline(&mut book.content).desired_width(f32::INFINITY));
                                    }
                                    ui.add_space(8.0);

                                    // Tags Section
                                    ui.label("Tags:");
                                    ui.horizontal_wrapped(|ui| {
                                        for tag in &book.tags {
                                            ui.group(|ui| {
                                                ui.horizontal(|ui| {
                                                    ui.label(egui::RichText::new(&tag.name).color(egui::Color32::WHITE).size(12.0));
                                                    if ui.small_button("x").clicked() {
                                                        tag_remove_request = Some((book.id, tag.id));
                                                    }
                                                });
                                            });
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        let response = ui.add(egui::TextEdit::singleline(&mut app.app_tag_input).desired_width(120.0));
                                        if (ui.button("Add").clicked() || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))) && !app.app_tag_input.is_empty() {
                                            tag_add_request = Some((book.id, app.app_tag_input.clone()));
                                            app.app_tag_input.clear();
                                            response.request_focus();
                                        }
                                    });
                                    
                                    ui.add_space(8.0);
                                });

                                // Right Column: Cover Image
                                ui.allocate_ui_with_layout(egui::vec2(right_width, 10.0), egui::Layout::top_down(egui::Align::Min), |ui| {
                                    let max_total_h = ui.ctx().screen_rect().height() * 0.333;
                                    ui.set_max_height(max_total_h);

                                    ui.label(egui::RichText::new("Cover Image").strong());
                                    
                                    if let Some(path_str) = &book.cover_path {
                                        let uri = crate::ui::get_image_uri(path_str);
                                        
                                        // Total height is limited to 1/3 of screen. 
                                        // Subtracting ~80px for labels and buttons.
                                        let image_max_h = (max_total_h - 80.0).max(100.0);
                                        let preview_width = ui.available_width();
                                        
                                        ui.add(
                                            egui::Image::new(uri)
                                                .rounding(egui::Rounding::same(4.0))
                                                .max_height(image_max_h)
                                                .max_width(preview_width)
                                        );
                                        
                                        ui.add_space(4.0);
                                        ui.horizontal_wrapped(|ui| {
                                            ui.spacing_mut().item_spacing.x = 0.0;
                                            ui.label(egui::RichText::new("Path: ").weak());
                                            ui.label(egui::RichText::new(path_str).weak().italics());
                                        });
                                    } else {
                                        // Empty state placeholder
                                        let (rect, _) = ui.allocate_exact_size(egui::vec2(100.0, 100.0), egui::Sense::hover());
                                        ui.painter().rect_stroke(rect, 4.0, (1.0, egui::Color32::from_gray(60)));
                                        ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "No Cover", egui::FontId::proportional(14.0), egui::Color32::from_gray(100));
                                    }

                                    ui.add_space(8.0);
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 8.0;
                                        if ui.button("Browse...").clicked() {
                                            if let Some(path) = rfd::FileDialog::new().add_filter("image", &["png", "jpg", "jpeg"]).pick_file() {
                                                  let dest_dir = std::path::Path::new("data/covers");
                                                  let _ = std::fs::create_dir_all(dest_dir);
                                                  if let Some(name) = path.file_name() {
                                                      let dest = dest_dir.join(name);
                                                      let _ = std::fs::copy(&path, &dest);
                                                      book.cover_path = Some(dest.to_string_lossy().to_string());
                                                  }
                                            }
                                        }
                                        if ui.button("Paste").clicked() {
                                             match arboard::Clipboard::new() {
                                                  Ok(mut clipboard) => {
                                                      if let Ok(img_data) = clipboard.get_image() {
                                                          let width = img_data.width as u32;
                                                          let height = img_data.height as u32;
                                                          let bytes = img_data.bytes.into_owned();
                                                          if let Some(image_buffer) = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(width, height, bytes) {
                                                              let timestamp = chrono::Utc::now().timestamp_millis();
                                                              let filename = format!("pasted_cover_{}.png", timestamp);
                                                              let dest_dir = std::path::Path::new("data/covers");
                                                              let _ = std::fs::create_dir_all(dest_dir);
                                                              let dest_path = dest_dir.join(&filename);
                                                              if let Ok(_) = image_buffer.save(&dest_path) {
                                                                  book.cover_path = Some(dest_path.to_string_lossy().to_string());
                                                                  status_update = Some(("Cover pasted!".to_string(), egui::Color32::GREEN));
                                                              }
                                                          }
                                                      }
                                                  },
                                                  Err(_) => {}
                                             }
                                        }
                                        if book.cover_path.is_some() {
                                            if ui.button("Remove").clicked() {
                                                book.cover_path = None;
                                            }
                                        }
                                    });
                                });
                            });
                        });
                    
                    ui.separator();

                    ui.separator();
                    
                    // --- Bottom Section: Tabs ---
                    let linked_char_count = app.characters.iter()
                        .filter(|c| app.char_lore_map.get(&c.id).map(|l| l.contains(&book.id)).unwrap_or(false))
                        .count();

                    ui.horizontal(|ui| {
                        let entries_label = format!("Entries ({})", book.entries.len());
                        if ui.selectable_label(app.active_lorebook_tab == crate::ui::LorebookTab::Entries, entries_label).clicked() {
                            app.active_lorebook_tab = crate::ui::LorebookTab::Entries;
                        }
                        let chars_label = format!("Characters ({})", linked_char_count);
                        if ui.selectable_label(app.active_lorebook_tab == crate::ui::LorebookTab::Characters, chars_label).clicked() {
                            app.active_lorebook_tab = crate::ui::LorebookTab::Characters;
                        }
                    });
                    ui.separator();

                    match app.active_lorebook_tab {
                        crate::ui::LorebookTab::Entries => {
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
                                                      let mut layouter = crate::ui::text_highlight::create_highlight_layouter(app.editor_search_query.clone());
                                                      ui.add(egui::TextEdit::singleline(&mut entry.name).layouter(&mut layouter));
                                                  } else {
                                                      ui.text_edit_singleline(&mut entry.name);
                                                  }
                                                  
                                                  ui.label("Keywords (comma separated)");
                                                  if app.editor_search_query.len() >= 3 {
                                                      let mut layouter = crate::ui::text_highlight::create_highlight_layouter(app.editor_search_query.clone());
                                                      ui.add(egui::TextEdit::singleline(&mut entry.keywords).layouter(&mut layouter));
                                                  } else {
                                                      ui.text_edit_singleline(&mut entry.keywords);
                                                  }
                                                  
                                                  ui.label("Content");
                                                  egui::ScrollArea::vertical().id_salt("entry_content_scroll").show(ui, |ui| {
                                                      if app.editor_search_query.len() >= 3 {
                                                          let mut layouter = crate::ui::text_highlight::create_highlight_layouter(app.editor_search_query.clone());
                                                          ui.add(egui::TextEdit::multiline(&mut entry.content).desired_width(f32::INFINITY).layouter(&mut layouter));
                                                      } else {
                                                          ui.add(egui::TextEdit::multiline(&mut entry.content).desired_width(f32::INFINITY));
                                                      }
                                                  });
        
                                                 ui.add_space(8.0);
                                                 ui.horizontal(|ui| {
                                                     ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                                                         if ui.button("Save Entry").clicked() {
                                                             entry_save_req = Some(entry.clone());
                                                         }
                                                     });
                                                     
                                                     ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                                         if ui.button(egui::RichText::new("Delete").color(egui::Color32::RED)).clicked() {
                                                             // Trigger confirmation popup
                                                             app.popup_state = crate::ui::PopupState::DeleteLorebookEntryConfirmation { 
                                                                 id: entry.id, 
                                                                 lorebook_id: book.id,
                                                                 name: entry.name.clone() 
                                                             };
                                                         }
                                                     });
                                                 });
                                             } else {
                                                 ui.centered_and_justified(|ui| ui.label("Select an entry from this Lorebook."));
                                             }
                                         } else {
                                             ui.centered_and_justified(|ui| ui.label("Select an entry to edit."));
                                         }
                                     });

                                     // ENTRY LIST (Now on Right)
                                     columns[1].vertical(|ui| {
                                         ui.horizontal(|ui| {
                                             ui.heading("Entries");
                                             if ui.small_button("+").clicked() {
                                                 entry_add_req = true;
                                             }
                                         });
                                         ui.separator();
                                         egui::ScrollArea::vertical().id_salt("entries_list_scroll").show(ui, |ui| {
                                             ui.set_width(ui.available_width());
                                              for entry in &book.entries {
                                                  let selected = app.selected_entry.as_ref().map(|e| e.id) == Some(entry.id);
                                                  
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
                                                          ui.label(egui::RichText::new("🔍").size(10.0).color(egui::Color32::from_rgb(255, 215, 0))); // Gold
                                                      } else {
                                                          // Add empty space to align with matched items
                                                          ui.add_space(12.0);
                                                      }
                                                      
                                                      let label_text = if is_match {
                                                          egui::RichText::new(&entry.name).color(egui::Color32::from_rgb(255, 215, 0))
                                                      } else {
                                                          egui::RichText::new(&entry.name)
                                                      };

                                                      if ui.selectable_label(selected, label_text).clicked() {
                                                          app.selected_entry = Some(entry.clone());
                                                      }
                                                  });
                                              }
                                         });
                                     });
                                 });
                            });
                        },
                        crate::ui::LorebookTab::Characters => {
                             // --- Linked Characters Gallery ---
                             ui.heading(format!("Characters Linked to '{}'", book.title));
                             ui.add_space(8.0);
                             
                             let mut browser_actions = Vec::new();
                             let all_colls = app.collections.clone(); // Clone for immutable access to collections inside loop
                             
                             // Filter characters
                             let linked_chars: Vec<crate::models::Character> = app.characters.iter()
                                .filter(|c| {
                                    if let Some(links) = app.char_lore_map.get(&c.id) {
                                        links.contains(&book.id)
                                    } else {
                                        false
                                    }
                                })
                                .cloned()
                                .collect();

                             if linked_chars.is_empty() {
                                 ui.label("No characters linked to this Lorebook.");
                                 ui.label("Go to a Character -> Lorebooks tab to link them.");
                             } else {
                                 egui::ScrollArea::vertical().id_salt("lore_chars_scroll").show(ui, |ui| {
                                     ui.horizontal_wrapped(|ui| {
                                         for char in &linked_chars {
                                             crate::ui::browser::render_character_card(ui, app, char, &all_colls, &mut browser_actions);
                                         }
                                     });
                                 });
                             }
                             
                             // Handle Browser Actions generated by cards
                             for action in browser_actions {
                                 match action {
                                     crate::ui::browser::BrowserAction::MoveCharacter(char_id, target_id) => {
                                         app.move_character(char_id, target_id);
                                     }
                                     crate::ui::browser::BrowserAction::ToggleFavorite(char_id) => {
                                         app.toggle_favorite(char_id);
                                     }
                                     crate::ui::browser::BrowserAction::DeleteCharacter(id) => {
                                          let name = app.characters.iter().find(|c| c.id == id).map(|c| c.name.clone()).unwrap_or_default();
                                          app.popup_state = crate::ui::PopupState::DeleteCharacterConfirmation { id, name };
                                     }
                                     crate::ui::browser::BrowserAction::RenameCollection(id, name) => {
                                         app.popup_state = crate::ui::PopupState::Renaming { id, name };
                                     }
                                     crate::ui::browser::BrowserAction::DeleteCollection(id) => {
                                         app.delete_collection(id); // Simple delete call, ignoring complex warning logic for now or reusing if precise
                                     }
                                      crate::ui::browser::BrowserAction::CreateCharacter(cid) => {
                                          app.create_new_character(cid);
                                      }
                                      crate::ui::browser::BrowserAction::CreateCollection(cid) => {
                                          app.save_collection(0, "New Folder".to_string(), cid);
                                      }
                                      crate::ui::browser::BrowserAction::UpdateCollectionIcon(id) => {
                                          app.popup_state = crate::ui::PopupState::CollectionIconConfirmation {
                                              id,
                                              path: String::new(),
                                              preview_texture: None,
                                          };
                                      }
                                      crate::ui::browser::BrowserAction::OpenCharacter(id) => {
                                           // Temporarily restore ownership so push_history sees it
                                           app.selected_lorebook = Some(book.clone());
                                           app.load_character(id);
                                           // Note: We don't need to unset it because we are navigating away entirely.
                                      }
                                 }
                             }
                        }
                    }


                    // --- Event Handling ---
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
                    app.selected_lorebook = Some(book);
                    
                    if back_history_req {
                        app.request_back();
                    }

                } else {
                    ui.label("Select a lorebook.");
                }
}
