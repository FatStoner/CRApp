use super::state::CrapApp;
use crate::card_v2::{CharacterCardV2, TavernCardV2};
use crate::models::Character;
use crate::ui::UiEvent;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

impl CrapApp {
    /// Export character as native .crapp format (full character JSON)
    pub fn export_character_native(&self, character: &Character) {
        if let Ok(json) = serde_json::to_string_pretty(&character) {
            let name_slug = character.name.replace(" ", "_");
            let task_name = format!("{}.crapp", name_slug);
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
    }

    /// Export character as SpicyChat-compatible JSON (Character Card V2 format)
    pub fn export_character_v2_json(&self, character: &Character) {
        let v2 = CharacterCardV2::new(
            character.char_name.clone(),
            character.personality.clone(),
            character.char_title.clone(),
            character.scenario.clone(),
            character.first_message.clone(),
            character.example_dialogue.clone(),
        );
        if let Ok(json) = serde_json::to_string_pretty(&v2) {
            let name_slug = character.name.replace(" ", "_");
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
    }

    /// Export character as Markdown document
    pub fn export_character_markdown(&self, character: &Character) {
        let md = format!(
            "# {}\n\n## Description\n{}\n\n## Personality\n{}\n\n## Scenario\n{}\n\n## First Message\n{}\n\n## Example Dialogue\n{}\n",
            character.char_name,
            character.char_title,
            character.personality,
            character.scenario,
            character.first_message,
            character.example_dialogue
        );
        let name_slug = character.name.replace(" ", "_");
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
    }

    /// Export character as PNG card (TavernAI format with embedded metadata)
    pub fn export_character_png(&self, character: &Character) {
        if let Some(avatar_path) = &character.avatar_path {
            let mut v2 = TavernCardV2::new(
                character.char_name.clone(),
                character.personality.clone(),
                character.char_title.clone(),
                character.scenario.clone(),
                character.first_message.clone(),
                character.example_dialogue.clone(),
            );
            v2.data.creator_notes = character.author_notes.clone();
            v2.data.tags = character
                .app_tags
                .iter()
                .chain(character.external_tags.iter())
                .map(|t| t.name.clone())
                .collect();

            if let Ok(json) = serde_json::to_string(&v2) {
                let b64 = BASE64.encode(json);
                let path_clone = avatar_path.clone();
                let name_slug = character.name.replace(" ", "_");
                let task_name = format!("{}.png", name_slug);

                tokio::spawn(async move {
                    if let Ok(img_bytes) = std::fs::read(&path_clone) {
                        if let Some(save_path) = rfd::FileDialog::new()
                            .set_directory("exports")
                            .set_file_name(task_name)
                            .save_file()
                        {
                            if let Ok(img) = image::load_from_memory(&img_bytes) {
                                let (w, h) = (img.width(), img.height());
                                let color_type = img.color();
                                let pixels = img.into_bytes();

                                if let Ok(mut out_file) = std::fs::File::create(save_path) {
                                    let mut encoder = png::Encoder::new(&mut out_file, w, h);
                                    encoder.set_color(match color_type {
                                        image::ColorType::Rgb8 => png::ColorType::Rgb,
                                        image::ColorType::Rgba8 => png::ColorType::Rgba,
                                        image::ColorType::L8 => png::ColorType::Grayscale,
                                        image::ColorType::La8 => png::ColorType::GrayscaleAlpha,
                                        _ => png::ColorType::Rgba,
                                    });
                                    encoder.set_depth(png::BitDepth::Eight);
                                    let _ = encoder
                                        .add_text_chunk("chara".to_string(), b64.to_string());

                                    if let Ok(mut writer) = encoder.write_header() {
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
            eprintln!("Cannot export PNG: No avatar.");
        }
    }

    /// Import character from file (JSON, PNG, or CRAPP)
    pub fn import_character_from_file(&self, target_id: Option<u64>) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Supported", &["crapp", "json", "png"])
                .pick_file()
            {
                match std::fs::read(&path) {
                    Ok(bytes) => {
                        let ext = path
                            .extension()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_lowercase();

                        let result = if ext == "png" {
                            // Parse PNG card
                            match crate::ui::parsing::parse_png_card(&bytes) {
                                Ok(mut parsed) => {
                                    // Save avatar
                                    let dest_dir = std::path::Path::new("data/avatars");
                                    let _ = std::fs::create_dir_all(dest_dir);
                                    let file_name =
                                        format!("imported_{}.png", uuid::Uuid::new_v4());
                                    let dest_path = dest_dir.join(&file_name);

                                    if let Ok(_) = std::fs::write(&dest_path, &bytes) {
                                        parsed.avatar_path =
                                            Some(dest_path.to_string_lossy().to_string());
                                    }
                                    Ok(parsed)
                                }
                                Err(e) => Err(e),
                            }
                        } else {
                            // Try JSON / Native
                            match String::from_utf8(bytes) {
                                Ok(text) => {
                                    if ext == "crapp" {
                                        if let Ok(mut char_obj) =
                                            serde_json::from_str::<crate::models::Character>(&text)
                                        {
                                            char_obj.id = 0;
                                            let parsed = crate::ui::parsing::ParsedCharacterData {
                                                name: char_obj.name,
                                                title: char_obj.char_title,
                                                personality: char_obj.personality,
                                                scenario: char_obj.scenario,
                                                first_message: char_obj.first_message,
                                                example_dialogue: char_obj.example_dialogue,
                                                external_tags: char_obj
                                                    .external_tags
                                                    .into_iter()
                                                    .map(|t| t.name)
                                                    .collect(),
                                                app_tags: char_obj
                                                    .app_tags
                                                    .into_iter()
                                                    .map(|t| t.name)
                                                    .collect(),
                                                urls: char_obj.urls,
                                                avatar_path: char_obj.avatar_path,
                                            };
                                            Ok(parsed)
                                        } else {
                                            Err("Failed to parse native .crapp file".to_string())
                                        }
                                    } else {
                                        // .json -> Try V2 first
                                        if let Ok(parsed) = crate::ui::parsing::parse_v2_card(&text)
                                        {
                                            Ok(parsed)
                                        } else {
                                            // Fallback to native
                                            if let Ok(mut char_obj) =
                                                serde_json::from_str::<crate::models::Character>(
                                                    &text,
                                                )
                                            {
                                                char_obj.id = 0;
                                                let parsed =
                                                    crate::ui::parsing::ParsedCharacterData {
                                                        name: char_obj.name,
                                                        title: char_obj.char_title,
                                                        personality: char_obj.personality,
                                                        scenario: char_obj.scenario,
                                                        first_message: char_obj.first_message,
                                                        example_dialogue: char_obj.example_dialogue,
                                                        external_tags: char_obj
                                                            .external_tags
                                                            .into_iter()
                                                            .map(|t| t.name)
                                                            .collect(),
                                                        app_tags: char_obj
                                                            .app_tags
                                                            .into_iter()
                                                            .map(|t| t.name)
                                                            .collect(),
                                                        urls: char_obj.urls,
                                                        avatar_path: char_obj.avatar_path,
                                                    };
                                                Ok(parsed)
                                            } else {
                                                Err("Failed to parse JSON (Tried V2 and Native)"
                                                    .to_string())
                                            }
                                        }
                                    }
                                }
                                Err(e) => Err(format!("Invalid UTF-8: {}", e)),
                            }
                        };

                        let _ = tx
                            .send(UiEvent::ImportCharacterData(result, target_id))
                            .await;
                    }
                    Err(e) => {
                        let _ = tx
                            .send(UiEvent::ImportCharacterData(Err(e.to_string()), target_id))
                            .await;
                    }
                }
            }
        });
    }
}
