use super::state::CrapApp;
use crate::ui::types::UiEvent;
use base64::Engine as _;

impl CrapApp {
    /// Exports database to a file (DB only, no data folder)
    pub fn trigger_db_export_file_only(&self) {
        let db = self.db.clone();
        let tx = self.tx.clone();
        let ctx = self.ctx.clone();
        crate::task::spawn_supervised(ctx.clone(), async move {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Export Database Value")
                .set_file_name(
                    format!(
                        "crap_data_backup_{}.db",
                        chrono::Local::now().format("%Y%m%d_%H%M%S")
                    )
                    .as_str(),
                )
                .save_file()
            {
                let target_str = path.to_string_lossy().to_string();

                // Use safe vacuum into
                match db.create_checkpoint_and_vacuum(&target_str).await {
                    Ok(_) => {
                        let _ = tx.send(UiEvent::DbExportFinished(Ok(target_str))).await;
                    }
                    Err(e) => {
                        let _ = tx
                            .send(UiEvent::DbExportFinished(Err(format!(
                                "Export Failed: {}",
                                e
                            ))))
                            .await;
                    }
                }
            }
            ctx.request_repaint();
            Ok(())
        }, self.tx.clone());
    }

    /// Exports full backup as ZIP (database + data folder)
    pub fn perform_full_zip_export(&self) {
        let db = self.db.clone();
        let tx = self.tx.clone();
        let ctx = self.ctx.clone();
        crate::task::spawn_supervised(ctx.clone(), async move {
            if let Some(zip_path) = rfd::FileDialog::new()
                .set_title("Export Full Backup")
                .set_file_name(
                    format!(
                        "crap_backup_{}.zip",
                        chrono::Local::now().format("%Y%m%d_%H%M%S")
                    )
                    .as_str(),
                )
                .save_file()
            {
                // 1. Create Temp DB Snapshot
                let temp_db_name = format!("temp_snapshot_{}.db", uuid::Uuid::new_v4());
                let temp_db_path = std::env::temp_dir().join(&temp_db_name);
                let temp_db_str = temp_db_path.to_string_lossy().to_string();

                if let Err(e) = db.create_checkpoint_and_vacuum(&temp_db_str).await {
                    let _ = tx
                        .send(UiEvent::DbExportFinished(Err(format!(
                            "Snapshot Failed: {}",
                            e
                        ))))
                        .await;
                    return Ok(());
                }

                // 2. Create Zip
                let file = match std::fs::File::create(&zip_path) {
                    Ok(f) => f,
                    Err(e) => {
                        let _ = tx
                            .send(UiEvent::DbExportFinished(Err(format!(
                                "Create Zip Failed: {}",
                                e
                            ))))
                            .await;
                        // Cleanup
                        let _ = std::fs::remove_file(&temp_db_path);
                        return Ok(());
                    }
                };

                let mut zip = zip::ZipWriter::new(file);
                let options = zip::write::FileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated)
                    .unix_permissions(0o755);

                // 3. Add DB
                if let Err(e) = zip.start_file("crap_data.db", options) {
                    let _ = tx
                        .send(UiEvent::DbExportFinished(Err(format!(
                            "Zip DB Add Failed: {}",
                            e
                        ))))
                        .await;
                    let _ = std::fs::remove_file(&temp_db_path);
                    return Ok(());
                }

                if let Ok(mut f) = std::fs::File::open(&temp_db_path) {
                    if let Err(e) = std::io::copy(&mut f, &mut zip) {
                        let _ = tx
                            .send(UiEvent::DbExportFinished(Err(format!(
                                "Zip DB Write Failed: {}",
                                e
                            ))))
                            .await;
                        let _ = std::fs::remove_file(&temp_db_path);
                        return Ok(());
                    }
                }

                // Cleanup snapshot
                let _ = std::fs::remove_file(&temp_db_path);

                // 4. Add 'data' folder
                let data_dir = std::path::Path::new("data");
                if data_dir.exists() {
                    let walk = walkdir::WalkDir::new(data_dir);
                    for entry in walk.into_iter().filter_map(|e| e.ok()) {
                        let path = entry.path();
                        let name = path.strip_prefix(std::path::Path::new(".")).unwrap_or(path);
                        let name_str = name.to_string_lossy().replace("\\", "/"); // Zip requires forward slashes

                        if path.is_file() {
                            if let Err(e) = zip.start_file(name_str, options) {
                                tracing::error!("Failed to start zip file {}: {}", path.display(), e);
                                let _ = tx.send(UiEvent::StatusMessage(format!("Export error: {}", e), eframe::egui::Color32::RED)).await;
                                continue;
                            }
                            if let Ok(mut f) = std::fs::File::open(path) {
                                let _ = std::io::copy(&mut f, &mut zip);
                            }
                        } else if path.is_dir() && !name.as_os_str().is_empty() {
                            let _ = zip.add_directory(name_str, options);
                        }
                    }
                }

                if let Err(e) = zip.finish() {
                    let _ = tx
                        .send(UiEvent::DbExportFinished(Err(format!(
                            "Zip Finish Failed: {}",
                            e
                        ))))
                        .await;
                } else {
                    let _ = tx
                        .send(UiEvent::DbExportFinished(Ok(zip_path
                            .to_string_lossy()
                            .to_string())))
                        .await;
                }
            }
            ctx.request_repaint();
            Ok(())
        }, self.tx.clone());
    }

    /// Imports database from file or ZIP backup
    pub fn trigger_db_import(&self) {
        let db = self.db.clone();
        let tx = self.tx.clone();
        let ctx = self.ctx.clone();
        crate::task::spawn_supervised(ctx.clone(), async move {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Import Data")
                .add_filter("All Supported", &["db", "sqlite", "sqlite3", "zip"])
                .add_filter("Database Files", &["db", "sqlite", "sqlite3"])
                .add_filter("Zip Backups", &["zip"])
                .pick_file()
            {
                // 1. Checkpoint current DB to ensure consistent state on disk
                if let Err(e) = db.checkpoint().await {
                    let _ = tx
                        .send(UiEvent::DbExportFinished(Err(format!(
                            "Pre-import checkpoint failed: {}",
                            e
                        ))))
                        .await;
                    return Ok(());
                }

                // 2. Close DB Connections using the existing async close
                db.close().await;

                // 3. Create Safety Backup
                let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                let backup_name = format!("crap_data_backup_{}.db", timestamp);
                if let Err(e) = std::fs::copy("crap_data.db", &backup_name) {
                    let _ = tx
                        .send(UiEvent::DbExportFinished(Err(format!(
                            "Auto-Backup Failed! Aborting import. Error: {}",
                            e
                        ))))
                        .await;
                    // Try to re-init
                    match crate::db::Database::init().await {
                        Ok(new_db) => {
                            let _ = tx.send(UiEvent::DbReloaded(Ok(new_db))).await;
                        }
                        Err(re_e) => {
                            let _ = tx.send(UiEvent::DbReloaded(Err(re_e.to_string()))).await;
                        }
                    }
                    return Ok(());
                }

                let import_path = path.as_path();
                let extension = import_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                let result = if extension == "zip" {
                    // ZIP IMPORT
                    match std::fs::File::open(path.clone()) {
                        Ok(file) => {
                            match zip::ZipArchive::new(file) {
                                Ok(mut archive) => {
                                    // Extract everything
                                    match archive.extract(".") {
                                        Ok(_) => Ok(()),
                                        Err(e) => Err(format!("Unzip failed: {}", e)),
                                    }
                                }
                                Err(e) => Err(format!("Invalid Zip: {}", e)),
                            }
                        }
                        Err(e) => Err(format!("Could not open zip: {}", e)),
                    }
                } else {
                    // DB IMPORT
                    match std::fs::copy(path.clone(), "crap_data.db") {
                        Ok(_) => Ok(()),
                        Err(e) => Err(format!("DB Copy Failed: {}", e)),
                    }
                };

                match result {
                    Ok(_) => {
                        // Re-init DB
                        match crate::db::Database::init().await {
                            Ok(new_db) => {
                                let _ = tx.send(UiEvent::DbReloaded(Ok(new_db))).await;
                            }
                            Err(re_e) => {
                                let _ = tx.send(UiEvent::DbReloaded(Err(re_e.to_string()))).await;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(UiEvent::DbExportFinished(Err(e))).await;
                        // Attempt re-init to restore state
                        match crate::db::Database::init().await {
                            Ok(new_db) => {
                                let _ = tx.send(UiEvent::DbReloaded(Ok(new_db))).await;
                            }
                            Err(re_e) => {
                                let _ = tx.send(UiEvent::DbReloaded(Err(re_e.to_string()))).await;
                            }
                        }
                    }
                }
            }
            ctx.request_repaint();
            Ok(())
        }, self.tx.clone());
    }

    /// Triggers the mass export of a collection (or All/Favorites)
    pub fn trigger_collection_export(
        &self,
        target: crate::ui::ExportTarget,
        format: crate::ui::ExportFormat,
    ) {
        // Clone data needed for export (Snapshot)
        let collections = self.collections.clone();
        let characters = self.characters.clone();
        let tx = self.tx.clone();

        let collection_name = match target {
            crate::ui::ExportTarget::Collection(id) => self
                .collections
                .iter()
                .find(|c| c.id == id)
                .map(|c| c.name.clone())
                .unwrap_or("Collection".to_string()),
            crate::ui::ExportTarget::All => "All_Characters".to_string(),
            crate::ui::ExportTarget::Favorites => "Favorites".to_string(),
        };

        tokio::task::spawn_blocking(move || {
            if let Some(target_dir) = rfd::FileDialog::new()
                .set_title(format!("Export '{}' to...", collection_name))
                .pick_folder()
            {
                tracing::info!("Starting collection export to {:?}", target_dir);
                let result = match target {
                    crate::ui::ExportTarget::Collection(id) => {
                        recursive_export_helper(
                            &collections,
                            &characters,
                            id,
                            &target_dir,
                            format,
                        )
                    }
                    crate::ui::ExportTarget::All => {
                        let chars_ref: Vec<&crate::models::Character> = characters.iter().collect();
                        export_flat_list(&chars_ref, &collection_name, &target_dir, format)
                    }
                    crate::ui::ExportTarget::Favorites => {
                        let chars_ref: Vec<&crate::models::Character> =
                            characters.iter().filter(|c| c.is_favorite).collect();
                        export_flat_list(&chars_ref, &collection_name, &target_dir, format)
                    }
                };
                
                match result {
                    Ok(_) => {
                        tracing::info!("Collection export completed successfully.");
                        let _ = tx.blocking_send(UiEvent::StatusMessage("Export successful!".to_string(), eframe::egui::Color32::GREEN));
                    }
                    Err(e) => {
                        tracing::error!("Collection export failed: {}", e);
                        let _ = tx.blocking_send(UiEvent::StatusMessage(format!("Export failed: {}", e), eframe::egui::Color32::RED));
                    }
                }
            } else {
                tracing::info!("Collection export canceled by user.");
            }
        });
    }

    pub fn trigger_advanced_export(
        &self,
        target: crate::ui::ExportTarget,
        settings: crate::ui::components::popups::AdvancedExportSettings,
    ) {
        let collections = self.collections.clone();
        let characters = self.characters.clone();
        let tx = self.tx.clone();

        let collection_name = match target {
            crate::ui::ExportTarget::Collection(id) => self
                .collections
                .iter()
                .find(|c| c.id == id)
                .map(|c| c.name.clone())
                .unwrap_or("Collection".to_string()),
            crate::ui::ExportTarget::All => "All_Characters".to_string(),
            crate::ui::ExportTarget::Favorites => "Favorites".to_string(),
        };

        tokio::task::spawn_blocking(move || {
            let suggested_name = format!(
                "{}_{}",
                sanitize_filename(&collection_name),
                match settings.format {
                    crate::ui::components::popups::AdvancedExportFormat::Grid => "grid.png",
                    crate::ui::components::popups::AdvancedExportFormat::List => "list.html",
                }
            );

            if let Some(path) = rfd::FileDialog::new()
                .set_title("Save Export As...")
                .set_file_name(suggested_name)
                .save_file()
            {
                tracing::info!("Starting advanced export to {:?}", path);
                // 1. Collect all characters
                let mut all_chars = Vec::new();
                match target {
                    crate::ui::ExportTarget::Collection(id) => {
                        collect_characters_recursively(
                            &collections,
                            &characters,
                            id,
                            &mut all_chars,
                        );
                    }
                    crate::ui::ExportTarget::All => {
                        all_chars = characters;
                    }
                    crate::ui::ExportTarget::Favorites => {
                        all_chars = characters.into_iter().filter(|c| c.is_favorite).collect();
                    }
                }

                // 2. Generate Output
                let result = match settings.format {
                    crate::ui::components::popups::AdvancedExportFormat::Grid => {
                        generate_collection_grid_png(&all_chars, &path, &settings)
                    }
                    crate::ui::components::popups::AdvancedExportFormat::List => {
                        generate_collection_list_html(&all_chars, &path, &settings)
                    }
                };
                
                match result {
                    Ok(_) => {
                        tracing::info!("Advanced export completed successfully.");
                        let _ = tx.blocking_send(UiEvent::StatusMessage("Export successful!".to_string(), eframe::egui::Color32::GREEN));
                    }
                    Err(e) => {
                        tracing::error!("Advanced export failed: {}", e);
                        let _ = tx.blocking_send(UiEvent::StatusMessage(format!("Export failed: {}", e), eframe::egui::Color32::RED));
                    }
                }
            } else {
                tracing::info!("Advanced export canceled by user.");
            }
        });
    }
}

fn export_flat_list(
    characters: &[&crate::models::Character],
    folder_name: &str,
    parent_dir: &std::path::Path,
    format: crate::ui::ExportFormat,
) -> Result<(), String> {
    let my_dir = parent_dir.join(sanitize_filename(folder_name));
    if !my_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&my_dir) {
            tracing::error!("Failed to create directory {:?}: {}", my_dir, e);
            return Err(e.to_string());
        }
    }

    for char in characters {
        let name_slug = sanitize_filename(&char.name);
        let file_name = match format {
            crate::ui::ExportFormat::Png => format!("{}.png", name_slug),
            crate::ui::ExportFormat::V2 => format!("{}.json", name_slug),
            crate::ui::ExportFormat::Native => format!("{}.crapp", name_slug),
            crate::ui::ExportFormat::Markdown => format!("{}.md", name_slug),
        };
        let target_path = my_dir.join(file_name);
        
        tracing::info!("Writing character {} to {:?}", char.name, target_path);

        let res = match format {
            crate::ui::ExportFormat::Png => CrapApp::write_character_png_static(char, &target_path),
            crate::ui::ExportFormat::V2 => {
                CrapApp::write_character_v2_json_static(char, &target_path)
            }
            crate::ui::ExportFormat::Native => {
                CrapApp::write_character_native_static(char, &target_path)
            }
            crate::ui::ExportFormat::Markdown => {
                CrapApp::write_character_markdown_static(char, &target_path)
            }
        };
        
        if let Err(e) = res {
            tracing::error!("Failed to write character {} to {:?}: {}", char.name, target_path, e);
            return Err(format!("Failed to write {}: {}", char.name, e));
        }
    }

    Ok(())
}

fn collect_characters_recursively(
    collections: &[crate::models::Collection],
    characters: &[crate::models::Character],
    collection_id: i64,
    acc: &mut Vec<crate::models::Character>,
) {
    // Add chars from this collection
    for c in characters
        .iter()
        .filter(|c| c.collection_id == Some(collection_id))
    {
        acc.push(c.clone());
    }

    // Recurse
    for sub in collections
        .iter()
        .filter(|c| c.parent_id == Some(collection_id))
    {
        collect_characters_recursively(collections, characters, sub.id, acc);
    }
}

fn generate_collection_grid_png(
    characters: &[crate::models::Character],
    path: &std::path::Path,
    settings: &crate::ui::components::popups::AdvancedExportSettings,
) -> Result<(), String> {
    if characters.is_empty() {
        return Err("No characters found".to_string());
    }

    let cols = settings.grid_columns as u32;
    let count = characters.len() as u32;
    let rows = (count as f32 / cols as f32).ceil() as u32;

    let tile_w = 300;
    let tile_h = 450;
    let margin = 20;

    let canvas_w = cols * (tile_w + margin) + margin;
    let canvas_h = rows * (tile_h + margin) + margin;

    let mut canvas = image::RgbaImage::new(canvas_w, canvas_h);

    // Fill background safely
    for p in canvas.pixels_mut() {
        *p = image::Rgba([30, 30, 30, 255]); // Dark grey background
    }

    // Attempt to load font
    let font_data = std::fs::read("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf")
        .or_else(|_| std::fs::read("/usr/share/fonts/TTF/DejaVuSans.ttf"))
        .or_else(|_| std::fs::read("/usr/share/fonts/liberation/LiberationSans-Regular.ttf"));

    let font = font_data
        .ok()
        .and_then(|data| rusttype::Font::try_from_vec(data));

    for (i, character) in characters.iter().enumerate() {
        let r = (i as u32) / cols;
        let c = (i as u32) % cols;

        let x = margin + c * (tile_w + margin);
        let y = margin + r * (tile_h + margin);

        // Load Avatar
        if let Some(avatar_path) = &character.avatar_path {
            if let Ok(img) = image::open(avatar_path) {
                // Resize
                let resized =
                    img.resize_to_fill(tile_w, tile_h, image::imageops::FilterType::Lanczos3);
                // Overlay
                image::imageops::overlay(&mut canvas, &resized, x as i64, y as i64);
            }
        }

        // Draw Name
        if settings.grid_show_names {
            // Draw text background with proper blending using overlay
            let name_h = 40;
            let name_y = y + tile_h - name_h;

            let mut bg_bar = image::RgbaImage::new(tile_w, name_h);
            for p in bg_bar.pixels_mut() {
                *p = image::Rgba([0, 0, 0, 180]);
            }
            image::imageops::overlay(&mut canvas, &bg_bar, x as i64, name_y as i64);

            if let Some(font) = &font {
                let scale = rusttype::Scale::uniform(24.0);
                imageproc::drawing::draw_text_mut(
                    &mut canvas,
                    image::Rgba([255, 255, 255, 255]),
                    x as i32 + 10,
                    name_y as i32 + 8,
                    scale,
                    font,
                    &character.char_name,
                );
            }
        }
    }

    canvas.save(path).map_err(|e| e.to_string())
}

fn generate_collection_list_html(
    characters: &[crate::models::Character],
    path: &std::path::Path,
    settings: &crate::ui::components::popups::AdvancedExportSettings,
) -> Result<(), String> {
    let mut html = String::from("<html><head><style>
        body { font-family: sans-serif; background: #1e1e1e; color: #ddd; padding: 20px; }
        .character { background: #2d2d2d; margin-bottom: 20px; padding: 15px; border-radius: 8px; display: flex; gap: 20px; }
        .avatar { width: 150px; height: 225px; object-fit: cover; border-radius: 4px; flex-shrink: 0; }
        .info { flex-grow: 1; }
        h2 { margin-top: 0; margin-bottom: 5px; color: #fff; }
        .subtitle { color: #aaa; font-style: italic; margin-bottom: 10px; }
        .tags span { background: #444; padding: 2px 6px; border-radius: 4px; font-size: 0.9em; margin-right: 5px; }
        .desc { white-space: pre-wrap; margin-top: 10px; color: #bbb; }
        .tokens { font-size: 0.8em; color: #888; margin-top: 5px; font-weight: bold; }
    </style></head><body>");

    html.push_str(&format!(
        "<h1>Exported Collection ({} characters)</h1>",
        characters.len()
    ));

    for char in characters {
        html.push_str("<div class='character'>");

        if settings.list_include_avatar {
            if let Some(avatar_path) = &char.avatar_path {
                if let Ok(bytes) = std::fs::read(avatar_path) {
                    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
                    html.push_str(&format!(
                        "<img class='avatar' src='data:image/png;base64,{}'>",
                        b64
                    ));
                } else {
                    html.push_str("<div class='avatar' style='background:#000'></div>");
                }
            } else {
                html.push_str("<div class='avatar' style='background:#000'></div>");
            }
        }

        html.push_str("<div class='info'>");

        if settings.list_include_name {
            html.push_str(&format!(
                "<h2>{}</h2>",
                html_escape::encode_text(&char.char_name)
            ));
        }

        if settings.list_include_tokens {
            // Calculate total tokens
            let mut total_text = String::new();
            total_text.push_str(&char.char_name);
            total_text.push_str(&char.char_title);
            total_text.push_str(&char.personality);
            total_text.push_str(&char.scenario);
            total_text.push_str(&char.first_message);
            total_text.push_str(&char.example_dialogue);
            let count = crate::models::count_tokens(&total_text);
            html.push_str(&format!("<div class='tokens'>Tokens: {}</div>", count));
        }

        if settings.list_include_tags {
            html.push_str("<div class='tags'>");
            for t in &char.app_tags {
                html.push_str(&format!(
                    "<span>{}</span>",
                    html_escape::encode_text(&t.name)
                ));
            }
            html.push_str("</div>");
        }

        if settings.list_include_description {
            // User requested Title/Description, not full personality/scenario dump
            if !char.char_title.is_empty() {
                html.push_str(&format!(
                    "<div class='subtitle'>{}</div>",
                    html_escape::encode_text(&char.char_title)
                ));
            }
            // maybe include author notes as "description" if title is short?
            // For now, adhere to "not personality/scenario".
        }

        html.push_str("</div></div>");
    }

    html.push_str("</body></html>");
    std::fs::write(path, html).map_err(|e| e.to_string())
}

fn recursive_export_helper(
    collections: &[crate::models::Collection],
    characters: &[crate::models::Character],
    collection_id: i64,
    parent_dir: &std::path::Path,
    format: crate::ui::ExportFormat,
) -> Result<(), String> {
    // 1. Get Collection Name and create dir
    let collection = collections
        .iter()
        .find(|c| c.id == collection_id)
        .ok_or("Collection not found")?;
    let sanitized_name = sanitize_filename(&collection.name);
    let my_dir = parent_dir.join(sanitized_name);

    if !my_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&my_dir) {
            tracing::error!("Failed to create directory {:?}: {}", my_dir, e);
            return Err(e.to_string());
        }
    }

    // 2. Export Characters in this collection
    let chars_in_col: Vec<&crate::models::Character> = characters
        .iter()
        .filter(|c| c.collection_id == Some(collection_id))
        .collect();

    for char in chars_in_col {
        // inline export_character_in_format logic to avoid referencing CrapApp instance
        let name_slug = sanitize_filename(&char.name);
        let file_name = match format {
            crate::ui::ExportFormat::Png => format!("{}.png", name_slug),
            crate::ui::ExportFormat::V2 => format!("{}.json", name_slug),
            crate::ui::ExportFormat::Native => format!("{}.crapp", name_slug),
            crate::ui::ExportFormat::Markdown => format!("{}.md", name_slug),
        };
        let target_path = my_dir.join(file_name);
        
        tracing::info!("Writing character {} to {:?}", char.name, target_path);

        let res = match format {
            crate::ui::ExportFormat::Png => CrapApp::write_character_png_static(char, &target_path),
            crate::ui::ExportFormat::V2 => {
                CrapApp::write_character_v2_json_static(char, &target_path)
            }
            crate::ui::ExportFormat::Native => {
                CrapApp::write_character_native_static(char, &target_path)
            }
            crate::ui::ExportFormat::Markdown => {
                CrapApp::write_character_markdown_static(char, &target_path)
            }
        };
        
        if let Err(e) = res {
            tracing::error!("Failed to write character {} to {:?}: {}", char.name, target_path, e);
            return Err(format!("Failed to write {}: {}", char.name, e));
        }
    }

    // 3. Recurse for sub-collections
    let sub_cols: Vec<i64> = collections
        .iter()
        .filter(|c| c.parent_id == Some(collection_id))
        .map(|c| c.id)
        .collect();
    for sub_id in sub_cols {
        if let Err(e) = recursive_export_helper(collections, characters, sub_id, &my_dir, format) {
            tracing::error!("Failed to export sub-collection: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

fn sanitize_filename(name: &str) -> String {
    // Replace reserved characters and control characters with '_'
    let mut s: String = name.chars().map(|c| {
        if ['/', '\\', ':', '*', '?', '"', '<', '>', '|'].contains(&c) || c.is_control() {
            '_'
        } else {
            c
        }
    }).collect();

    // Windows drops trailing spaces and dots in directory and file names.
    // If we keep them in our PathBuf, subsequent operations will fail with "path not found".
    s = s.trim_end_matches(|c| c == ' ' || c == '.').to_string();
    
    // Trim leading whitespace as well just to be tidy
    s = s.trim_start().to_string();

    let upper = s.to_uppercase();
    let reserved = [
        "CON", "PRN", "AUX", "NUL",
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    if reserved.contains(&upper.as_str()) {
        s.push('_');
    }

    if s.is_empty() {
        s = "Unnamed".to_string();
    }
    
    s
}
