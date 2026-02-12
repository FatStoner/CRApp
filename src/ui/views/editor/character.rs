use crate::models::{count_tokens, Character};
use crate::ui::{CharacterTab, CrapApp};
use eframe::egui;

pub fn render_editor_view(app: &mut CrapApp, ui: &mut egui::Ui) {
    let mut trigger_import = false;

    let mut save_req = None;
    let mut toggle_requests = Vec::<(i64, i64, bool)>::new();
    let mut status_update = None;
    let mut back_history_req = false;
    let mut back_req = None;

    // Check Dirty State (before taking ownership, or logic moved down? We can do it on the owned copy too)
    // Actually, we can check dirty state on the owned copy vs the DB cache.
    // Ideally we assume 'app' state is consistent.

    // Prepare collection options - Need value BEFORE taking character if we want to use 'app' easily,
    // though if we take character, 'app' is free to be used for this too.
    let collection_options: Vec<(i64, String)> = app
        .collections
        .iter()
        .map(|c| (c.id, app.get_collection_path(c.id)))
        .collect();

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


        // Render toolbar and handle its actions
        let toolbar_action = super::toolbar::render_toolbar(ui, app, &character, is_dirty, &mut trigger_import);
        
        if toolbar_action.template_requested {
            app.popup_state = crate::ui::PopupState::TemplateSelector;
        }
        if toolbar_action.save_requested {
            save_req = Some(character.clone());
        }
        if toolbar_action.back_history_requested {
            back_history_req = true;
        }
        if let Some(target) = toolbar_action.back_to_collection {
            back_req = Some(target);
        }


        ui.horizontal(|ui| {
            ui.label("Collection:");
            let current_col_name = character
                .collection_id
                .and_then(|id| {
                    collection_options
                        .iter()
                        .find(|(cid, _)| *cid == id)
                        .map(|(_, name)| name.clone())
                })
                .unwrap_or_else(|| "Uncategorized".to_string());

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
                egui::Button::new(
                    egui::RichText::new("\u{2764} Favorite").color(egui::Color32::WHITE),
                )
                .fill(egui::Color32::from_rgb(200, 50, 50))
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
            let response = ui.add(
                egui::TextEdit::singleline(&mut app.editor_search_query)
                    .id_salt("editor_search_field")
                    .hint_text("Type 3+ chars to highlight...")
                    .desired_width(200.0),
            );

            if app.focus_search_field {
                response.request_focus();
                app.focus_search_field = false;
            }

            if !app.editor_search_query.is_empty() {
                if ui.small_button("✖").clicked() {
                    app.editor_search_query.clear();
                }

                ui.label(
                    egui::RichText::new(format!("Highlighting: '{}'", app.editor_search_query))
                        .size(11.0)
                        .color(egui::Color32::GRAY),
                );
            }
        });
        ui.separator();

        // Tabs
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut app.active_char_tab,
                CharacterTab::MainData,
                "Main Data",
            );
            ui.selectable_value(&mut app.active_char_tab, CharacterTab::Notes, "Notes");
            ui.selectable_value(
                &mut app.active_char_tab,
                CharacterTab::Lorebooks,
                "Lorebooks",
            );
            ui.selectable_value(&mut app.active_char_tab, CharacterTab::Gallery, "Gallery");
        });
        ui.separator();

        // Handle Drag and Drop for Avatar
        let dropped_files = ui.input(|i| i.raw.dropped_files.clone());
        if !dropped_files.is_empty() {
            for dropped in dropped_files {
                if let Some(path) = dropped.path {
                    let ext = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    if ["png", "jpg", "jpeg", "webp"].contains(&ext.as_str()) {
                        let dest_dir = std::path::Path::new("data/avatars");
                        let _ = std::fs::create_dir_all(dest_dir);
                        if let Some(name) = path.file_name() {
                            let dest = dest_dir.join(name);
                            if let Ok(_) = std::fs::copy(&path, &dest) {
                                character.avatar_path = Some(dest.to_string_lossy().to_string());
                                status_update = Some((
                                    "Avatar loaded from dropped file!".to_string(),
                                    egui::Color32::GREEN,
                                ));
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
                                          {
                                               let mut layouter = crate::ui::spell_layout::create_spell_check_layouter(None, app.editor_search_query.clone());
                                               let response = ui.add(egui::TextEdit::singleline(&mut character.name).layouter(&mut *layouter));
                                               crate::ui::widgets::track_text_selection(ui, &response);
                                               response.context_menu(|ui| {
                                                   crate::ui::widgets::text_context_menu(ui, &mut character.name, response.id);
                                               });
                                          }


                                         ui.label("Character Name");
                                         // Character Name (character.char_name) with search highlight
                                          {
                                               let mut layouter = crate::ui::spell_layout::create_spell_check_layouter(None, app.editor_search_query.clone());
                                               let response = ui.add(egui::TextEdit::singleline(&mut character.char_name).layouter(&mut *layouter));
                                               crate::ui::widgets::track_text_selection(ui, &response);
                                               response.context_menu(|ui| {
                                                   crate::ui::widgets::text_context_menu(ui, &mut character.char_name, response.id);
                                               });
                                          }

                                          let id = ui.make_persistent_id("title_header");
                                          egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
                                              .show_header(ui, |ui| {
                                                  ui.label("Title / Description");
                                                  ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                      if ui.small_button("Copy").clicked() {
                                                          ui.output_mut(|o| o.copied_text = character.char_title.clone());
                                                          status_update = Some(("Copied Title to clipboard".to_string(), egui::Color32::GREEN));
                                                      }
                                                      if ui.toggle_value(&mut app.count_title_in_total, "count in total").changed() {
                                                          app.token_cache.clear();
                                                      }
                                                      let mut ignore = character.spell_check_overrides.contains("title");
                                                      if ui.checkbox(&mut ignore, "Ignore Spell Check").changed() {
                                                          if ignore {
                                                              character.spell_check_overrides.insert("title".to_string());
                                                          } else {
                                                              character.spell_check_overrides.remove("title");
                                                          }
                                                      }
                                                      ui.label(egui::RichText::new(format!("Tokens: {} | Chars: {}", count_tokens(&character.char_title), character.char_title.chars().count())).size(12.0).color(egui::Color32::GRAY));
                                                  });
                                              })
                                              .body(|ui| {
                                                 // Title (character.char_title) with search highlight AND auto-resize
                                                 // Changed to multiline with min_rows(1) for auto-resize behavior
                                                 let title_edit = egui::TextEdit::multiline(&mut character.char_title)
                                                     .desired_width(f32::INFINITY)
                                                     .desired_rows(1);
                                                  {
                                                      let mut layouter = crate::ui::spell_layout::create_spell_check_layouter(
                                                          if app.enable_spell_check && !character.spell_check_overrides.contains("title") { app.spell_checker.clone() } else { None },
                                                          app.editor_search_query.clone());
                                                      let response = ui.add(title_edit.layouter(&mut *layouter));
                                                      crate::ui::widgets::track_text_selection(ui, &response);
                                                      response.context_menu(|ui| {
                                                           crate::ui::widgets::text_context_menu(ui, &mut character.char_title, response.id);
                                                       });
                                                  }
                                              });


                                         ui.add_space(8.0);
                                         
                                         ui.add_space(8.0);
                                         let id = ui.make_persistent_id("first_message_header");
                                         egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
                                             .show_header(ui, |ui| {
                                                 ui.label("First Message");
                                                 ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                     let mut ignore = character.spell_check_overrides.contains("first_message");
                                                     if ui.checkbox(&mut ignore, "Ignore Spell Check").changed() {
                                                         if ignore {
                                                             character.spell_check_overrides.insert("first_message".to_string());
                                                         } else {
                                                             character.spell_check_overrides.remove("first_message");
                                                         }
                                                     }
                                                     if ui.small_button("Copy").clicked() {
                                                         ui.output_mut(|o| o.copied_text = character.first_message.clone());
                                                         status_update = Some(("Copied First Message to clipboard".to_string(), egui::Color32::GREEN));
                                                     }
                                                     ui.label(egui::RichText::new(format!("Tokens: {} | Chars: {}", count_tokens(&character.first_message), character.first_message.chars().count())).size(12.0).color(egui::Color32::GRAY));
                                                 });
                                             })
                                             .body(|ui| {
                                                 let text_edit = egui::TextEdit::multiline(&mut character.first_message).desired_width(f32::INFINITY);
                                                  {
                                                      let mut layouter = crate::ui::spell_layout::create_spell_check_layouter(
                                                          if app.enable_spell_check && !character.spell_check_overrides.contains("first_message") { app.spell_checker.clone() } else { None },
                                                          app.editor_search_query.clone());
                                                      let response = ui.add(text_edit.layouter(&mut *layouter));
                                                      crate::ui::widgets::track_text_selection(ui, &response);
                                                      response.context_menu(|ui| {
                                                          crate::ui::widgets::text_context_menu(ui, &mut character.first_message, response.id);
                                                      });
                                                  }
                                             });
                                         
                                         ui.add_space(8.0);
                                         ui.add_space(8.0);
                                         let id = ui.make_persistent_id("personality_header");
                                         egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
                                             .show_header(ui, |ui| {
                                                 ui.label("Personality");
                                                 ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                     let mut ignore = character.spell_check_overrides.contains("personality");
                                                     if ui.checkbox(&mut ignore, "Ignore Spell Check").changed() {
                                                         if ignore {
                                                             character.spell_check_overrides.insert("personality".to_string());
                                                         } else {
                                                             character.spell_check_overrides.remove("personality");
                                                         }
                                                     }
                                                     if ui.small_button("Copy").clicked() {
                                                         ui.output_mut(|o| o.copied_text = character.personality.clone());
                                                         status_update = Some(("Copied Personality to clipboard".to_string(), egui::Color32::GREEN));
                                                     }
                                                     ui.label(egui::RichText::new(format!("Tokens: {} | Chars: {}", count_tokens(&character.personality), character.personality.chars().count())).size(12.0).color(egui::Color32::GRAY));
                                                 });
                                             })
                                             .body(|ui| {
                                                 let text_edit = egui::TextEdit::multiline(&mut character.personality).desired_width(f32::INFINITY);
                                                  {
                                                      let mut layouter = crate::ui::spell_layout::create_spell_check_layouter(
                                                          if app.enable_spell_check && !character.spell_check_overrides.contains("personality") { app.spell_checker.clone() } else { None },
                                                          app.editor_search_query.clone());
                                                      let response = ui.add(text_edit.layouter(&mut *layouter));
                                                      crate::ui::widgets::track_text_selection(ui, &response);
                                                      response.context_menu(|ui| {
                                                          crate::ui::widgets::text_context_menu(ui, &mut character.personality, response.id);
                                                      });
                                                  }
                                             });
        
                                         ui.add_space(8.0);
                                         let id = ui.make_persistent_id("scenario_header");
                                         egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
                                             .show_header(ui, |ui| {
                                                 ui.label("Scenario");
                                                 ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                     let mut ignore = character.spell_check_overrides.contains("scenario");
                                                     if ui.checkbox(&mut ignore, "Ignore Spell Check").changed() {
                                                         if ignore {
                                                             character.spell_check_overrides.insert("scenario".to_string());
                                                         } else {
                                                             character.spell_check_overrides.remove("scenario");
                                                         }
                                                     }
                                                     if ui.small_button("Copy").clicked() {
                                                         ui.output_mut(|o| o.copied_text = character.scenario.clone());
                                                         status_update = Some(("Copied Scenario to clipboard".to_string(), egui::Color32::GREEN));
                                                     }
                                                     ui.label(egui::RichText::new(format!("Tokens: {} | Chars: {}", count_tokens(&character.scenario), character.scenario.chars().count())).size(12.0).color(egui::Color32::GRAY));
                                                 });
                                             })
                                             .body(|ui| {
                                                 let text_edit = egui::TextEdit::multiline(&mut character.scenario).desired_width(f32::INFINITY);
                                                  {
                                                      let mut layouter = crate::ui::spell_layout::create_spell_check_layouter(
                                                          if app.enable_spell_check && !character.spell_check_overrides.contains("scenario") { app.spell_checker.clone() } else { None },
                                                          app.editor_search_query.clone());
                                                      let response = ui.add(text_edit.layouter(&mut *layouter));
                                                      crate::ui::widgets::track_text_selection(ui, &response);
                                                      response.context_menu(|ui| {
                                                          crate::ui::widgets::text_context_menu(ui, &mut character.scenario, response.id);
                                                      });
                                                  }
                                             });
        
                                         ui.add_space(8.0);
                                         let id = ui.make_persistent_id("example_dialogue_header");
                                         egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
                                             .show_header(ui, |ui| {
                                                 ui.label("Example Dialogue");
                                                 ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                     let mut ignore = character.spell_check_overrides.contains("example");
                                                     if ui.checkbox(&mut ignore, "Ignore Spell Check").changed() {
                                                         if ignore {
                                                             character.spell_check_overrides.insert("example".to_string());
                                                         } else {
                                                             character.spell_check_overrides.remove("example");
                                                         }
                                                     }
                                                     if ui.small_button("Copy").clicked() {
                                                         ui.output_mut(|o| o.copied_text = character.example_dialogue.clone());
                                                         status_update = Some(("Copied Example Dialogue to clipboard".to_string(), egui::Color32::GREEN));
                                                     }
                                                     ui.label(egui::RichText::new(format!("Tokens: {} | Chars: {}", count_tokens(&character.example_dialogue), character.example_dialogue.chars().count())).size(12.0).color(egui::Color32::GRAY));
                                                 });
                                             })
                                             .body(|ui| {
                                                 let text_edit = egui::TextEdit::multiline(&mut character.example_dialogue).desired_width(f32::INFINITY);
                                                  {
                                                      let mut layouter = crate::ui::spell_layout::create_spell_check_layouter(
                                                          if app.enable_spell_check && !character.spell_check_overrides.contains("example") { app.spell_checker.clone() } else { None },
                                                          app.editor_search_query.clone());
                                                      let response = ui.add(text_edit.layouter(&mut *layouter));
                                                      crate::ui::widgets::track_text_selection(ui, &response);
                                                      response.context_menu(|ui| {
                                                          crate::ui::widgets::text_context_menu(ui, &mut character.example_dialogue, response.id);
                                                      });
                                                  }
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
                                             let uri = crate::ui::utils::get_image_uri(path_str);
                                             
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
                                                      if let Some(avatar_path) = app.update_avatar_from_file(path) {
                                                          character.avatar_path = Some(avatar_path);
                                                      }
                                                  }
                                              }
                                                                                          if ui.button("Paste from Clipboard").clicked() {
                                                  match app.paste_avatar_from_clipboard() {
                                                      Ok(avatar_path) => {
                                                          character.avatar_path = Some(avatar_path);
                                                          status_update = Some(("Avatar pasted successfully!".to_string(), egui::Color32::GREEN));
                                                      }
                                                      Err(e) => {
                                                          status_update = Some((e, egui::Color32::RED));
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
                                         let t_title = if app.count_title_in_total { count_tokens(&character.char_title) } else { 0 };
                                         
                                         let total_tokens = t_first + t_pers + t_scen + t_ex + t_title;
                                         let perm_tokens = t_pers + t_scen;
                                         
                                         ui.label(egui::RichText::new(format!("Total Tokens: {} (Permanent: {})", total_tokens, perm_tokens))
                                             .strong()
                                             .color(egui::Color32::WHITE));
                                         
                                         let c_first = character.first_message.chars().count();
                                         let c_pers = character.personality.chars().count();
                                         let c_scen = character.scenario.chars().count();
                                         let c_ex = character.example_dialogue.chars().count();
                                         let c_title = if app.count_title_in_total { character.char_title.chars().count() } else { 0 };
                                         
                                         let total_chars = c_first + c_pers + c_scen + c_ex + c_title;
                                         let perm_chars = c_pers + c_scen;

                                         ui.label(egui::RichText::new(format!("Total Chars: {} (Permanent: {})", total_chars, perm_chars))
                                             .strong()
                                             .color(egui::Color32::WHITE));
                                     });
                                 });
                             },
                             CharacterTab::Notes => {
                                 ui.label("Notes");
                                 let width = ui.ctx().screen_rect().width() * 2.0 / 3.0;
                                 let text_edit = egui::TextEdit::multiline(&mut character.author_notes).desired_width(width);
                                  {
                                      let mut layouter = crate::ui::spell_layout::create_spell_check_layouter(
                                          if app.enable_spell_check && !character.spell_check_overrides.contains("notes") { app.spell_checker.clone() } else { None },
                                          app.editor_search_query.clone());
                                      let response = ui.add(text_edit.layouter(&mut *layouter));
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
                                     // Temporarily restore character ownership so push_history sees it
                                     app.selected_character = Some(character.clone());
                                     app.load_lorebook(target_lore.id);
                                     // Navigation complete, mode switched.
                                 }
                             },
                            CharacterTab::Gallery => {
                                ui.heading("Character Gallery");
                                ui.label(egui::RichText::new("Images associated with this character.").size(11.0).color(egui::Color32::GRAY));
                                ui.add_space(8.0);

                                let gallery_dir = format!("data/gallery/{}", character.id);
                                let _ = std::fs::create_dir_all(&gallery_dir);
                                let mut refresh_gallery = false;

                                // Add Image Button
                                ui.horizontal(|ui| {
                                    if ui.button("➕ Add Image").clicked() {
                                        app.add_gallery_image_async(character.id);
                                    }
                                     if ui.button("📋 Paste").clicked() {
                                        match app.paste_gallery_image_from_clipboard(character.id) {
                                            Ok(_) => {
                                                status_update = Some(("Image pasted to gallery!".to_string(), egui::Color32::GREEN));
                                                ui.ctx().request_repaint();
                                            }
                                            Err(e) => {
                                                status_update = Some((e, egui::Color32::RED));
                                            }
                                        }
                                    }
                                    if ui.button("🔄 Refresh").clicked() {
                                        // Just triggers repaint naturally
                                    }
                                    if ui.button("📂 Open Folder").clicked() {
                                         #[cfg(target_os = "linux")]
                                         {
                                             if let Ok(abs_path) = std::fs::canonicalize(&gallery_dir) {
                                                  let _ = std::process::Command::new("xdg-open").arg(abs_path).spawn();
                                             }
                                         }
                                    }
                                });
                                ui.add_space(8.0);
                                
                                let mut files = Vec::new();
                                if let Ok(entries) = std::fs::read_dir(&gallery_dir) {
                                    for entry in entries.flatten() {
                                        if let Ok(file_type) = entry.file_type() {
                                            if file_type.is_file() {
                                                let path = entry.path();
                                                if let Some(ext) = path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase()) {
                                                    if ["png", "jpg", "jpeg", "webp"].contains(&ext.as_str()) {
                                                        files.push(path);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                files.sort();

                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    ui.horizontal_wrapped(|ui| {
                                         for path in &files {
                                             let path_str = path.to_string_lossy().to_string();
                                             // Use get_image_uri to handle caching and protocol
                                             let uri = crate::ui::utils::get_image_uri(&path_str);
                                             
                                             let size = 150.0;
                                             let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());
                                             
                                             if ui.is_rect_visible(rect) {
                                                  crate::ui::widgets::paint_gallery_image(ui, rect, &uri, 4.0);
                                             }
                                             
                                             // Hover
                                             if response.hovered() {
                                                 ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(2.0, egui::Color32::WHITE));
                                             }

                                             if response.clicked() {
                                                 app.fullscreen_image = Some(uri.clone());
                                                 app.gallery_context = Some(
                                                     files
                                                         .iter()
                                                         .map(|p| crate::ui::utils::get_image_uri(&p.to_string_lossy()))
                                                         .collect(),
                                                 );
                                             }

                                             response.context_menu(|ui| {
                                                 if ui.button("🗑 Delete").clicked() {
                                                     let _ = std::fs::remove_file(path);
                                                     refresh_gallery = true;
                                                     ui.close_menu();
                                                 }
                                             });
                                         }
                                    });
                                });
                                
                                if refresh_gallery {
                                    ui.ctx().request_repaint();
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
        if app.central_view == crate::ui::CentralView::Editor && app.selected_character.is_none() {
            app.selected_character = Some(character);
        }
        if back_history_req {
            app.request_back();
        }
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
