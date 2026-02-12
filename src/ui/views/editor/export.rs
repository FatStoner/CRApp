use crate::card_v2::CharacterCardV2;
use crate::models::Character;
use crate::ui::UiEvent;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use eframe::egui;
use tokio::sync::mpsc::Sender;

/// Renders the EXPORT menu button with all export options
pub fn render_export_menu(ui: &mut egui::Ui, character: &Character) {
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
                let mut v2 = crate::card_v2::TavernCardV2::new(
                    character.char_name.clone(),
                    character.personality.clone(),
                    character.char_title.clone(),
                    character.scenario.clone(),
                    character.first_message.clone(),
                    character.example_dialogue.clone(),
                );
                // Include extra fields if available in model
                v2.data.creator_notes = character.author_notes.clone();
                // Populate tags
                v2.data.tags = character.app_tags.iter().chain(character.external_tags.iter()).map(|t| t.name.clone()).collect();
                
                if let Ok(json) = serde_json::to_string(&v2) {
                    let b64 = BASE64.encode(json);
                    let path_clone = avatar_path.clone(); // Path string
                    let task_name = format!("{}.png", name_slug);
                    
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
                                        // Add tEXt chunk
                                        // TavernAI spec requires tEXt chunk with keyword "chara" and value as base64 encoded JSON
                                        // add_text_chunk adds a tEXt chunk.
                                            let _ = encoder.add_text_chunk("chara".to_string(), b64.to_string());
                                                                    
                                        let mut writer = encoder.write_header().expect("Failed to write PNG header");
                                        let _ = writer.write_image_data(&pixels);
                                        let _ = writer.finish();
                                    }
                                }
                            }
                        }
                    });
                }
            } else {
                eprintln!("Cannot export PNG: No avatar.");
            }
            ui.close_menu();
        }
    });
}

/// Renders the IMPORT menu button with all import options
pub fn render_import_menu(
    ui: &mut egui::Ui,
    character: &Character,
    tx: &Sender<UiEvent>,
    trigger_import: &mut bool,
) {
    ui.menu_button("IMPORT", |ui| {
        if ui.button("Import File (JSON, PNG, CRAPP)").clicked() {
            let tx_clone = tx.clone();
            // Capture target_id to move into the async block
            let target_id_clone = if character.id != 0 { Some(character.id as u64) } else { None };
            
            tokio::spawn(async move {
                let target_id = target_id_clone; // Move it here
                if let Some(path) = rfd::FileDialog::new().add_filter("Supported", &["crapp", "json", "png"]).pick_file() {
                    match std::fs::read(&path) {
                        Ok(bytes) => {
                            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                            
                            let result = if ext == "png" {
                                // 1. Parse Metadata
                                match crate::ui::parsing::parse_png_card(&bytes) {
                                    Ok(mut parsed) => {
                                        // 2. Save Avatar
                                        // We copy the original PNG to avatars folder
                                        let dest_dir = std::path::Path::new("data/avatars");
                                        let _ = std::fs::create_dir_all(dest_dir);
                                        let file_name = format!("imported_{}.png", uuid::Uuid::new_v4());
                                        let dest_path = dest_dir.join(&file_name);
                                        
                                        if let Ok(_) = std::fs::write(&dest_path, &bytes) {
                                            parsed.avatar_path = Some(dest_path.to_string_lossy().to_string());
                                        }
                                        Ok(parsed)
                                    },
                                    Err(e) => Err(e),
                                }
                            } else {
                                // Try JSON / Native
                                // For now, treat both as JSON text
                                match String::from_utf8(bytes) {
                                    Ok(text) => {
                                        if ext == "crapp" {
                                            if let Ok(mut char_obj) = serde_json::from_str::<crate::models::Character>(&text) {
                                                 char_obj.id = 0;
                                                 let parsed = crate::ui::parsing::ParsedCharacterData {
                                                    name: char_obj.name,
                                                    title: char_obj.char_title,
                                                    personality: char_obj.personality,
                                                    scenario: char_obj.scenario,
                                                    first_message: char_obj.first_message,
                                                    example_dialogue: char_obj.example_dialogue,
                                                    external_tags: char_obj.external_tags.into_iter().map(|t| t.name).collect(),
                                                    app_tags: char_obj.app_tags.into_iter().map(|t| t.name).collect(),
                                                    urls: char_obj.urls,
                                                    avatar_path: char_obj.avatar_path,
                                                 };
                                                 Ok(parsed)
                                            } else {
                                                Err("Failed to parse native .crapp file".to_string())
                                            }
                                        } else {
                                            // .json -> Try V2 first
                                            if let Ok(parsed) = crate::ui::parsing::parse_v2_card(&text) {
                                                Ok(parsed)
                                            } else {
                                                 // Fallback to native
                                                  if let Ok(mut char_obj) = serde_json::from_str::<crate::models::Character>(&text) {
                                                     char_obj.id = 0;
                                                     let parsed = crate::ui::parsing::ParsedCharacterData {
                                                        name: char_obj.name,
                                                        title: char_obj.char_title,
                                                        personality: char_obj.personality,
                                                        scenario: char_obj.scenario,
                                                        first_message: char_obj.first_message,
                                                        example_dialogue: char_obj.example_dialogue,
                                                        external_tags: char_obj.external_tags.into_iter().map(|t| t.name).collect(),
                                                        app_tags: char_obj.app_tags.into_iter().map(|t| t.name).collect(),
                                                        urls: char_obj.urls,
                                                        avatar_path: char_obj.avatar_path,
                                                     };
                                                     Ok(parsed)
                                                } else {
                                                    Err("Failed to parse JSON (Tried V2 and Native)".to_string())
                                                }
                                            }
                                        }
                                    },
                                    Err(e) => Err(format!("Invalid UTF-8: {}", e)),
                                }
                            };
                            
                            let _ = tx_clone.send(UiEvent::ImportCharacterData(result, target_id)).await;
                        },
                        Err(e) => {
                            let _ = tx_clone.send(UiEvent::ImportCharacterData(Err(e.to_string()), target_id)).await;
                        }
                    }
                }
            });
            ui.close_menu();
        }
        
        if ui.button("Import from Clipboard").clicked() {
            *trigger_import = true;
            ui.close_menu();
        }
    });
}
