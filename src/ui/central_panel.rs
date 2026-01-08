use eframe::egui;
use crate::models::Tag;
use crate::ui::{CrapApp, AppMode, CharacterTab, UiEvent};
use crate::card_v2::CharacterCardV2;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

// ----------------------------------------------------------------------------
// Parsing Logic (Moved from ui.rs)
// ----------------------------------------------------------------------------

#[derive(Default, Clone)]
pub struct ParsedCharacterData {
    pub name: String,
    pub title: String,
    pub personality: String,
    pub scenario: String,
    pub first_message: String,
    pub example_dialogue: String,
    pub external_tags: Vec<String>,
}

pub fn parse_clipboard(text: &str) -> ParsedCharacterData {
    let mut data = ParsedCharacterData::default();
    
    // Split into lines
    let mut current_section = "";
    let mut potential_block: Vec<String> = Vec::new();
    let mut tags_processed = false;
    
    let strict_blacklist = [
        "Spicychat", "Home", "Chats", "My Personas", "Create", "Chatbot", "Lorebook", "New", "Group", "My Creations", "Chatbots", "Lorebooks", "Groups", "Favorites", "Recommendations", "Leaderboard", "Blocked Creators", "Subscribe", "Help", "Sign Out", "Terms", "Privacy", "Refunds", "Reporting", "Guidelines", "Support", "Affiliates", "Download SpicychatAI on the PlayStore download", "Get Premium", "Free", "Back", "avatar image", "Chat Now",
        "SHOW LESS", "Owned & operated by:", "NextDay AI Incorporated", "NextDay AI USA Inc", "NextDay AI EU Ltd", "Resources", "Terms & Conditions", "Privacy Policy", "Refund Policy", "Community", "Community Guidelines", "Become an Affiliate", "Report Content", "Join Us", "Discord", "Twitter", "Reddit", "18 U.S.C. 2257 Record-Keeping Requirements Compliance Statement", "Edit"
    ];
    
    let soft_blacklist = ["Favorite", "Share"];

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        
        // Skip garbage
        if current_section.is_empty() && (trimmed.len() < 3 || trimmed.chars().all(|c| !c.is_alphabetic())) {
            continue;
        }
        
        // 1. Strict Blacklist Check
        let lower = trimmed.to_lowercase();
        let mut is_blacklisted = strict_blacklist.iter().any(|b| lower.contains(&b.to_lowercase()));
        
        // 2. Soft Blacklist Check
        if !is_blacklisted {
            if soft_blacklist.iter().any(|b| lower == b.to_lowercase()) {
                is_blacklisted = true;
            }
        }
        
        let is_tag_trigger = lower.contains("suggest tag") || lower == "nsfw" || lower == "sfw" || lower == "tags";
        if is_tag_trigger { is_blacklisted = true; }

        if is_blacklisted {
             if is_tag_trigger && !tags_processed {
                 // Capture Title/Tags from potential block
                 if !potential_block.is_empty() {
                     // Assume first line of potential block is Title (if reasonable length)
                     // The rest are tags.
                     // Heuristic: Last line might be title if block is huge? No, usually Title is separate line above tags.
                     
                     if let Some(first) = potential_block.first() {
                         if data.name.is_empty() {
                             // If name wasn't found (rare), maybe this title is name? 
                             // But usually name is specifically caught.
                             // Let's assume title.
                         }
                         data.title = first.clone();
                     }
                     
                     // Any subsequent lines in potential block, PLUS the lines after "Suggest Tag" found later, are tags.
                     // But wait, "Suggest Tag" usually APPEARS BEFORE tags list in some views, or AFTER?
                     // In Spicychat: Name -> Desc -> Suggest Tag -> Tags
                     // Our block collected stuff BEFORE "Suggest Tag".
                     // So actually, if we hit "Suggest Tag", the block *was* the description/title/tags area.
                     // Let's trust the "potential_block" as tags if they are short.
                     
                     for item in potential_block.iter().skip(1) { // Skip title
                         if item.len() < 50 {
                             data.external_tags.push(item.clone());
                         }
                     }
                     tags_processed = true;
                 }
                 potential_block.clear();
             }
             
             // Clear potential block on ANY blacklist hit to prevent garbage accumulation
             // UNLESS it was the tag trigger we just handled.
             if !is_tag_trigger {
                potential_block.clear();
             }
             continue; 
        }

        // Section Detection
        if lower.starts_with("personality") { current_section = "personality"; continue; }
        if lower.starts_with("scenario") { current_section = "scenario"; continue; }
        if lower.starts_with("first message") || lower.starts_with("greeting") { current_section = "first_message"; continue; }
        if lower.starts_with("example dialogue") { current_section = "personality"; continue; } // Map to personality/dialogue
        
        // Tag Line Detection (Comma separated)
        if current_section.is_empty() && trimmed.contains(',') && trimmed.split(',').count() > 3 {
             // Likely a tag line if we haven't processed tags yet
             if !tags_processed {
                 let tags: Vec<String> = trimmed.split(',').map(|s| s.trim().to_string()).collect();
                 data.external_tags.extend(tags);
                 tags_processed = true;
             }
             continue; // Don't add to potential block
        }
        
        // Tag Accumulation (if we hit a block of short words after "Suggest Tag" or similar, handled via potential_block usually, but sometimes tags are just lines)
        // If we extracted tags via "Suggest Tag" trigger, we might be done.
        
        if current_section.is_empty() {
             // Check if it's a tag-like line (short, alphabetic)
             // Spicychat tags are often just words on new lines.
             if tags_processed && trimmed.len() < 30 {
                  // Assume continuation of tags?
                  data.external_tags.push(trimmed.to_string());
                  continue;
             }
             
             // Heuristic: If we are collecting "potential block" (pre-header), treat as candidates
             if !tags_processed {
                 // FIX: Increased limit to 250
                  if trimmed.len() < 250 {
                     potential_block.push(trimmed.to_string());
                  } else {
                     data.personality.push_str(trimmed);
                     data.personality.push('\n');
                  }
             } else {
                 // Fallback
                 data.personality.push_str(trimmed);
                 data.personality.push('\n');
             }
             continue;
        }
        
        match current_section {
            "personality" => { data.personality.push_str(trimmed); data.personality.push('\n'); },
            "scenario" => { data.scenario.push_str(trimmed); data.scenario.push('\n'); },
            "first_message" => { data.first_message.push_str(trimmed); data.first_message.push('\n'); },
            _ => {
                 if data.name.is_empty() && trimmed.len() < 50 && !trimmed.contains(':') && !lower.starts_with("description") && !lower.starts_with("@") {
                     data.name = trimmed.to_string();
                 }
            }
        }
    }
    
    data.personality = data.personality.trim().to_string();
    data.scenario = data.scenario.trim().to_string();
    data.first_message = data.first_message.trim().to_string();
    
    data
}

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
                        ui.label("Paste raw character text below (Spicy/Janitor format):");
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
                            });
                        });
                        
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if ui.button("Apply to Character").clicked() {
                                if let Some(c) = &mut app.selected_character {
                                    let d = app.parsed_data.take().unwrap();
                                    
                                    if !d.name.is_empty() { c.name = d.name.clone(); }
                                    if !d.title.is_empty() { c.char_title = d.title.clone(); }
                                    if !d.personality.is_empty() { c.personality = d.personality.clone(); }
                                    if !d.scenario.is_empty() { c.scenario = d.scenario; }
                                    if !d.first_message.is_empty() { c.first_message = d.first_message; }
                                    if !d.example_dialogue.is_empty() { c.example_dialogue = d.example_dialogue; }
                                    
                                    if c.id != 0 {
                                        let tx_clone = app.tx.clone();
                                        let db_clone = app.db.clone();
                                        let cid = c.id;
                                        let tags = d.external_tags.clone();
                                        
                                        tokio::spawn(async move {
                                            for tag_name in tags {
                                                let _ = db_clone.add_tag_to_character(cid, &tag_name, true).await;
                                            }
                                            let _ = tx_clone.send(UiEvent::TagOperationFinished(Ok(()))).await;
                                        });
                                        app.set_status("Data updated. Tags being added.".to_string(), egui::Color32::GREEN);
                                    } else {
                                        // New Character - Tags are added to the list but not saved to DB yet (will be on Save)
                                        // Wait, the "d.external_tags" strings need to be converted to Tag structs.
                                        // The original code loop at line 273 was doing this.
                                        for tag_name in d.external_tags {
                                            c.external_tags.push(Tag { id: 0, name: tag_name });
                                        }
                                        app.set_status("Import applied to New Character (Unsaved).".to_string(), egui::Color32::YELLOW);
                                    }
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
                let mut trigger_import = false;
                let mut save_req = None;
                let mut toggle_requests = Vec::new();
                
                 // Prepare collection options to avoid borrow checker issues inside the closure where we mutate char.
                 let collection_options: Vec<(i64, String)> = app.collections.iter().map(|c| {
                    (c.id, app.get_collection_path(c.id))
                 }).collect();
                
                if let Some(character) = &mut app.selected_character {
                     // Clone for closures
                     let _tx_clone = app.tx.clone();
                     let _db_clone = app.db.clone();
                     
                     ui.horizontal(|ui| {
                        ui.heading("Edit Character");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.menu_button("EXPORT", |ui| {
                                let name_slug = character.name.replace(" ", "_");
                                
                                if ui.button("Native (.crapp)").clicked() {
                                    if let Ok(json) = serde_json::to_string_pretty(&character) {
                                        let task_name = format!("{}.crapp", name_slug);
                                        let task_json = json.clone();
                                        tokio::spawn(async move {
                                            if let Some(path) = rfd::FileDialog::new().set_file_name(task_name).save_file() {
                                                let _ = std::fs::write(path, task_json);
                                            }
                                        });
                                    }
                                    ui.close_menu();
                                }
                                
                                if ui.button("Platform V2 (.json)").clicked() {
                                    let v2 = CharacterCardV2::new(
                                        character.char_name.clone(),
                                        character.char_title.clone(),
                                        character.personality.clone(),
                                        character.scenario.clone(),
                                        character.first_message.clone(),
                                        character.example_dialogue.clone(),
                                    );
                                    if let Ok(json) = serde_json::to_string_pretty(&v2) {
                                        let task_name = format!("{}.json", name_slug);
                                        tokio::spawn(async move {
                                            if let Some(path) = rfd::FileDialog::new().set_file_name(task_name).save_file() {
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
                                        if let Some(path) = rfd::FileDialog::new().set_file_name(task_name).save_file() {
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
                                            character.char_title.clone(),
                                            character.personality.clone(),
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
                                                    if let Some(save_path) = rfd::FileDialog::new().set_file_name(task_name).save_file() {
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
                        });
                    });
                     
                    ui.horizontal(|ui| {
                        ui.label("Collection:");
                        let current_col_name = character.collection_id.and_then(|id| {
                            collection_options.iter().find(|(cid, _)| *cid == id).map(|(_, name)| name.clone())
                        }).unwrap_or_else(|| "Uncategorized".to_string());
                        
                        egui::ComboBox::from_id_source("collection_combo")
                            .selected_text(current_col_name)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut character.collection_id, None, "Uncategorized");
                                for (id, name) in &collection_options {
                                    ui.selectable_value(&mut character.collection_id, Some(*id), name);
                                }
                            });
                    });
                    ui.separator();

                     // Tabs
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut app.active_char_tab, CharacterTab::MainData, "Main Data");
                        ui.selectable_value(&mut app.active_char_tab, CharacterTab::AuthorNotes, "Author Notes");
                        ui.selectable_value(&mut app.active_char_tab, CharacterTab::AssociatedLore, "Associated Lore");
                    });
                    ui.separator();
                    
                    let mut tag_add_request: Option<(i64, String, bool)> = None;
                    let mut tag_remove_request: Option<(i64, i64, bool)> = None;
                    
                     egui::ScrollArea::vertical().show(ui, |ui| {
                         match app.active_char_tab {
                             CharacterTab::MainData => {
                                 ui.label("Name (File Name)");
                                 ui.text_edit_singleline(&mut character.name);
                                 ui.label("Character Name");
                                 ui.text_edit_singleline(&mut character.char_name);
                                 ui.label("Title");
                                 ui.text_edit_singleline(&mut character.char_title);
                                 ui.add_space(8.0);
                                 
                                 egui::CollapsingHeader::new("Tags & Metadata")
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        ui.vertical(|ui| {
                                             // App Tags
                                            ui.label(egui::RichText::new("App Tags").strong().color(egui::Color32::from_rgb(100, 150, 255)));
                                            ui.horizontal(|ui| {
                                                for tag in &character.app_tags {
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
                                                ui.text_edit_singleline(&mut app.app_tag_input);
                                                if ui.button("Add").clicked() && !app.app_tag_input.is_empty() {
                                                    tag_add_request = Some((character.id, app.app_tag_input.clone(), false));
                                                    app.app_tag_input.clear();
                                                }
                                            });
                                            
                                            ui.add_space(8.0);
                                            
                                            // External Tags
                                            ui.label(egui::RichText::new("External Tags").strong().color(egui::Color32::GRAY));
                                            ui.horizontal(|ui| {
                                                for tag in &character.external_tags {
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
                                                ui.text_edit_singleline(&mut app.ext_tag_input);
                                                if ui.button("Add").clicked() && !app.ext_tag_input.is_empty() {
                                                     tag_add_request = Some((character.id, app.ext_tag_input.clone(), true));
                                                     app.ext_tag_input.clear();
                                                }
                                            });
                                        });
                                    });
                                 
                                 ui.add_space(8.0);
                                 ui.label("Personality");
                                 ui.text_edit_multiline(&mut character.personality);
                                 ui.label("Scenario");
                                 ui.text_edit_multiline(&mut character.scenario);
                                 ui.label("Example Dialogue");
                                 ui.text_edit_multiline(&mut character.example_dialogue);
                                 ui.label("First Message");
                                 ui.text_edit_multiline(&mut character.first_message);
                                 
                                 ui.label("Avatar");
                                 if let Some(path_str) = &character.avatar_path {
                                     // ... existing image code ...
                                     ui.label(path_str);
                                 }
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
                             },
                             CharacterTab::AuthorNotes => {
                                 ui.label("Author Notes");
                                 ui.text_edit_multiline(&mut character.author_notes);
                             },
                             CharacterTab::AssociatedLore => {
                                 ui.label("Select relevant lorebooks:");
                                 for lore in &app.lorebooks {
                                     let mut is_linked = app.lore_links.contains(&lore.id);
                                     if ui.checkbox(&mut is_linked, &lore.title).clicked() {
                                         if character.id != 0 {
                                             toggle_requests.push((character.id, lore.id, is_linked));
                                         }
                                     }
                                 }
                             }
                         }
                         
                         ui.add_space(20.0);
                         ui.horizontal(|ui| {
                             if app.is_saving {
                                 ui.spinner();
                                 ui.label("Saving...");
                             } else {
                                 if ui.button("Save Character").clicked() {
                                     save_req = Some(character.clone());
                                 }
                                 if let Some((msg, color)) = &app.status_message {
                                     ui.colored_label(*color, msg);
                                 }
                             }
                         });
                     });
                     
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
                } else {
                    ui.label("Select a character to edit.");
                }
                
                if let Some(c) = save_req {
                    app.save_character(c);
                }
            },
            AppMode::Lorebooks => {
                if let Some(book) = &mut app.selected_lorebook {
                    let mut save_lore_req = None;
                    
                    ui.heading("Edit Lorebook");
                    ui.text_edit_singleline(&mut book.title);
                    ui.add_space(4.0);
                    ui.label("Description / Content");
                    ui.text_edit_multiline(&mut book.description);
                    
                    if ui.button("Save Lorebook").clicked() {
                         save_lore_req = Some(book.clone());
                    }
                    
                    if let Some(l) = save_lore_req {
                        app.save_lorebook(l);
                    }
                } else {
                    ui.label("Select a lorebook.");
                }
            },
            _ => {
                ui.label("Unknown mode");
            }
        }
    });
}
