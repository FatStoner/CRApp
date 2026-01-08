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
    
    // Pre-processing: simple clean lines
    let lines: Vec<&str> = text.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    if lines.is_empty() {
        return data;
    }

    // -------------------------------------------------------------------------
    // Strategy 1: Structural Parsing (Spicychat Copy-Paste Format)
    // -------------------------------------------------------------------------
    
    // Find Anchors
    let idx_back = lines.iter().position(|&l| l == "Back");
    let idx_share = lines.iter().position(|&l| l == "Share").or_else(|| lines.iter().position(|&l| l == "Favorite")); 
    let idx_suggest_tag = lines.iter().position(|&l| l.eq_ignore_ascii_case("suggest tag"));
    let idx_greeting_header = lines.iter().position(|&l| l.eq_ignore_ascii_case("greeting"));
    let idx_show_less = lines.iter().position(|&l| l.eq_ignore_ascii_case("show less"));
    let idx_personality = lines.iter().position(|&l| l.eq_ignore_ascii_case("personality"));
    let idx_scenario = lines.iter().position(|&l| l.eq_ignore_ascii_case("scenario"));

    // 1. NAME
    // Pattern: Back -> avatar image -> NAME
    if let Some(idx) = idx_back {
        // Look for "avatar image" in next few lines
        if let Some(offset) = lines.iter().skip(idx).take(3).position(|&l| l == "avatar image") {
            let name_idx = idx + offset + 1;
            if name_idx < lines.len() {
                data.name = lines[name_idx].to_string();
            }
        }
    }

    // 2. TITLE & TAGS
    // Pattern: Share -> [Stats x3] -> TITLE -> TAGS... -> Suggest Tag
    // Note: Stats lines can be 2 or 3. 
    // Heuristic: Scan from Share+1 until Suggest Tag.
    if let Some(start) = idx_share {
        if let Some(end) = idx_suggest_tag {
            if end > start {
                let range = &lines[(start + 1)..end];
                // We typically have: [Num], [Percent], [Tokens], [Title], [Tag1], [Tag2]...
                // Filter out the stats (digits, %, "tokens")
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
                    // First candidate is Title
                    data.title = candidates[0].to_string();
                    
                    // Rest are tags
                    for tag in candidates.iter().skip(1) {
                        data.external_tags.push(tag.to_string());
                    }
                }
            }
        }
    }

    // 3. FIRST MESSAGE
    // Pattern: Greeting -> [Message Lines] -> (Next Header: SHOW LESS, Personality, Scenario, or end)
    if let Some(start) = idx_greeting_header {
        let mut end = lines.len();
        
        // Find nearest next section/stopper
        let stoppers = [idx_show_less, idx_personality, idx_scenario];
        for &stop in stoppers.iter().flatten() {
            if stop > start && stop < end {
                end = stop;
            }
        }
        
        for i in (start + 1)..end {
            data.first_message.push_str(lines[i]);
            data.first_message.push('\n');
        }
    }

    // -------------------------------------------------------------------------
    // Strategy 2: Scan for Blocks (Personality, Scenario) & Key-Values
    // -------------------------------------------------------------------------
    
    let mut current_section = "";
    
    for line in &lines {
        let lower = line.to_lowercase();
        
        // Key-Value Detection (High priority overrides)
        if lower.starts_with("name:") && data.name.is_empty() {
             if let Some((_, val)) = line.split_once(':') {
                 data.name = val.trim().to_string();
             }
        }
        
        // Section Headers
        if lower == "personality" { current_section = "personality"; continue; }
        if lower == "scenario" { current_section = "scenario"; continue; }
        if lower == "greeting" || lower == "first message" { current_section = "ignore"; continue; } // Handled structurally
        if lower == "show less" { current_section = "ignore"; continue; }
        
        // Footer Detection (Stop parsing)
        if lower == "spicychat" || lower.starts_with("owned & operated by") {
            current_section = "ignore";
            continue;
        }
        
        // Append to sections
        match current_section {
            "personality" => {
                // Avoid capturing other headers if they accidentally triggered
                 if lower != "scenario" {
                    data.personality.push_str(line);
                    data.personality.push('\n');
                 }
            },
            "scenario" => {
                 data.scenario.push_str(line);
                 data.scenario.push('\n');
            },
            _ => {}
        }
    }

    // Cleanup
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

fn render_browser_view(app: &mut CrapApp, ui: &mut egui::Ui) {
    let collection_id = app.selected_collection_id;
    let collection_name = if let Some(id) = collection_id {
        app.collections.iter().find(|c| c.id == id).map(|c| c.name.clone()).unwrap_or("Unknown".to_string())
    } else {
        "All Characters".to_string()
    };
    
    let parent_id = if let Some(id) = collection_id {
        app.collections.iter().find(|c| c.id == id).and_then(|c| c.parent_id)
    } else {
        None
    };

    ui.horizontal(|ui| {
        if collection_id.is_some() {
             if ui.button("⬅ Back").clicked() {
                 app.selected_collection_id = parent_id;
             }
        }
        ui.heading(format!("Browsing: {}", collection_name));
    });
    ui.add_space(10.0);
    
    // Filter characters
    // Note: We are filtering by direct parent currently.
    // If "All Characters", show all? Or show root?
    // Side panel logic implies "No selection" = "All/Root". 
    // Let's assume Root means parent_id == None if we strictly follow folder logic.
    // But typically "All Characters" view shows everything flat.
    // User requested "Browser View... activates when selecting a collection".
    // If selected_collection_id is None, let's show ALL characters for now, or Root? 
    // SidePanel uses "None" for Root in tree view.
    // Let's filter: if Some(id) -> c.collection_id == Some(id).
    // If None -> Show ALL (Flat view) is usually more useful for a "Browser".
    // Or if None -> c.collection_id == None (Root items only).
    // Let's go with: If collection selected, show content. If None, show nothing/instruction? 
    // Wait, side panel sets Browser view on "Deselect".
    // Let's show "Uncategorized" (Root) if None? Or All?
    // Let's show matching `collection_id` exactly for now. 
    // If None, show those with None (Uncategorized).
    
    let subfolders: Vec<crate::models::Collection> = app.collections.iter()
        .filter(|c| c.parent_id == collection_id)
        .cloned()
        .collect();

    let chars: Vec<crate::models::Character> = app.characters.iter()
        .filter(|c| c.collection_id == collection_id)
        .cloned()
        .collect();

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
                     egui::Image::new(uri)
                         .rounding(egui::Rounding::same(4.0))
                         .paint_at(ui, avatar_rect);
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
                let text_rect = egui::Rect::from_min_max(
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
}

fn render_editor_view(app: &mut CrapApp, ui: &mut egui::Ui) {
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








