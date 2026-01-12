use eframe::egui;
use crate::models::count_tokens;
use crate::ui::{CrapApp, CharacterTab, UiEvent};
use crate::card_v2::CharacterCardV2;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

pub fn render_editor_view(app: &mut CrapApp, ui: &mut egui::Ui) {
    let mut trigger_import = false;

    let mut save_req = None;
    let mut toggle_requests = Vec::<(i64, i64, bool)>::new();
    let mut status_update = None;
    let mut back_req = None;

    // Check Dirty State (before taking ownership, or logic moved down? We can do it on the owned copy too)
    // Actually, we can check dirty state on the owned copy vs the DB cache.
    // Ideally we assume 'app' state is consistent.

    // Prepare collection options - Need value BEFORE taking character if we want to use 'app' easily, 
    // though if we take character, 'app' is free to be used for this too.
    let collection_options: Vec<(i64, String)> = app.collections.iter().map(|c| {
       (c.id, app.get_collection_path(c.id))
    }).collect();
   
    // Take ownership of selected_character to allow mutable access to it AND app simultaneously
    if let Some(mut character) = app.selected_character.take() {
                     // Clone for closures
                     let _tx_clone = app.tx.clone();
                     let _db_clone = app.db.clone();
                     
                     // Helper for dirty check locally
                     let is_dirty = if character.id == 0 {
                         true
                     } else {
                         if let Some(original) = app.characters.iter().find(|c| c.id == character.id) {
                             !character.content_eq(original)
                         } else {
                             true
                         }
                     };
                     
                     ui.horizontal(|ui| {
                        if ui.button("⬅ Back").clicked() {
                            back_req = Some(character.collection_id);
                        }
                        // Handle Esc key for Back navigation
                        if ui.memory(|m| m.focused().is_none()) && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            back_req = Some(character.collection_id);
                        }
                        ui.heading("Edit Character");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.menu_button("EXPORT", |ui| {
                                let name_slug = character.name.replace(" ", "_");
                                
                                if ui.button("Native (.crapp)").clicked() {
                                    if let Ok(json) = serde_json::to_string_pretty(&character) {
                                        let task_name = format!("{}.crapp", name_slug);
                                        let task_json = json.clone();
                                        tokio::spawn(async move {
                                            if let Some(path) = rfd::FileDialog::new()
                                                .set_directory("exports")
                                                .set_file_name(task_name)
                                                .save_file() 
                                            {
                                                let _ = std::fs::write(path, task_json);
                                            }
                                        });
                                    }
                                    ui.close_menu();
                                }
                                
                                if ui.button("Character Card - spicychat.ai (.json)").clicked() {
                                    let v2 = CharacterCardV2::new(
                                        character.char_name.clone(),
                                        character.personality.clone(),
                                        character.char_title.clone(),
                                        character.scenario.clone(),
                                        character.first_message.clone(),
                                        character.example_dialogue.clone(),
                                    );
                                    if let Ok(json) = serde_json::to_string_pretty(&v2) {
                                        let task_name = format!("{}.json", name_slug);
                                        tokio::spawn(async move {
                                            if let Some(path) = rfd::FileDialog::new()
                                                .set_directory("exports")
                                                .set_file_name(task_name)
                                                .save_file() 
                                            {
                                                let _ = std::fs::write(path, json);
                                            }
                                        });
                                    }
                                    ui.close_menu();
                                }
                                
                                if ui.button("Document (.md)").clicked() {
                                    let md = format!(
                                        "# {}\n\n## Description\n{}\n\n## Personality\n{}\n\n## Scenario\n{}\n\n## First Message\n{}\n\n## Example Dialogue\n{}\n",
                                        character.char_name, character.char_title, character.personality, character.scenario, character.first_message, character.example_dialogue
                                    );
                                    let task_name = format!("{}.md", name_slug);
                                    tokio::spawn(async move {
                                        if let Some(path) = rfd::FileDialog::new()
                                            .set_directory("exports")
                                            .set_file_name(task_name)
                                            .save_file() 
                                        {
                                            let _ = std::fs::write(path, md);
                                        }
                                    });
                                    ui.close_menu();
                                }
                                
                                if ui.button("Character Card (.png)").clicked() {
                                    if let Some(avatar_path) = &character.avatar_path {
                                        // Logic to export PNG
                                        let v2 = CharacterCardV2::new(
                                            character.char_name.clone(),
                                            character.personality.clone(),
                                            character.char_title.clone(),
                                            character.scenario.clone(),
                                            character.first_message.clone(),
                                            character.example_dialogue.clone(),
                                        );
                                        
                                        if let Ok(json) = serde_json::to_string(&v2) {
                                            let b64 = BASE64.encode(json);
                                            let path_clone = avatar_path.clone(); // Path string
                                            let task_name = format!("{}.png", name_slug);
                                            
                                            // Clone tx to report status if needed, though we can't easily set status from async without event 
                                            // Assuming "app" is not available here easily (it is captured?) - wait, "character" is borrowed from app.
                                            // We need to be careful with app access. "app" is available in outer scope but mutable borrow of "character" exists.
                                            // So we cannot touch app.
                                            
                                            // We'll spawn the IO.
                                            tokio::spawn(async move {
                                                // 1. Read Valid Image
                                                if let Ok(img_bytes) = std::fs::read(&path_clone) {
                                                    if let Some(save_path) = rfd::FileDialog::new()
                                                        .set_directory("exports")
                                                        .set_file_name(task_name)
                                                        .save_file() 
                                                    {
                                                        // 2. Decode to get raw pixels (via image crate) to ensure clean state or just append?
                                                        // Appending to existing PNG is risky if it has other chunks.
                                                        // Best is: Decode -> Encode with new chunk.
                                                        
                                                        if let Ok(img) = image::load_from_memory(&img_bytes) {
                                                            let (w, h) = (img.width(), img.height());
                                                            let color_type = img.color();
                                                            let pixels = img.into_bytes();
                                                            
                                                            let mut out_file = std::fs::File::create(save_path).unwrap();
                                                            
                                                            {
                                                                let mut encoder = png::Encoder::new(&mut out_file, w, h);
                                                                encoder.set_color(match color_type {
                                                                    image::ColorType::Rgb8 => png::ColorType::Rgb,
                                                                    image::ColorType::Rgba8 => png::ColorType::Rgba,
                                                                    image::ColorType::L8 => png::ColorType::Grayscale,
                                                                    image::ColorType::La8 => png::ColorType::GrayscaleAlpha,
                                                                    _ => png::ColorType::Rgba, // Fallback
                                                                });
                                                                encoder.set_depth(png::BitDepth::Eight);
                                                                                                                            
                                                                let mut writer = encoder.write_header().expect("Failed to write PNG header");
                                                                // Add tEXt chunk
                                                                let chunk = png::text_metadata::ITXtChunk::new("chara".to_string(), b64.to_string());
                                                                if writer.write_text_chunk(&chunk).is_ok() {
                                                                    let _ = writer.write_image_data(&pixels);
                                                                    let _ = writer.finish();
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                    } else {
                                        // Cannot set status here because 'character' is borrowing 'app'.
                                        // Ideally we should flag this error visually or deferred.
                                        // For now, let's just print to stderr/log, or we need a UI feedback mechanism that doesn't borrow app.
                                        // We can use a local flag in the UI loop if we structure differently.
                                        // But here we are inside the button closure.
                                        eprintln!("Cannot export PNG: No avatar.");
                                    }
                                    ui.close_menu();
                                }
                            });
                            ui.menu_button("IMPORT", |ui| {
                                if ui.button("Load from .crapp file").clicked() {
                                    let tx_clone = app.tx.clone();
                                    tokio::spawn(async move {
                                        if let Some(path) = rfd::FileDialog::new().add_filter("Native", &["crapp", "json"]).pick_file() {
                                            let res = std::fs::read_to_string(path).map_err(|e| e.to_string());
                                            let _ = tx_clone.send(UiEvent::ImportFileLoaded(res)).await;
                                        }
                                    });
                                    ui.close_menu();
                                }
                                
                                if ui.button("Paste from Clipboard").clicked() {
                                    trigger_import = true;
                                    ui.close_menu();
                                }
                            });
                            
                            ui.add_space(10.0);
                            if app.is_saving {
                                ui.spinner();
                                ui.label("Saving...");
                            } else {
                                // Actually, we need to construct the button logic *before* adding.
                                let mut save_color = ui.visuals().widgets.inactive.bg_fill;
                                if is_dirty {
                                     save_color = egui::Color32::from_rgb(200, 100, 50); // Orange/Red
                                }
                                
                                // Check for Ctrl+S
                                if ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::S)) {
                                     save_req = Some(character.clone());
                                }

                                if ui.add(egui::Button::new(egui::RichText::new("SAVE").strong()).fill(save_color)).clicked() {
                                    save_req = Some(character.clone());
                                }
                                if let Some((msg, color)) = &app.status_message {
                                    ui.colored_label(*color, msg);
                                }
                            }
                        });
                    });
                     
                    ui.horizontal(|ui| {
                        ui.label("Collection:");
                        let current_col_name = character.collection_id.and_then(|id| {
                            collection_options.iter().find(|(cid, _)| *cid == id).map(|(_, name)| name.clone())
                        }).unwrap_or_else(|| "Uncategorized".to_string());
                        
                        egui::ComboBox::from_id_salt("collection_combo")
                            .selected_text(current_col_name)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut character.collection_id, None, "Uncategorized");
                                for (id, name) in &collection_options {
                                    ui.selectable_value(&mut character.collection_id, Some(*id), name);
                                }
                            });
                        
                        ui.add_space(8.0);
                        let fav_btn = if character.is_favorite {
                            egui::Button::new(egui::RichText::new("\u{2764} Favorite").color(egui::Color32::WHITE)).fill(egui::Color32::from_rgb(200, 50, 50))
                        } else {
                             egui::Button::new("\u{2764} Favorite")
                        };
                        
                        if ui.add(fav_btn).clicked() {
                            character.is_favorite = !character.is_favorite;
                        }
                    });
                    
                    // In-editor search
                    ui.horizontal(|ui| {
                        ui.label("🔍 Search:");
                        ui.add(
                            egui::TextEdit::singleline(&mut app.editor_search_query)
                                .hint_text("Type 3+ chars to highlight...")
                                .desired_width(200.0)
                        );
                        
                        if !app.editor_search_query.is_empty() {
                            if ui.small_button("✖").clicked() {
                                app.editor_search_query.clear();
                            }
                            
                            ui.label(
                                egui::RichText::new(format!("Highlighting: '{}'", app.editor_search_query))
                                    .size(11.0)
                                    .color(egui::Color32::GRAY)
                            );
                        }
                    });
                    ui.separator();

                     // Tabs
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut app.active_char_tab, CharacterTab::MainData, "Main Data");
                        ui.selectable_value(&mut app.active_char_tab, CharacterTab::Notes, "Notes");
                    ui.selectable_value(&mut app.active_char_tab, CharacterTab::Lorebooks, "Lorebooks");
                    });
                    ui.separator();

                    // Handle Drag and Drop for Avatar
                    let dropped_files = ui.input(|i| i.raw.dropped_files.clone());
                    if !dropped_files.is_empty() {
                        for dropped in dropped_files {
                            if let Some(path) = dropped.path {
                                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                                if ["png", "jpg", "jpeg", "webp"].contains(&ext.as_str()) {
                                    let dest_dir = std::path::Path::new("data/avatars");
                                    let _ = std::fs::create_dir_all(dest_dir);
                                    if let Some(name) = path.file_name() {
                                        let dest = dest_dir.join(name);
                                        if let Ok(_) = std::fs::copy(&path, &dest) {
                                            character.avatar_path = Some(dest.to_string_lossy().to_string());
                                            status_update = Some(("Avatar loaded from dropped file!".to_string(), egui::Color32::GREEN));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    let mut tag_add_request: Option<(i64, String, bool)> = None;
                    let mut tag_remove_request: Option<(i64, i64, bool)> = None;
                    
                     egui::ScrollArea::vertical().show(ui, |ui| {
                         match app.active_char_tab {
                             CharacterTab::MainData => {
                                 ui.horizontal(|ui| {
                                     let available_width = ui.available_width();
                                     let left_width = available_width * 0.66;
                                     // Right width is remaining
                                     
                                     ui.allocate_ui_with_layout(egui::vec2(left_width, ui.available_height()), egui::Layout::top_down(egui::Align::Min), |ui| {
                                         ui.label("Name (File Name)");
                                         // File Name (character.name) with search highlight
                                          if app.editor_search_query.len() >= 3 {
                                               let mut layouter = crate::ui::text_highlight::create_highlight_layouter(app.editor_search_query.clone());
                                               let response = ui.add(egui::TextEdit::singleline(&mut character.name).layouter(&mut *layouter));
                                               crate::ui::widgets::track_text_selection(ui, &response);
                                               response.context_menu(|ui| {
                                                   crate::ui::widgets::text_context_menu(ui, &mut character.name, response.id);
                                               });
                                          } else {
                                               let response = ui.add(egui::TextEdit::singleline(&mut character.name).desired_width(f32::INFINITY));
                                               crate::ui::widgets::track_text_selection(ui, &response);
                                               response.context_menu(|ui| {
                                                   crate::ui::widgets::text_context_menu(ui, &mut character.name, response.id);
                                               });
                                          }


                                         ui.label("Character Name");
                                         // Character Name (character.char_name) with search highlight
                                          if app.editor_search_query.len() >= 3 {
                                               let mut layouter = crate::ui::text_highlight::create_highlight_layouter(app.editor_search_query.clone());
                                               let response = ui.add(egui::TextEdit::singleline(&mut character.char_name).layouter(&mut *layouter));
                                               crate::ui::widgets::track_text_selection(ui, &response);
                                               response.context_menu(|ui| {
                                                   crate::ui::widgets::text_context_menu(ui, &mut character.char_name, response.id);
                                               });
                                          } else {
                                               let response = ui.add(egui::TextEdit::singleline(&mut character.char_name).desired_width(f32::INFINITY));
                                               crate::ui::widgets::track_text_selection(ui, &response);
                                               response.context_menu(|ui| {
                                                   crate::ui::widgets::text_context_menu(ui, &mut character.char_name, response.id);
                                               });
                                          }

                                         // Title (character.char_title) with search highlight AND auto-resize
                                         // Changed to multiline with min_rows(1) for auto-resize behavior
                                         let title_edit = egui::TextEdit::multiline(&mut character.char_title)
                                             .desired_width(f32::INFINITY)
                                             .desired_rows(1);
                                                                                  if app.editor_search_query.len() >= 3 {
                                              let mut layouter = crate::ui::text_highlight::create_highlight_layouter(app.editor_search_query.clone());
                                              let response = ui.add(title_edit.layouter(&mut *layouter));
                                              crate::ui::widgets::track_text_selection(ui, &response);
                                              response.context_menu(|ui| {
                                                   crate::ui::widgets::text_context_menu(ui, &mut character.char_title, response.id);
                                               });
                                          } else {
                                              let response = ui.add(title_edit);
                                              crate::ui::widgets::track_text_selection(ui, &response);
                                               response.context_menu(|ui| {
                                                   crate::ui::widgets::text_context_menu(ui, &mut character.char_title, response.id);
                                               });
                                          }


                                         ui.add_space(8.0);
                                         
                                         egui::CollapsingHeader::new("First Message")
                                             .default_open(true)
                                             .show(ui, |ui| {
                                                 let mut text_edit = egui::TextEdit::multiline(&mut character.first_message).desired_width(f32::INFINITY);
                                                  if app.editor_search_query.len() >= 3 {
                                                      let search_query = app.editor_search_query.clone();
                                                      let mut my_layouter = move |ui: &egui::Ui, text: &str, wrap_width: f32| {
                                                          let mut layout_job = egui::text::LayoutJob::default();
                                                          let search_lower = search_query.to_lowercase();
                                                          let text_lower = text.to_lowercase();
                                                          let mut last_end = 0;
                                                          
                                                          let font_id = egui::TextStyle::Body.resolve(ui.style());
                                                          let default_format = egui::TextFormat {
                                                              font_id: font_id.clone(),
                                                              ..Default::default()
                                                          };
                                                          
                                                          for (start, _) in text_lower.match_indices(&search_lower) {
                                                              let end = start + search_query.len();
                                                              if start > last_end {
                                                                  layout_job.append(&text[last_end..start], 0.0, default_format.clone());
                                                              }
                                                              layout_job.append(&text[start..end], 0.0, egui::TextFormat {
                                                                  font_id: font_id.clone(),
                                                                  background: egui::Color32::from_rgb(255, 255, 0),
                                                                  color: egui::Color32::BLACK,
                                                                  ..Default::default()
                                                              });
                                                              last_end = end;
                                                          }
                                                          if last_end < text.len() {
                                                              layout_job.append(&text[last_end..], 0.0, default_format);
                                                          }
                                                          layout_job.wrap.max_width = wrap_width;
                                                      ui.fonts(|f| f.layout_job(layout_job))
                                                      };
                                                      let response = ui.add(text_edit.layouter(&mut my_layouter));
                                                      crate::ui::widgets::track_text_selection(ui, &response);
                                                      response.context_menu(|ui| {
                                                          crate::ui::widgets::text_context_menu(ui, &mut character.first_message, response.id);
                                                      });
                                                  } else {
                                                      let response = ui.add(text_edit);
                                                      crate::ui::widgets::track_text_selection(ui, &response);
                                                      response.context_menu(|ui| {
                                                          crate::ui::widgets::text_context_menu(ui, &mut character.first_message, response.id);
                                                      });
                                                  }
                                                 ui.horizontal(|ui| {
                                                     ui.label(egui::RichText::new(format!("Tokens: {} | Chars: {}", count_tokens(&character.first_message), character.first_message.chars().count())).size(12.0).color(egui::Color32::GRAY));
                                                     ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                         if ui.small_button("Copy").clicked() {
                                                             ui.output_mut(|o| o.copied_text = character.first_message.clone());
                                                             status_update = Some(("Copied First Message to clipboard".to_string(), egui::Color32::GREEN));
                                                         }
                                                     });
                                                 });
                                             });
                                         
                                         ui.add_space(8.0);
                                         egui::CollapsingHeader::new("Personality")
                                             .default_open(true)
                                             .show(ui, |ui| {
                                                 let mut text_edit = egui::TextEdit::multiline(&mut character.personality).desired_width(f32::INFINITY);
                                                  if app.editor_search_query.len() >= 3 {
                                                      let search_query = app.editor_search_query.clone();
                                                      let mut my_layouter = move |ui: &egui::Ui, text: &str, wrap_width: f32| {
                                                          let mut layout_job = egui::text::LayoutJob::default();
                                                          let search_lower = search_query.to_lowercase();
                                                          let text_lower = text.to_lowercase();
                                                          let mut last_end = 0;
                                                          
                                                          let font_id = egui::TextStyle::Body.resolve(ui.style());
                                                          let default_format = egui::TextFormat {
                                                              font_id: font_id.clone(),
                                                              ..Default::default()
                                                          };
                                                          
                                                          for (start, _) in text_lower.match_indices(&search_lower) {
                                                              let end = start + search_query.len();
                                                              if start > last_end {
                                                                  layout_job.append(&text[last_end..start], 0.0, default_format.clone());
                                                              }
                                                              layout_job.append(&text[start..end], 0.0, egui::TextFormat {
                                                                  font_id: font_id.clone(),
                                                                  background: egui::Color32::from_rgb(255, 255, 0),
                                                                  color: egui::Color32::BLACK,
                                                                  ..Default::default()
                                                              });
                                                              last_end = end;
                                                          }
                                                          if last_end < text.len() {
                                                              layout_job.append(&text[last_end..], 0.0, default_format);
                                                          }
                                                          layout_job.wrap.max_width = wrap_width;
                                                          ui.fonts(|f| f.layout_job(layout_job))
                                                      };
                                                      let response = ui.add(text_edit.layouter(&mut my_layouter));
                                                      crate::ui::widgets::track_text_selection(ui, &response);
                                                      response.context_menu(|ui| {
                                                          crate::ui::widgets::text_context_menu(ui, &mut character.personality, response.id);
                                                      });
                                                  } else {
                                                      let response = ui.add(text_edit);
                                                      crate::ui::widgets::track_text_selection(ui, &response);
                                                      response.context_menu(|ui| {
                                                          crate::ui::widgets::text_context_menu(ui, &mut character.personality, response.id);
                                                      });
                                                  }
                                                 ui.horizontal(|ui| {
                                                     ui.label(egui::RichText::new(format!("Tokens: {} | Chars: {}", count_tokens(&character.personality), character.personality.chars().count())).size(12.0).color(egui::Color32::GRAY));
                                                     ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                         if ui.small_button("Copy").clicked() {
                                                             ui.output_mut(|o| o.copied_text = character.personality.clone());
                                                             status_update = Some(("Copied Personality to clipboard".to_string(), egui::Color32::GREEN));
                                                         }
                                                     });
                                                 });
                                             });
        
                                         egui::CollapsingHeader::new("Scenario")
                                             .default_open(true)
                                             .show(ui, |ui| {
                                                 let mut text_edit = egui::TextEdit::multiline(&mut character.scenario).desired_width(f32::INFINITY);
                                                  if app.editor_search_query.len() >= 3 {
                                                      let search_query = app.editor_search_query.clone();
                                                      let mut my_layouter = move |ui: &egui::Ui, text: &str, wrap_width: f32| {
                                                          let mut layout_job = egui::text::LayoutJob::default();
                                                          let search_lower = search_query.to_lowercase();
                                                          let text_lower = text.to_lowercase();
                                                          let mut last_end = 0;
                                                          
                                                          let font_id = egui::TextStyle::Body.resolve(ui.style());
                                                          let default_format = egui::TextFormat {
                                                              font_id: font_id.clone(),
                                                              ..Default::default()
                                                          };
                                                          
                                                          for (start, _) in text_lower.match_indices(&search_lower) {
                                                              let end = start + search_query.len();
                                                              if start > last_end {
                                                                  layout_job.append(&text[last_end..start], 0.0, default_format.clone());
                                                              }
                                                              layout_job.append(&text[start..end], 0.0, egui::TextFormat {
                                                                  font_id: font_id.clone(),
                                                                  background: egui::Color32::from_rgb(255, 255, 0),
                                                                  color: egui::Color32::BLACK,
                                                                  ..Default::default()
                                                              });
                                                              last_end = end;
                                                          }
                                                          if last_end < text.len() {
                                                              layout_job.append(&text[last_end..], 0.0, default_format);
                                                          }
                                                          layout_job.wrap.max_width = wrap_width;
                                                          ui.fonts(|f| f.layout_job(layout_job))
                                                      };
                                                      let response = ui.add(text_edit.layouter(&mut my_layouter));
                                                      crate::ui::widgets::track_text_selection(ui, &response);
                                                      response.context_menu(|ui| {
                                                          crate::ui::widgets::text_context_menu(ui, &mut character.scenario, response.id);
                                                      });
                                                  } else {
                                                      let response = ui.add(text_edit);
                                                      crate::ui::widgets::track_text_selection(ui, &response);
                                                      response.context_menu(|ui| {
                                                          crate::ui::widgets::text_context_menu(ui, &mut character.scenario, response.id);
                                                      });
                                                  }
                                                 ui.horizontal(|ui| {
                                                     ui.label(egui::RichText::new(format!("Tokens: {} | Chars: {}", count_tokens(&character.scenario), character.scenario.chars().count())).size(12.0).color(egui::Color32::GRAY));
                                                     ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        if ui.small_button("Copy").clicked() {
                                                             ui.output_mut(|o| o.copied_text = character.scenario.clone());
                                                             status_update = Some(("Copied Scenario to clipboard".to_string(), egui::Color32::GREEN));
                                                         }
                                                     });
                                                 });
                                             });
        
                                         egui::CollapsingHeader::new("Example Dialogue")
                                             .default_open(true)
                                             .show(ui, |ui| {
                                                 let mut text_edit = egui::TextEdit::multiline(&mut character.example_dialogue).desired_width(f32::INFINITY);
                                                  if app.editor_search_query.len() >= 3 {
                                                      let search_query = app.editor_search_query.clone();
                                                      let mut my_layouter = move |ui: &egui::Ui, text: &str, wrap_width: f32| {
                                                          let mut layout_job = egui::text::LayoutJob::default();
                                                          let search_lower = search_query.to_lowercase();
                                                          let text_lower = text.to_lowercase();
                                                          let mut last_end = 0;
                                                          
                                                          let font_id = egui::TextStyle::Body.resolve(ui.style());
                                                          let default_format = egui::TextFormat {
                                                              font_id: font_id.clone(),
                                                              ..Default::default()
                                                          };
                                                          
                                                          for (start, _) in text_lower.match_indices(&search_lower) {
                                                              let end = start + search_query.len();
                                                              if start > last_end {
                                                                  layout_job.append(&text[last_end..start], 0.0, default_format.clone());
                                                              }
                                                              layout_job.append(&text[start..end], 0.0, egui::TextFormat {
                                                                  font_id: font_id.clone(),
                                                                  background: egui::Color32::from_rgb(255, 255, 0),
                                                                  color: egui::Color32::BLACK,
                                                                  ..Default::default()
                                                              });
                                                              last_end = end;
                                                          }
                                                          if last_end < text.len() {
                                                              layout_job.append(&text[last_end..], 0.0, default_format);
                                                          }
                                                          layout_job.wrap.max_width = wrap_width;
                                                          ui.fonts(|f| f.layout_job(layout_job))
                                                      };
                                                      let response = ui.add(text_edit.layouter(&mut my_layouter));
                                                      crate::ui::widgets::track_text_selection(ui, &response);
                                                      response.context_menu(|ui| {
                                                          crate::ui::widgets::text_context_menu(ui, &mut character.example_dialogue, response.id);
                                                      });
                                                  } else {
                                                      let response = ui.add(text_edit);
                                                      crate::ui::widgets::track_text_selection(ui, &response);
                                                      response.context_menu(|ui| {
                                                          crate::ui::widgets::text_context_menu(ui, &mut character.example_dialogue, response.id);
                                                      });
                                                  }
                                                 ui.horizontal(|ui| {
                                                     ui.label(egui::RichText::new(format!("Tokens: {} | Chars: {}", count_tokens(&character.example_dialogue), character.example_dialogue.chars().count())).size(12.0).color(egui::Color32::GRAY));
                                                     ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                         if ui.small_button("Copy").clicked() {
                                                             ui.output_mut(|o| o.copied_text = character.example_dialogue.clone());
                                                             status_update = Some(("Copied Example Dialogue to clipboard".to_string(), egui::Color32::GREEN));
                                                         }
                                                     });
                                                 });
                                             });
                                         
                                         egui::CollapsingHeader::new("Tags & Metadata")
                                            .default_open(true)
                                            .show(ui, |ui| {
                                                ui.vertical(|ui| {
                                                     // App Tags
                                                    ui.label(egui::RichText::new("CRApp Tags").strong().color(egui::Color32::from_rgb(100, 150, 255)));
                                                    ui.horizontal(|ui| {
                                                        let mut app_tags_sorted: Vec<_> = character.app_tags.iter().collect();
                                                        app_tags_sorted.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                                                        for tag in app_tags_sorted {
                                                            egui::Frame::none().fill(egui::Color32::from_rgb(50, 80, 150)).rounding(12.0).inner_margin(4.0).show(ui, |ui| {
                                                                ui.horizontal(|ui| {
                                                                    ui.label(egui::RichText::new(&tag.name).color(egui::Color32::WHITE).size(12.0));
                                                                    if ui.small_button("x").clicked() {
                                                                         tag_remove_request = Some((character.id, tag.id, false));
                                                                    }
                                                                });
                                                            });
                                                        }
                                                    });
                                                    ui.horizontal(|ui| {
                                                        let response = ui.text_edit_singleline(&mut app.app_tag_input);
                                                        if (ui.button("Add").clicked() || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))) && !app.app_tag_input.is_empty() {
                                                            tag_add_request = Some((character.id, app.app_tag_input.clone(), false));
                                                            app.app_tag_input.clear();
                                                            response.request_focus();
                                                        }
                                                    });
                                                    
                                                    ui.add_space(8.0);
                                                    
                                                    // External Tags
                                                    ui.label(egui::RichText::new("External Tags").strong().color(egui::Color32::GRAY));
                                                    ui.horizontal(|ui| {
                                                        let mut ext_tags_sorted: Vec<_> = character.external_tags.iter().collect();
                                                        ext_tags_sorted.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                                                        for tag in ext_tags_sorted {
                                                            egui::Frame::none().fill(egui::Color32::from_gray(80)).rounding(12.0).inner_margin(4.0).show(ui, |ui| {
                                                                ui.horizontal(|ui| {
                                                                    ui.label(egui::RichText::new(&tag.name).color(egui::Color32::WHITE).size(12.0));
                                                                    if ui.small_button("x").clicked() {
                                                                         tag_remove_request = Some((character.id, tag.id, true));
                                                                    }
                                                                });
                                                            });
                                                        }
                                                    });
                                                    ui.horizontal(|ui| {
                                                        let response = ui.text_edit_singleline(&mut app.ext_tag_input);
                                                        if (ui.button("Add").clicked() || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))) && !app.ext_tag_input.is_empty() {
                                                             tag_add_request = Some((character.id, app.ext_tag_input.clone(), true));
                                                             app.ext_tag_input.clear();
                                                             response.request_focus();
                                                        }
                                                    });
                                                });
                                            });
                                     });
                                     
                                     ui.add_space(8.0);
                                     
                                     ui.vertical(|ui| {
                                         ui.label("Avatar");
                                         
                                         // Show image preview if available
                                         if let Some(path_str) = &character.avatar_path {
                                             let uri = if path_str.contains("://") { 
                                                 path_str.clone() 
                                             } else {
                                                 if let Ok(abs_path) = std::fs::canonicalize(path_str) {
                                                      format!("file://{}", abs_path.to_string_lossy())
                                                 } else {
                                                      path_str.clone() 
                                                 }
                                             };
                                             
                                             // Calculate preview size based on available width in this column
                                             let preview_width = ui.available_width() - 8.0;
                                             ui.add(egui::Image::new(uri)
                                                 .rounding(egui::Rounding::same(4.0))
                                                 .fit_to_original_size(0.5) // Adjust scaling logic if needed or use max_width
                                                 .max_width(preview_width));
                                              
                                             ui.label(path_str);
                                             
                                             ui.horizontal(|ui| {
                                                 if ui.button("Copy to Clipboard").clicked() {
                                                     match std::fs::read(path_str) {
                                                         Ok(bytes) => {
                                                             match image::load_from_memory(&bytes) {
                                                                 Ok(dynamic_img) => {
                                                                     let rgba = dynamic_img.to_rgba8();
                                                                     let img_data = arboard::ImageData {
                                                                         width: rgba.width() as usize,
                                                                         height: rgba.height() as usize,
                                                                         bytes: std::borrow::Cow::from(rgba.into_raw()),
                                                                     };
                                                                     
                                                                     match arboard::Clipboard::new() {
                                                                         Ok(mut clipboard) => {
                                                                             if let Err(e) = clipboard.set_image(img_data) {
                                                                                 status_update = Some((format!("Failed to copy to clipboard: {}", e), egui::Color32::RED));
                                                                             } else {
                                                                                 status_update = Some(("Avatar copied to clipboard!".to_string(), egui::Color32::GREEN));
                                                                             }
                                                                         },
                                                                         Err(e) => {
                                                                             status_update = Some((format!("Clipboard access failed: {}", e), egui::Color32::RED));
                                                                         }
                                                                     }
                                                                 },
                                                                 Err(e) => {
                                                                     status_update = Some((format!("Failed to load image: {}", e), egui::Color32::RED));
                                                                 }
                                                             }
                                                         },
                                                         Err(e) => {
                                                             status_update = Some((format!("Failed to read avatar file: {}", e), egui::Color32::RED));
                                                         }
                                                     }
                                                 }
 
                                                 if ui.button("Open Folder").clicked() {
                                                     #[cfg(target_os = "windows")]
                                                     {
                                                         let _ = std::process::Command::new("explorer")
                                                             .arg("/select,")
                                                             .arg(path_str.replace("/", "\\"))
                                                             .spawn();
                                                     }
 
                                                     #[cfg(target_os = "linux")]
                                                     {
                                                         if let Ok(abs_path) = std::fs::canonicalize(path_str) {
                                                             let file_uri = format!("file://{}", abs_path.to_string_lossy());
                                                             // Try D-Bus for selection first (standard modern Linux)
                                                             let status = std::process::Command::new("dbus-send")
                                                                 .args(&[
                                                                     "--session",
                                                                     "--dest=org.freedesktop.FileManager1",
                                                                     "--type=method_call",
                                                                     "/org/freedesktop/FileManager1",
                                                                     "org.freedesktop.FileManager1.ShowItems",
                                                                     &format!("array:string:{}", file_uri),
                                                                     "string:\"\"",
                                                                 ])
                                                                 .status();
                                                             
                                                             if status.is_err() || !status.unwrap().success() {
                                                                 // Fallback to just opening the parent directory
                                                                 if let Some(parent) = abs_path.parent() {
                                                                     let _ = std::process::Command::new("xdg-open")
                                                                         .arg(parent)
                                                                         .spawn();
                                                                 }
                                                             }
                                                         }
                                                     }
 
                                                     #[cfg(target_os = "macos")]
                                                     {
                                                         let _ = std::process::Command::new("open")
                                                             .arg("-R")
                                                             .arg(path_str)
                                                             .spawn();
                                                     }
                                                 }
                                             });
                                             
                                         } else {
                                             ui.label(egui::RichText::new("No avatar selected").italics());
                                         }
                                         
                                         ui.horizontal(|ui| {
                                             if ui.button("Browse Avatar").clicked() {
                                                 if let Some(path) = rfd::FileDialog::new().add_filter("image", &["png", "jpg", "jpeg"]).pick_file() {
                                                      let dest_dir = std::path::Path::new("data/avatars");
                                                      let _ = std::fs::create_dir_all(dest_dir);
                                                      if let Some(name) = path.file_name() {
                                                          let dest = dest_dir.join(name);
                                                          let _ = std::fs::copy(&path, &dest);
                                                          character.avatar_path = Some(dest.to_string_lossy().to_string());
                                                      }
                                                 }
                                             }
                                             
                                             if ui.button("Paste from Clipboard").clicked() {
                                                  match arboard::Clipboard::new() {
                                                      Ok(mut clipboard) => {
                                                          match clipboard.get_image() {
                                                              Ok(img_data) => {
                                                                 let width = img_data.width as u32;
                                                                 let height = img_data.height as u32;
                                                                 
                                                                 // Convert Cow<'a, [u8]> to Vec<u8>
                                                                 let bytes = img_data.bytes.into_owned();
                                                                 
                                                                 if let Some(image_buffer) = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(width, height, bytes) {
                                                                      let timestamp = chrono::Utc::now().timestamp_millis();
                                                                      let filename = format!("pasted_avatar_{}.png", timestamp);
                                                                      let dest_dir = std::path::Path::new("data/avatars");
                                                                      let _ = std::fs::create_dir_all(dest_dir);
                                                                      let dest_path = dest_dir.join(&filename);
                                                                      
                                                                      if let Ok(_) = image_buffer.save(&dest_path) {
                                                                          character.avatar_path = Some(dest_path.to_string_lossy().to_string());
                                                                          status_update = Some(("Avatar pasted successfully!".to_string(), egui::Color32::GREEN));
                                                                      } else {
                                                                           status_update = Some(("Failed to save avatar image to disk.".to_string(), egui::Color32::RED));
                                                                      }
                                                                 } else {
                                                                      status_update = Some(("Failed to process image buffer from clipboard.".to_string(), egui::Color32::RED));
                                                                 }
                                                              },
                                                              Err(_) => {
                                                                  status_update = Some(("Clipboard does not contain an image.".to_string(), egui::Color32::RED));
                                                              }
                                                          }
                                                      },
                                                      Err(e) => {
                                                          status_update = Some((format!("Clipboard access failed: {}", e), egui::Color32::RED));
                                                      }
                                                  }
                                             }
                                         });
                                         
                                         ui.add_space(8.0);
                                         
                                         // Token Summary
                                         let t_first = count_tokens(&character.first_message);
                                         let t_pers = count_tokens(&character.personality);
                                         let t_scen = count_tokens(&character.scenario);
                                         let t_ex = count_tokens(&character.example_dialogue);
                                         
                                         let total_tokens = t_first + t_pers + t_scen + t_ex;
                                         let perm_tokens = t_pers + t_scen;
                                         
                                         ui.label(egui::RichText::new(format!("Total Tokens: {} (Permanent: {})", total_tokens, perm_tokens))
                                             .strong()
                                             .color(egui::Color32::WHITE));
                                     });
                                 });
                             },
                             CharacterTab::Notes => {
                                 ui.label("Notes");
                                 let width = ui.ctx().screen_rect().width() * 2.0 / 3.0;
                                 let mut text_edit = egui::TextEdit::multiline(&mut character.author_notes).desired_width(width);
                                  if app.editor_search_query.len() >= 3 {
                                      let search_query = app.editor_search_query.clone();
                                      let mut my_layouter = move |ui: &egui::Ui, text: &str, wrap_width: f32| {
                                          let mut layout_job = egui::text::LayoutJob::default();
                                          let search_lower = search_query.to_lowercase();
                                          let text_lower = text.to_lowercase();
                                          let mut last_end = 0;
                                                          
                                                          let font_id = egui::TextStyle::Body.resolve(ui.style());
                                                          let default_format = egui::TextFormat {
                                                              font_id: font_id.clone(),
                                                              ..Default::default()
                                                          };
                                          for (start, _) in text_lower.match_indices(&search_lower) {
                                              let end = start + search_query.len();
                                              if start > last_end {
                                                  layout_job.append(&text[last_end..start], 0.0, default_format.clone());
                                              }
                                              layout_job.append(&text[start..end], 0.0, egui::TextFormat {
                                                  font_id: font_id.clone(),
                                                  background: egui::Color32::from_rgb(255, 255, 0),
                                                  color: egui::Color32::BLACK,
                                                  ..Default::default()
                                              });
                                              last_end = end;
                                          }
                                          if last_end < text.len() {
                                              layout_job.append(&text[last_end..], 0.0, default_format);
                                          }
                                          layout_job.wrap.max_width = wrap_width;
                                          ui.fonts(|f| f.layout_job(layout_job))
                                      };
                                      let response = ui.add(text_edit.layouter(&mut my_layouter));
                                      crate::ui::widgets::track_text_selection(ui, &response);
                                      response.context_menu(|ui| {
                                          crate::ui::widgets::text_context_menu(ui, &mut character.author_notes, response.id);
                                      });
                                  } else {
                                      let response = ui.add(text_edit);
                                      crate::ui::widgets::track_text_selection(ui, &response);
                                      response.context_menu(|ui| {
                                          crate::ui::widgets::text_context_menu(ui, &mut character.author_notes, response.id);
                                      });
                                  }

                                 ui.add_space(16.0);
                                 ui.separator();
                                 ui.heading("Character Source URLs");
                                 ui.label(egui::RichText::new("Links to where this character is hosted (e.g. spicychat.ai, janitor.ai)").size(11.0).color(egui::Color32::GRAY));

                                 // Ensure there is always one empty slot at the end
                                 if character.urls.is_empty() || !character.urls.last().unwrap().url.is_empty() {
                                     character.urls.push(crate::models::CharacterUrl {
                                         id: 0,
                                         character_id: character.id,
                                         url: String::new(),
                                         label: None,
                                     });
                                 }

                                 let mut urls_to_remove = Vec::new();

                                 // Iterate with index to allow removal
                                 for (i, char_url) in character.urls.iter_mut().enumerate() {
                                     ui.horizontal(|ui| {
                                         ui.label("URL:");
                                         let url_resp = ui.add(egui::TextEdit::singleline(&mut char_url.url).desired_width(250.0).hint_text("https://..."));

                                         ui.label("Service:");
                                         let mut label_val = char_url.label.clone().unwrap_or_default();
                                         let _ = ui.add(egui::TextEdit::singleline(&mut label_val).desired_width(100.0).hint_text("Auto"));

                                         if label_val.is_empty() {
                                             char_url.label = None;
                                         } else {
                                             char_url.label = Some(label_val);
                                         }
                                         
                                         if !char_url.url.is_empty() {
                                             if ui.button("🌐").on_hover_text("Open in Browser").clicked() {
                                                 ui.ctx().open_url(egui::OpenUrl::new_tab(&char_url.url));
                                             }
                                         }

                                         // Auto-fill label logic
                                         if url_resp.changed() || url_resp.lost_focus() {
                                             if char_url.label.is_none() || char_url.label.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                                                  // Try to extract domain
                                                  if let Ok(parsed) = url::Url::parse(&char_url.url) {
                                                      if let Some(host) = parsed.host_str() {
                                                          let clean = host.replace("www.", "");
                                                          let parts: Vec<&str> = clean.split('.').collect();
                                                          if !parts.is_empty() {
                                                              let service_name = parts[0];
                                                              let mut c = service_name.chars();
                                                              if let Some(f) = c.next() {
                                                                 let cap = f.to_uppercase().collect::<String>() + c.as_str();
                                                                 char_url.label = Some(cap);
                                                              } else {
                                                                 char_url.label = Some(service_name.to_string());
                                                              }
                                                          }
                                                      }
                                                  }
                                             }
                                         }

                                         if !char_url.url.is_empty() {
                                              if ui.button("🗑").clicked() {
                                                  urls_to_remove.push(i);
                                              }
                                         }
                                     });
                                 }

                                 // Remove deleted
                                 for i in urls_to_remove.iter().rev() {
                                     character.urls.remove(*i);
                                 }
                             },
                             CharacterTab::Lorebooks => {
                                 ui.label("Select relevant lorebooks:");
                                 let mut go_to_lorebook = None;
                                 for lore in &app.lorebooks {
                                     ui.horizontal(|ui| {
                                         let mut is_linked = app.lore_links.contains(&lore.id);
                                         if ui.checkbox(&mut is_linked, &lore.title).clicked() {
                                             if character.id != 0 {
                                                 toggle_requests.push((character.id, lore.id, is_linked));
                                             }
                                         }
                                         if ui.small_button("➡").on_hover_text("Go to Lorebook").clicked() {
                                             go_to_lorebook = Some(lore.clone());
                                         }
                                     });
                                 }
                                 
                                 if let Some(target_lore) = go_to_lorebook {
                                     app.selected_lorebook = Some(target_lore);
                                     app.mode = crate::ui::AppMode::Lorebooks;
                                     app.central_view = crate::ui::CentralView::Editor;
                                 }
                             }
                         }
                         
 
                     });
                     
                     
                     if let Some((msg, color)) = status_update {
                         app.set_status(msg, color);
                     }
                     
                     // Handle events
                     if trigger_import {
                         app.show_import_modal = true;
                         app.import_text.clear();
                         app.parsed_data = None;
                     }
                     
                     // Execute deferred tag operations
                     if let Some((cid, name, is_ext)) = tag_add_request {
                         app.add_tag(cid, name, is_ext);
                     }
                     if let Some((cid, tid, is_ext)) = tag_remove_request {
                         app.remove_tag(cid, tid, is_ext);
                     }
                     
                     // Restore ownership
                     app.selected_character = Some(character);

                } else {
                    ui.label("Select a character to edit.");
                }
                
                // Process Toggle Requests
                for (cid, lid, linked) in toggle_requests {
                     app.toggle_lore_link(cid, lid, linked);
                }

                if let Some(c) = save_req {
                     app.save_character(c);
                }
    
    if let Some(target) = back_req {
        app.request_collection_switch(target);
    }
}


pub fn render_lorebook_editor(app: &mut CrapApp, ui: &mut egui::Ui) {
                if let Some(mut book) = app.selected_lorebook.take() {
                    let mut save_lore_req = None;
                    let mut tag_add_request = None;
                    let mut tag_remove_request = None;
                    let mut entry_save_req = None;
                    let mut entry_delete_req = None;
                    let mut entry_add_req = false;
                    
                    let mut status_update: Option<(String, egui::Color32)> = None;
                    
                    ui.heading("Edit Lorebook");
                    ui.add_space(4.0);
                    
                    // --- Top Section: Metadata ---
                    egui::ScrollArea::vertical()
                        .max_height(ui.available_height() * 0.45)
                        .id_source("lorebook_metadata_scroll")
                        .show(ui, |ui| {
                            ui.columns(2, |columns| {
                                // Left Column: Basic Data
                                columns[0].vertical(|ui| {
                                    ui.label("Title");
                                    ui.text_edit_singleline(&mut book.title);
                                    ui.add_space(8.0);
                                    
                                    ui.label("Description");
                                    ui.text_edit_multiline(&mut book.content);
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
                                        let response = ui.text_edit_singleline(&mut app.app_tag_input);
                                        if (ui.button("Add").clicked() || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))) && !app.app_tag_input.is_empty() {
                                            tag_add_request = Some((book.id, app.app_tag_input.clone()));
                                            app.app_tag_input.clear();
                                            response.request_focus();
                                        }
                                    });
                                    
                                    ui.add_space(16.0);
                                    if ui.button("Save Lorebook Metadata").clicked() {
                                         save_lore_req = Some(book.clone());
                                    }
                                });

                                // Right Column: Cover Image
                                columns[1].vertical(|ui| {
                                    ui.group(|ui| {
                                        ui.label("Cover Image");
                                        let mut cover_path_str = book.cover_path.clone().unwrap_or_default();
                                        if ui.text_edit_singleline(&mut cover_path_str).changed() {
                                             if cover_path_str.trim().is_empty() {
                                                 book.cover_path = None;
                                             } else {
                                                 book.cover_path = Some(cover_path_str.clone());
                                             }
                                        }
                                        if let Some(path_str) = &book.cover_path {
                                            ui.add_space(4.0);
                                             let uri = if path_str.contains("://") { path_str.clone() } else {
                                                 if let Ok(abs_path) = std::fs::canonicalize(path_str) {
                                                      format!("file://{}", abs_path.to_string_lossy())
                                                 } else { path_str.clone() }
                                             };
                                             let preview_width = ui.available_width() - 8.0;
                                             ui.add(egui::Image::new(uri).rounding(egui::Rounding::same(4.0)).fit_to_original_size(0.5).max_width(preview_width));
                                         }
                                        ui.add_space(8.0);
                                        ui.vertical(|ui| {
                                            if ui.button("Browse Image").clicked() {
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
                                            if ui.button("Paste from Clipboard").clicked() {
                                                // (Simplified for brevity, assuming existing logic works)
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
                                        });
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
                                                 ui.text_edit_singleline(&mut entry.name);
                                                 
                                                 ui.label("Keywords (comma separated)");
                                                 ui.text_edit_singleline(&mut entry.keywords);
                                                 
                                                 ui.label("Content");
                                                 egui::ScrollArea::vertical().id_source("entry_content_scroll").show(ui, |ui| {
                                                       ui.add(egui::TextEdit::multiline(&mut entry.content).desired_width(f32::INFINITY));
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
                                         egui::ScrollArea::vertical().id_source("entries_list_scroll").show(ui, |ui| {
                                             ui.set_width(ui.available_width());
                                             for entry in &book.entries {
                                                 let selected = app.selected_entry.as_ref().map(|e| e.id) == Some(entry.id);
                                                 if ui.selectable_label(selected, &entry.name).clicked() {
                                                     app.selected_entry = Some(entry.clone());
                                                 }
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
                                 egui::ScrollArea::vertical().id_source("lore_chars_scroll").show(ui, |ui| {
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
                    if let Some(eid) = entry_delete_req {
                        app.delete_lorebook_entry(eid, book.id);
                        app.selected_entry = None; // Deselect after delete
                    }
                    
                    // Restore ownership
                    app.selected_lorebook = Some(book);

                } else {
                    ui.label("Select a lorebook.");
                }
            }
