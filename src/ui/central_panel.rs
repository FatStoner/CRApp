use eframe::egui;
use crate::models::{Tag, count_tokens};
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
    
    // Pre-processing
    let lines: Vec<&str> = text.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    if lines.is_empty() { return data; }

    // -------------------------------------------------------------------------
    // 1. ANCHORS & METADATA
    // -------------------------------------------------------------------------
    let idx_back = lines.iter().position(|&l| l == "Back");
    let idx_share = lines.iter().position(|&l| l == "Share").or_else(|| lines.iter().position(|&l| l == "Favorite"));
    let idx_suggest_tag = lines.iter().position(|&l| l.eq_ignore_ascii_case("suggest tag"));

    // Name extraction (Back -> avatar image -> Name)
    if let Some(idx) = idx_back {
        if let Some(offset) = lines.iter().skip(idx).take(3).position(|&l| l == "avatar image") {
            let name_idx = idx + offset + 1;
            if name_idx < lines.len() {
                data.name = lines[name_idx].to_string();
            }
        }
    }

    // Title & Tags extraction (Share -> ... -> Suggest Tag)
    // We prioritize this block for tags to avoid reading section headers as tags or vice versa.
    let content_start_idx = if let Some(end) = idx_suggest_tag {
        if let Some(start) = idx_share {
            if end > start {
                 let range = &lines[(start + 1)..end];
                 let candidates: Vec<&str> = range.iter().filter(|&&l| {
                    let s = l.to_lowercase();
                    !s.chars().all(|c| c.is_numeric() || c == ',' || c == '.') && // pure numbers
                    !s.contains('%') &&
                    !s.contains("tokens") &&
                    !s.contains("chat now") &&
                    s != "share" &&
                    s != "favorite"
                }).cloned().collect();
                
                if !candidates.is_empty() {
                    data.title = candidates[0].to_string();
                    for tag in candidates.iter().skip(1) {
                         data.external_tags.push(tag.to_string());
                    }
                }
            }
        }
        end + 1 // Start scanning content just after "Suggest Tag"
    } else {
        0 // Fallback: if no strict metadata block, scan whole file (less safe but necessary fallback)
    };

    // -------------------------------------------------------------------------
    // 2. CONTENT SECTIONS (Strict Scan)
    // -------------------------------------------------------------------------
    let mut current_section = "";
    
    for i in content_start_idx..lines.len() {
        let line = lines[i];
        let lower = line.to_lowercase();

        // Footer Stop
        if lower == "spicychat" || lower.starts_with("owned & operated by") {
            break; 
        }

        // Headers
        if lower == "greeting" || lower == "first message" { current_section = "first_message"; continue; }
        if lower == "personality" { current_section = "personality"; continue; }
        if lower == "scenario" { current_section = "scenario"; continue; }
        if lower == "example dialogues" || lower == "example dialogue" { current_section = "example_dialogue"; continue; }
        if lower == "show less" { current_section = "ignore"; continue; }
        
        // Key-Value checks (Only inside relevant sections or if valid)
        // Note: Name might be in Personality if we missed it earlier
        if current_section == "personality" && lower.starts_with("name:") && data.name.is_empty() {
             if let Some((_, val)) = line.split_once(':') {
                 data.name = val.trim().to_string();
             }
        }

        match current_section {
            "first_message" => { data.first_message.push_str(line); data.first_message.push('\n'); },
            "personality" => { data.personality.push_str(line); data.personality.push('\n'); },
            "scenario" => { data.scenario.push_str(line); data.scenario.push('\n'); },
            "example_dialogue" => { data.example_dialogue.push_str(line); data.example_dialogue.push('\n'); },
            _ => {
                 // Fallback catch for Name if purely unstructured and we are outside sections
                 if i < 20 && data.name.is_empty() && line.len() < 50 && !line.contains(':') && !lower.starts_with('@') && !lower.contains("tokens") {
                      // Only if we haven't found a name yet and we are early in the file
                      // data.name = line.to_string(); // Too risky with strict parsing?
                 }
            }
        }
    }

    // Cleanup
    data.personality = data.personality.trim().to_string();
    data.scenario = data.scenario.trim().to_string();
    data.first_message = data.first_message.trim().to_string();
    data.example_dialogue = data.example_dialogue.trim().to_string();
    
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

enum BrowserAction {
    MoveCharacter(i64, Option<i64>),
    DeleteCharacter(i64),
    RenameCollection(i64, String),
    DeleteCollection(i64),
}

fn render_collection_move_menu(
    ui: &mut egui::Ui,
    collections: &Vec<crate::models::Collection>,
    parent_id: Option<i64>,
    target_char_id: i64,
    actions: &mut Vec<BrowserAction>,
) {
    let current_level: Vec<&crate::models::Collection> = collections
        .iter()
        .filter(|c| c.parent_id == parent_id)
        .collect();

    for col in current_level {
        ui.menu_button(&col.name, |ui| {
            if ui.button("Move Here").clicked() {
                actions.push(BrowserAction::MoveCharacter(target_char_id, Some(col.id)));
                ui.close_menu();
            }
            render_collection_move_menu(ui, collections, Some(col.id), target_char_id, actions);
        });
    }
}

fn render_browser_view(app: &mut CrapApp, ui: &mut egui::Ui) {
    let viewing_all = app.viewing_all_characters;
    let collection_id = app.selected_collection_id;
    let mut actions = Vec::new();

    // Clone collections for context menu usage
    let all_collections = app.collections.clone();

    
    let collection_name = if viewing_all {
        "All Characters (Flat View)".to_string()
    } else if let Some(id) = collection_id {
        app.collections.iter().find(|c| c.id == id).map(|c| c.name.clone()).unwrap_or("Unknown".to_string())
    } else {
        "Uncategorized (Root)".to_string()
    };
    
    let parent_id = if let Some(id) = collection_id {
        app.collections.iter().find(|c| c.id == id).and_then(|c| c.parent_id)
    } else {
        None
    };

    ui.horizontal(|ui| {
        // Back only if in a collection, not in "All" mode which is top level.
        if !viewing_all && collection_id.is_some() {
             if ui.button("⬅ Back").clicked() {
                 app.request_collection_switch(parent_id);
             }
        }
        ui.heading(format!("Browsing: {}", collection_name));

        // Rename Button (Far Right)
        if !viewing_all {
            if let Some(id) = collection_id {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✏ Rename").clicked() {
                        let current_name = app.collections.iter().find(|c| c.id == id).map(|c| c.name.clone()).unwrap_or_default();
                        app.popup_state = crate::ui::PopupState::Renaming { id, name: current_name };
                    }
                });
            }
        }
    });
    ui.add_space(10.0);
    
    let subfolders: Vec<crate::models::Collection> = if viewing_all {
        Vec::new() // "All" view is flat, no folders shown typically? Or maybe we just show all chars.
    } else {
        app.collections.iter()
            .filter(|c| c.parent_id == collection_id)
            .cloned()
            .collect()
    };

    let chars: Vec<crate::models::Character> = if viewing_all {
        app.characters.clone()
    } else {
        app.characters.iter()
            .filter(|c| c.collection_id == collection_id)
            .cloned()
            .collect()
    };

    if chars.is_empty() && subfolders.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.label(egui::RichText::new("No characters or subfolders in this collection").size(18.0).color(egui::Color32::GRAY));
            ui.add_space(10.0);
            if ui.button(egui::RichText::new("➕ Add New Character here").size(16.0)).clicked() {
                 app.selected_character = Some(crate::models::Character::default());
                 app.selected_character.as_mut().unwrap().collection_id = collection_id;
                 app.mode = AppMode::Characters;
                 app.central_view = crate::ui::CentralView::Editor;
            }
        });
        return;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            // Render Subfolders
            // Render Subfolders
            for folder in subfolders {
                let card_width = 180.0;
                let card_height = 260.0;
                
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(card_width, card_height),
                    egui::Sense::click()
                );
                
                let bg_color = if response.hovered() {
                    ui.visuals().widgets.hovered.bg_fill
                } else {
                    ui.visuals().widgets.noninteractive.bg_fill
                };
                
                ui.painter().rect_filled(rect, 8.0, bg_color);
                ui.painter().rect_stroke(rect, 1.0, ui.visuals().widgets.noninteractive.bg_stroke);
                
                if response.clicked() {
                    app.selected_collection_id = Some(folder.id);
                }
                
                response.context_menu(|ui| {
                    if ui.button("✏ Rename").clicked() {
                        actions.push(BrowserAction::RenameCollection(folder.id, folder.name.clone()));
                        ui.close_menu();
                    }
                    if ui.button("🗑 Delete").clicked() {
                        actions.push(BrowserAction::DeleteCollection(folder.id));
                        ui.close_menu();
                    }
                });
                
                // Content
                let content_rect = rect.shrink(8.0);
                
                // Avatar (Top Square - Folder Icon)
                let avatar_size = content_rect.width();
                let avatar_rect = egui::Rect::from_min_size(content_rect.min, egui::vec2(avatar_size, avatar_size));

                // Draw centered folder icon
                ui.painter().rect_filled(avatar_rect, 4.0, egui::Color32::from_rgb(60, 60, 70)); // Darker bg for folder
                ui.painter().text(
                    avatar_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "📁",
                    egui::FontId::proportional(64.0),
                    egui::Color32::from_rgb(200, 200, 220)
                );

                // Text Area (Name)
                let text_top = avatar_rect.max.y + 8.0;
                let name_font = egui::FontId::proportional(16.0);
                let name_galley = ui.painter().layout_no_wrap(folder.name.clone(), name_font.clone(), ui.visuals().text_color());
                ui.painter().galley(egui::pos2(content_rect.min.x, text_top), name_galley, ui.visuals().text_color());
            }
            
            // Render Characters
            for char in chars {
                let card_width = 180.0;
                let card_height = 260.0;
                
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(card_width, card_height),
                    egui::Sense::click()
                );
                
                // Hover Effect
                let bg_color = if response.hovered() {
                    ui.visuals().widgets.hovered.bg_fill
                } else {
                    ui.visuals().widgets.noninteractive.bg_fill
                };
                
                ui.painter().rect_filled(rect, 8.0, bg_color);
                ui.painter().rect_stroke(rect, 8.0, ui.visuals().widgets.noninteractive.bg_stroke);

                // Interaction
                if response.clicked() {
                    app.selected_character = Some(char.clone());
                    app.central_view = crate::ui::CentralView::Editor;
                    app.load_tags(char.id);
                    app.load_links(char.id);
                }
                
                response.context_menu(|ui| {
                     ui.menu_button("Move to...", |ui| {
                         if ui.button("Root (Uncategorized)").clicked() {
                             actions.push(BrowserAction::MoveCharacter(char.id, None));
                             ui.close_menu();
                         }
                         ui.separator();
                         render_collection_move_menu(ui, &all_collections, None, char.id, &mut actions);
                     });
                     
                     if ui.button("🗑 Delete").clicked() {
                         actions.push(BrowserAction::DeleteCharacter(char.id));
                         ui.close_menu();
                     }
                });
                
                // Content
                let content_rect = rect.shrink(8.0);
                
                // Avatar (Top Square)
                let avatar_size = content_rect.width();
                let avatar_rect = egui::Rect::from_min_size(content_rect.min, egui::vec2(avatar_size, avatar_size));
                
                if let Some(path_str) = &char.avatar_path {
                     let uri = if path_str.contains("://") { 
                         path_str.clone() 
                     } else {
                         if let Ok(abs_path) = std::fs::canonicalize(path_str) {
                              format!("file://{}", abs_path.to_string_lossy())
                         } else {
                              path_str.clone() 
                         }
                     };
                     crate::ui::widgets::paint_avatar_crop(ui, avatar_rect, &uri, 4.0);
                } else {
                     ui.painter().rect_filled(avatar_rect, 4.0, egui::Color32::from_gray(60));
                     let initial = char.name.chars().next().unwrap_or('?').to_uppercase().to_string();
                     ui.painter().text(
                         avatar_rect.center(),
                         egui::Align2::CENTER_CENTER,
                         initial,
                         egui::FontId::proportional(40.0),
                         egui::Color32::WHITE
                     );
                }
                
                // Text Area
                let text_top = avatar_rect.max.y + 8.0;
                let _text_rect = egui::Rect::from_min_max(
                    egui::pos2(content_rect.min.x, text_top),
                    content_rect.max
                );
                
                let mut cursor_y = text_top;
                
                // Name
                let name_font = egui::FontId::proportional(16.0);
                let name_galley = ui.painter().layout_no_wrap(char.name.clone(), name_font.clone(), ui.visuals().text_color());
                ui.painter().galley(egui::pos2(content_rect.min.x, cursor_y), name_galley, ui.visuals().text_color());
                cursor_y += 20.0;
                
                // Title
                if !char.char_title.is_empty() {
                    let title_font = egui::FontId::proportional(12.0);
                    let title_galley = ui.painter().layout_no_wrap(char.char_title.clone(), title_font, ui.visuals().text_color().linear_multiply(0.7));
                    ui.painter().with_clip_rect(rect).galley(egui::pos2(content_rect.min.x, cursor_y), title_galley, ui.visuals().text_color());
                    cursor_y += 16.0;
                } else {
                     cursor_y += 16.0; // Spacer
                }
                
                cursor_y += 4.0;
                
                // Tags (Chips)
                let tag_font = egui::FontId::proportional(10.0);
                let mut tag_x = content_rect.min.x;
                
                for tag in char.app_tags.iter().take(3) {
                    let tag_galley = ui.painter().layout_no_wrap(tag.name.clone(), tag_font.clone(), egui::Color32::WHITE);
                    let pad = 4.0;
                    let chip_w = tag_galley.rect.width() + pad * 2.0;
                    
                    if tag_x + chip_w > content_rect.max.x { break; }
                    
                    let chip_rect = egui::Rect::from_min_size(egui::pos2(tag_x, cursor_y), egui::vec2(chip_w, 16.0));
                    ui.painter().rect_filled(chip_rect, 8.0, egui::Color32::from_rgb(50, 80, 150));
                    ui.painter().galley(egui::pos2(tag_x + pad, cursor_y + 2.0), tag_galley, egui::Color32::WHITE);
                    
                    tag_x += chip_w + 4.0;
                }
            }
        });
    });
    
    // Handle Actions
    for action in actions {
        match action {
            BrowserAction::MoveCharacter(char_id, target_id) => {
                app.move_character(char_id, target_id);
            },
            BrowserAction::DeleteCharacter(id) => {
                let name = app.characters.iter().find(|c| c.id == id).map(|c| c.name.clone()).unwrap_or_default();
                app.popup_state = crate::ui::PopupState::DeleteCharacterConfirmation { id, name };
            },
            BrowserAction::RenameCollection(id, name) => {
                app.popup_state = crate::ui::PopupState::Renaming { id, name };
            },
            BrowserAction::DeleteCollection(id) => {
                // Calculate count for warning
                 let count = app.collections.iter().filter(|c| c.parent_id == Some(id)).count() + 
                             app.characters.iter().filter(|c| c.collection_id == Some(id)).count();
                 
                 if count > 0 {
                     app.popup_state = crate::ui::PopupState::DeleteWarning { id, count };
                 } else {
                     app.delete_collection(id);
                 }
            }
        }
    }
}

fn render_editor_view(app: &mut CrapApp, ui: &mut egui::Ui) {
    let mut trigger_import = false;

                let mut save_req = None;
                let mut toggle_requests = Vec::new();
                let mut status_update = None;  

                
                // Check Dirty State
                let is_dirty = if let Some(selected) = &app.selected_character {
                    if selected.id == 0 {
                        true
                    } else {
                        // Compare with DB version in app.characters
                        // Note: This relies on app.characters being up to date.
                        // Ideally we should have a separate "original" but app.characters is the cache.
                        if let Some(original) = app.characters.iter().find(|c| c.id == selected.id) {
                            !selected.content_eq(original)
                        } else {
                            true // Should not happen if data is consistent
                        }
                    }
                } else {
                    false
                };

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
                                 ui.horizontal(|ui| {
                                     let available_width = ui.available_width();
                                     let left_width = available_width * 0.66;
                                     // Right width is remaining
                                     
                                     ui.allocate_ui_with_layout(egui::vec2(left_width, ui.available_height()), egui::Layout::top_down(egui::Align::Min), |ui| {
                                         ui.label("Name (File Name)");
                                         ui.add(egui::TextEdit::singleline(&mut character.name).desired_width(f32::INFINITY));
                                         ui.label("Character Name");
                                         ui.add(egui::TextEdit::singleline(&mut character.char_name).desired_width(f32::INFINITY));
                                         ui.label("Title");
                                         ui.add(egui::TextEdit::singleline(&mut character.char_title).desired_width(f32::INFINITY));
                                         ui.add_space(8.0);
                                         
                                         ui.label("First Message");
                                         ui.add(egui::TextEdit::multiline(&mut character.first_message).desired_width(f32::INFINITY));
                                         ui.label(egui::RichText::new(format!("Tokens: {}", count_tokens(&character.first_message))).size(10.0).color(egui::Color32::GRAY));
                                         
                                         ui.add_space(8.0);
                                         ui.label("Personality");
                                         ui.add(egui::TextEdit::multiline(&mut character.personality).desired_width(f32::INFINITY));
                                         ui.label(egui::RichText::new(format!("Tokens: {}", count_tokens(&character.personality))).size(10.0).color(egui::Color32::GRAY));
        
                                         ui.label("Scenario");
                                         ui.add(egui::TextEdit::multiline(&mut character.scenario).desired_width(f32::INFINITY));
                                         ui.label(egui::RichText::new(format!("Tokens: {}", count_tokens(&character.scenario))).size(10.0).color(egui::Color32::GRAY));
        
                                         ui.label("Example Dialogue");
                                         ui.add(egui::TextEdit::multiline(&mut character.example_dialogue).desired_width(f32::INFINITY));
                                         ui.label(egui::RichText::new(format!("Tokens: {}", count_tokens(&character.example_dialogue))).size(10.0).color(egui::Color32::GRAY));
                                         
                                         egui::CollapsingHeader::new("Tags & Metadata")
                                            .default_open(true)
                                            .show(ui, |ui| {
                                                ui.vertical(|ui| {
                                                     // App Tags
                                                    ui.label(egui::RichText::new("CRApp Tags").strong().color(egui::Color32::from_rgb(100, 150, 255)));
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
                                     });
                                 });
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
                } else {
                    ui.label("Select a character to edit.");
                }
                
                if let Some(c) = save_req {
                    app.save_character(c);
                }
    }


fn render_lorebook_editor(app: &mut CrapApp, ui: &mut egui::Ui) {
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
            }










