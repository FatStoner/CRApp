use super::state::CrapApp;
use crate::card_v2::{CharacterCardV2, TavernCardV2};
use crate::models::Character;
use crate::ui::UiEvent;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

impl CrapApp {
    /// Export character in a specific format to a target folder
    pub fn export_character_in_format(
        &self,
        character: &Character,
        target_folder: &std::path::Path,
        format: crate::ui::ExportFormat,
    ) -> Result<String, String> {
        let name_slug = character.name.replace(" ", "_");
        let file_name = match format {
            crate::ui::ExportFormat::Png => format!("{}.png", name_slug),
            crate::ui::ExportFormat::V2 => format!("{}.json", name_slug),
            crate::ui::ExportFormat::Native => format!("{}.crapp", name_slug),
            crate::ui::ExportFormat::Markdown => format!("{}.md", name_slug),
        };
        let target_path = target_folder.join(file_name);

        match format {
            crate::ui::ExportFormat::Png => self.write_character_png(character, &target_path),
            crate::ui::ExportFormat::V2 => self.write_character_v2_json(character, &target_path),
            crate::ui::ExportFormat::Native => self.write_character_native(character, &target_path),
            crate::ui::ExportFormat::Markdown => {
                self.write_character_markdown(character, &target_path)
            }
        }
        .map(|_| target_path.to_string_lossy().to_string())
    }

    /// Export character as native .crapp format (full character JSON)
    pub fn export_character_native(&self, character: &Character) {
        let char_clone = character.clone();

        let name_slug = character.name.replace(" ", "_");
        let task_name = format!("{}.crapp", name_slug);

        tokio::task::spawn_blocking(move || {
            if let Some(path) = rfd::FileDialog::new()
                .set_directory("exports")
                .set_file_name(task_name)
                .save_file()
            {
                let _ = CrapApp::write_character_native_static(&char_clone, &path);
            }
        });
    }

    // Static versions for Async usage
    pub fn write_character_native_static(
        character: &Character,
        path: &std::path::Path,
    ) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&character).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }

    pub fn write_character_v2_json_static(
        character: &Character,
        path: &std::path::Path,
    ) -> Result<(), String> {
        let v2 = CharacterCardV2::new(
            character.char_name.clone(),
            character.personality.clone(),
            character.char_title.clone(),
            character.scenario.clone(),
            character.first_message.clone(),
            character.example_dialogue.clone(),
        );
        let json = serde_json::to_string_pretty(&v2).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }

    pub fn write_character_markdown_static(
        character: &Character,
        path: &std::path::Path,
    ) -> Result<(), String> {
        let md = format!(
            "# {}\n\n## Description\n{}\n\n## Personality\n{}\n\n## Scenario\n{}\n\n## First Message\n{}\n\n## Example Dialogue\n{}\n",
            character.char_name,
            character.char_title,
            character.personality,
            character.scenario,
            character.first_message,
            character.example_dialogue
        );
        std::fs::write(path, md).map_err(|e| e.to_string())
    }

    pub fn write_character_png_static(
        character: &Character,
        path: &std::path::Path,
    ) -> Result<(), String> {
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

            let json = serde_json::to_string(&v2).map_err(|e| e.to_string())?;
            let b64 = BASE64.encode(json);

            let img_bytes = std::fs::read(avatar_path).map_err(|e| e.to_string())?;
            let img = image::load_from_memory(&img_bytes).map_err(|e| e.to_string())?;

            let (w, h) = (img.width(), img.height());
            let color_type = img.color();
            let pixels = img.into_bytes();

            let mut out_file = std::fs::File::create(path).map_err(|e| e.to_string())?;
            let mut encoder = png::Encoder::new(&mut out_file, w, h);
            encoder.set_color(match color_type {
                image::ColorType::Rgb8 => png::ColorType::Rgb,
                image::ColorType::Rgba8 => png::ColorType::Rgba,
                image::ColorType::L8 => png::ColorType::Grayscale,
                image::ColorType::La8 => png::ColorType::GrayscaleAlpha,
                _ => png::ColorType::Rgba,
            });
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .add_text_chunk("chara".to_string(), b64.to_string())
                .map_err(|e| e.to_string())?;

            let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
            writer
                .write_image_data(&pixels)
                .map_err(|e| e.to_string())?;
            writer.finish().map_err(|e| e.to_string())?;

            Ok(())
        } else {
            CrapApp::write_character_v2_json_static(
                character,
                path.with_extension("json").as_path(),
            )
        }
    }

    // Instance methods wrappers for compatibility and cleaner calls
    pub fn write_character_native(
        &self,
        character: &Character,
        path: &std::path::Path,
    ) -> Result<(), String> {
        Self::write_character_native_static(character, path)
    }
    pub fn write_character_v2_json(
        &self,
        character: &Character,
        path: &std::path::Path,
    ) -> Result<(), String> {
        Self::write_character_v2_json_static(character, path)
    }
    pub fn write_character_markdown(
        &self,
        character: &Character,
        path: &std::path::Path,
    ) -> Result<(), String> {
        Self::write_character_markdown_static(character, path)
    }
    pub fn write_character_png(
        &self,
        character: &Character,
        path: &std::path::Path,
    ) -> Result<(), String> {
        Self::write_character_png_static(character, path)
    }

    /// Export character as SpicyChat-compatible JSON (Character Card V2 format)
    pub fn export_character_v2_json(&self, character: &Character) {
        let char_clone = character.clone();
        let name_slug = character.name.replace(" ", "_");
        let task_name = format!("{}.json", name_slug);
        tokio::task::spawn_blocking(move || {
            if let Some(path) = rfd::FileDialog::new()
                .set_directory("exports")
                .set_file_name(task_name)
                .save_file()
            {
                let _ = CrapApp::write_character_v2_json_static(&char_clone, &path);
            }
        });
    }

    /// Export character as Markdown document
    pub fn export_character_markdown(&self, character: &Character) {
        let char_clone = character.clone();
        let name_slug = character.name.replace(" ", "_");
        let task_name = format!("{}.md", name_slug);
        tokio::task::spawn_blocking(move || {
            if let Some(path) = rfd::FileDialog::new()
                .set_directory("exports")
                .set_file_name(task_name)
                .save_file()
            {
                let _ = CrapApp::write_character_markdown_static(&char_clone, &path);
            }
        });
    }

    /// Export character as PNG card (TavernAI format with embedded metadata)
    pub fn export_character_png(&self, character: &Character) {
        let char_clone = character.clone();
        let name_slug = character.name.replace(" ", "_");
        let task_name = format!("{}.png", name_slug);
        tokio::task::spawn_blocking(move || {
            if let Some(path) = rfd::FileDialog::new()
                .set_directory("exports")
                .set_file_name(task_name)
                .save_file()
            {
                let _ = CrapApp::write_character_png_static(&char_clone, &path);
            }
        });
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
                                                char_name: char_obj.char_name,
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
                                                        char_name: char_obj.char_name,
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
