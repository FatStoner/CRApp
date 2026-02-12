use super::state::CrapApp;
use crate::ui::types::UiEvent;

impl CrapApp {
    /// Exports database to a file (DB only, no data folder)
    pub fn trigger_db_export_file_only(&self) {
        let db = self.db.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
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
        });
    }

    /// Exports full backup as ZIP (database + data folder)
    pub fn perform_full_zip_export(&self) {
        let db = self.db.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
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
                    return;
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
                        return;
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
                    return;
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
                        return;
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
                                eprintln!("Failed to start zip file {}: {}", path.display(), e);
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
        });
    }

    /// Imports database from file or ZIP backup
    pub fn trigger_db_import(&self) {
        let db = self.db.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
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
                    return;
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
                    return;
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
        });
    }

    /// Triggers the mass export of a collection
    pub fn trigger_collection_export(&self, collection_id: i64, format: crate::ui::ExportFormat) {
        // Clone data needed for export (Snapshot)
        let collections = self.collections.clone();
        let characters = self.characters.clone();

        let collection_name = self
            .collections
            .iter()
            .find(|c| c.id == collection_id)
            .map(|c| c.name.clone())
            .unwrap_or("Collection".to_string());

        tokio::task::spawn_blocking(move || {
            if let Some(target_dir) = rfd::FileDialog::new()
                .set_title(format!("Export '{}' to...", collection_name))
                .pick_folder()
            {
                let _ = recursive_export_helper(
                    &collections,
                    &characters,
                    collection_id,
                    &target_dir,
                    format,
                );
            }
        });
    }
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
    let sanitized_name = collection.name.replace("/", "_").replace("\\", "_");
    let my_dir = parent_dir.join(sanitized_name);

    if !my_dir.exists() {
        std::fs::create_dir_all(&my_dir).map_err(|e| e.to_string())?;
    }

    // 2. Export Characters in this collection
    let chars_in_col: Vec<&crate::models::Character> = characters
        .iter()
        .filter(|c| c.collection_id == Some(collection_id))
        .collect();

    for char in chars_in_col {
        // inline export_character_in_format logic to avoid referencing CrapApp instance
        let name_slug = char.name.replace(" ", "_");
        let file_name = match format {
            crate::ui::ExportFormat::Png => format!("{}.png", name_slug),
            crate::ui::ExportFormat::V2 => format!("{}.json", name_slug),
            crate::ui::ExportFormat::Native => format!("{}.crapp", name_slug),
            crate::ui::ExportFormat::Markdown => format!("{}.md", name_slug),
        };
        let target_path = my_dir.join(file_name);

        let _ = match format {
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
    }

    // 3. Recurse for sub-collections
    let sub_cols: Vec<i64> = collections
        .iter()
        .filter(|c| c.parent_id == Some(collection_id))
        .map(|c| c.id)
        .collect();
    for sub_id in sub_cols {
        let _ = recursive_export_helper(collections, characters, sub_id, &my_dir, format);
    }

    Ok(())
}
