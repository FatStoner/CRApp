use crate::db::Database;
use eframe::egui;

use crate::models::{Character, SearchResultKind, Template};

use std::time::{Duration, Instant};

pub mod browser;
pub mod central_panel;
pub mod editor;
pub mod global_search;
pub mod popups;
pub mod side_panel;
pub mod text_highlight;
pub mod widgets;

pub use global_search::{CharacterSearchFieldFilters, LorebookSearchFieldFilters};
pub use popups::PopupState;

pub mod options_window;
pub mod spell_check;
pub mod spell_layout;

pub mod parsing;

// Re-export specific items if needed
pub use parsing::ParsedCharacterData;

pub mod types;
pub use types::*;
pub mod app;
pub use app::CrapApp;

impl eframe::App for CrapApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.pointer.button_pressed(egui::PointerButton::Extra1)) {
            self.request_back();
        }

        // Smoother UI Scaling with debouncing
        let (scroll_y, ctrl) = ctx.input(|i| (i.raw_scroll_delta.y, i.modifiers.ctrl));
        if ctrl
            && scroll_y.abs() > 0.01
            && self.last_scroll_time.elapsed() > Duration::from_millis(50)
        {
            self.last_scroll_time = Instant::now();
            let step = 0.05;
            let mut new_scale = self.ui_scale;
            if scroll_y > 0.0 {
                new_scale += step;
            } else {
                new_scale -= step;
            }

            new_scale = (new_scale * 20.0).round() / 20.0; // Snap to 5%
            new_scale = new_scale.clamp(0.5, 2.0);

            if (new_scale - self.ui_scale).abs() > 0.001 {
                self.ui_scale = new_scale;
                self.ctx.set_pixels_per_point(new_scale);
                self.scale_last_updated = Some(Instant::now());
                self.ctx.request_repaint(); // Snappy refresh

                let pct = (new_scale * 100.0).round() as i32;
                self.set_status(
                    format!("UI Scale: {}%", pct),
                    egui::Color32::from_rgb(100, 200, 255),
                );
            }
        }

        // Search Focus Shortcut
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::F)) {
            self.focus_search_field = true;
        }

        // Debounced Save
        if let Some(last_time) = self.scale_last_updated {
            if last_time.elapsed() > Duration::from_millis(1000) {
                self.scale_last_updated = None;
                self.set_scale(self.ui_scale); // Triggers DB save
            }
        }

        // Event Loop
        let mut received_event = false;
        while let Ok(event) = self.rx.try_recv() {
            received_event = true;
            match event {
                UiEvent::CharactersLoaded(res) => match res {
                    Ok(list) => {
                        self.characters = list;
                        self.loading_error = None;
                    }
                    Err(e) => {
                        eprintln!("Load error: {}", e);
                        self.loading_error = Some(e);
                    }
                },
                UiEvent::LorebooksLoaded(res) => match res {
                    Ok(books) => self.lorebooks = books,
                    Err(e) => {
                        self.loading_error = Some(e);
                    }
                },
                UiEvent::CollectionsLoaded(res) => match res {
                    Ok(collections) => self.collections = collections,
                    Err(e) => {
                        self.loading_error = Some(e);
                    }
                },
                UiEvent::ThemeLoaded(res) => {
                    if let Ok(mode) = res {
                        self.theme = mode;
                        self.apply_theme();
                    }
                }
                UiEvent::ScaleLoaded(res) => {
                    if let Ok(scale) = res {
                        self.ui_scale = scale;
                        self.ctx.set_pixels_per_point(scale);
                    }
                }
                UiEvent::LoreLinksLoaded(res) => match res {
                    Ok(set) => self.lore_links = set,
                    Err(e) => eprintln!("Link load error: {}", e),
                },
                UiEvent::LoreLinksBulkLoaded(map) => {
                    self.char_lore_map = map;
                }
                UiEvent::CharacterSaved(res) => {
                    self.is_saving = false;
                    match res {
                        Ok(c) => {
                            // Ensure links and tags are loaded (critical for new characters)
                            self.load_links(c.id);
                            self.load_tags(c.id);

                            self.selected_character = Some(c);
                            self.set_status("Character Saved!".to_string(), egui::Color32::GREEN);

                            // Handle pending action if any
                            if let Some(action) = self.pending_action.take() {
                                self.perform_action(action, &ctx);
                            }
                        }
                        Err(e) => {
                            self.set_status(format!("Save Error: {}", e), egui::Color32::RED);
                            self.pending_action = None;
                        }
                    }
                }
                UiEvent::LorebookSaved(res) => {
                    self.is_saving = false;
                    match res {
                        Ok(l) => {
                            self.selected_lorebook = Some(l);
                            self.set_status("Lorebook Saved!".to_string(), egui::Color32::GREEN);

                            // Handle pending action if any (Fix for Save & Continue)
                            if let Some(action) = self.pending_action.take() {
                                self.perform_action(action, &ctx);
                            }
                        }
                        Err(e) => {
                            self.set_status(format!("Save Error: {}", e), egui::Color32::RED);
                            self.pending_action = None;
                        }
                    }
                }
                UiEvent::CollectionSaved(res) => {
                    self.is_saving = false;
                    match res {
                        Ok(_) => {
                            self.set_status("Collection Saved!".to_string(), egui::Color32::GREEN);
                            self.reload_collections();
                            self.popup_state = PopupState::None;
                        }
                        Err(e) => self.set_status(format!("Save Error: {}", e), egui::Color32::RED),
                    }
                }
                UiEvent::CollectionDeleted(res) => {
                    self.is_saving = false;
                    match res {
                        Ok(id) => {
                            self.set_status("Collection Deleted".to_string(), egui::Color32::GREEN);
                            // Optimistic update
                            self.collections.retain(|c| c.id != id);
                            self.reload_collections();
                            self.reload_characters();
                            if self.selected_collection_id == Some(id) {
                                self.selected_collection_id = None;
                            }
                        }
                        Err(e) => {
                            self.set_status(format!("Delete Error: {}", e), egui::Color32::RED)
                        }
                    }
                }
                UiEvent::LinkUpdated(res) => {
                    if let Err(e) = res {
                        self.set_status(format!("Link Error: {}", e), egui::Color32::RED);
                    }
                }
                UiEvent::TagsLoaded(res) => match res {
                    Ok((id, app, ext)) => {
                        if let Some(c) = &mut self.selected_character {
                            if c.id == id {
                                c.app_tags = app;
                                c.external_tags = ext;
                            }
                        }
                    }
                    Err(e) => self.set_status(format!("Tag Load Error: {}", e), egui::Color32::RED),
                },
                UiEvent::TagOperationFinished(res) => match res {
                    Ok(_) => {
                        if let Some(c) = &self.selected_character {
                            self.load_tags(c.id);
                        }
                        self.refresh_all();
                    }
                    Err(e) => self.set_status(format!("Tag Error: {}", e), egui::Color32::RED),
                },
                UiEvent::LorebookTagsLoaded(res) => match res {
                    Ok((id, tags)) => {
                        // Update selected if matches
                        if let Some(l) = &mut self.selected_lorebook {
                            if l.id == id {
                                l.tags = tags.clone();
                            }
                        }
                        // Update cache
                        if let Some(cached) = self.lorebooks.iter_mut().find(|b| b.id == id) {
                            cached.tags = tags;
                        }
                    }
                    Err(e) => eprintln!("Lorebook tags load error: {}", e),
                },
                UiEvent::LorebookTagOperationFinished(res) => {
                    if let Err(e) = res {
                        self.set_status(format!("Tag Error: {}", e), egui::Color32::RED);
                    }
                }
                UiEvent::LorebookEntriesLoaded(res) => match res {
                    Ok((lid, entries)) => {
                        // Update cache
                        if let Some(l) = self.lorebooks.iter_mut().find(|l| l.id == lid) {
                            l.entries = entries.clone();
                        }
                        // Update selected
                        if let Some(l) = &mut self.selected_lorebook {
                            if l.id == lid {
                                l.entries = entries;
                            }
                        }
                    }
                    Err(e) => self
                        .set_status(format!("Failed to load entries: {}", e), egui::Color32::RED),
                },
                UiEvent::LorebookEntryAdded(res) => match res {
                    Ok(_) => self.set_status("Entry added".to_string(), egui::Color32::GREEN),
                    Err(e) => {
                        self.set_status(format!("Failed to add entry: {}", e), egui::Color32::RED)
                    }
                },
                UiEvent::LorebookEntrySaved(res) => match res {
                    Ok(_) => {} // Silent save
                    Err(e) => {
                        self.set_status(format!("Failed to save entry: {}", e), egui::Color32::RED)
                    }
                },
                UiEvent::LorebookEntryDeleted(res) => match res {
                    Ok(_) => self.set_status("Entry deleted".to_string(), egui::Color32::GREEN),
                    Err(e) => self
                        .set_status(format!("Failed to delete entry: {}", e), egui::Color32::RED),
                },
                UiEvent::LorebookDeleted(res) => match res {
                    Ok(id) => {
                        self.set_status("Lorebook Deleted".to_string(), egui::Color32::GREEN);
                        self.lorebooks.retain(|b| b.id != id);
                        if let Some(selected) = &self.selected_lorebook {
                            if selected.id == id {
                                self.selected_lorebook = None;
                            }
                        }
                    }
                    Err(e) => self.set_status(format!("Delete Error: {}", e), egui::Color32::RED),
                },
                UiEvent::TemplatesLoaded(res) => match res {
                    Ok(list) => {
                        self.templates = list;
                    }
                    Err(e) => {
                        eprintln!("Load error: {}", e);
                        self.loading_error = Some(e);
                    }
                },
                UiEvent::TemplateSaved(res) => {
                    self.is_saving = false;
                    match res {
                        Ok(t) => {
                            self.selected_template = Some(t);
                            self.set_status("Template Saved!".to_string(), egui::Color32::GREEN);
                        }
                        Err(e) => {
                            self.set_status(format!("Save Error: {}", e), egui::Color32::RED);
                        }
                    }
                }
                UiEvent::TemplateDeleted(res) => {
                    self.is_saving = false;
                    match res {
                        Ok(id) => {
                            self.set_status("Template Deleted".to_string(), egui::Color32::GREEN);
                            self.templates.retain(|t| t.id != id);
                            if let Some(selected) = &self.selected_template {
                                if selected.id == id {
                                    self.selected_template = None;
                                }
                            }
                        }
                        Err(e) => {
                            self.set_status(format!("Delete Error: {}", e), egui::Color32::RED);
                        }
                    }
                }
                UiEvent::DeepSearchCompleted(res) => {
                    self.is_deep_searching = false;
                    match res {
                        Ok(results) => self.deep_search_results = results,
                        Err(e) => {
                            self.set_status(format!("Search failed: {}", e), egui::Color32::RED)
                        }
                    }
                }
                UiEvent::UiRepaint => {
                    // Just wakes the loop, nothing to do
                }
                UiEvent::CharacterDeleted(res) => {
                    match res {
                        Ok(id) => {
                            // Optimistic update
                            self.characters.retain(|c| c.id != id);
                            if let Some(selected) = &self.selected_character {
                                if selected.id == id {
                                    self.selected_character = None;
                                    self.central_view = CentralView::Browser;
                                }
                            }
                            self.set_status("Character Deleted".to_string(), egui::Color32::GREEN);
                        }
                        Err(e) => {
                            self.set_status(format!("Delete Error: {}", e), egui::Color32::RED)
                        }
                    }
                }
                UiEvent::CharacterMoved(res) => {
                    match res {
                        Ok((char_id, new_coll_id)) => {
                            self.set_status("Character Moved".to_string(), egui::Color32::GREEN);

                            // 1. Sync Selected Character (Fix for editor desync)
                            if let Some(selected) = &mut self.selected_character {
                                if selected.id == char_id {
                                    selected.collection_id = new_coll_id;
                                }
                            }

                            // 2. Optimistic List Update
                            if let Some(c) = self.characters.iter_mut().find(|c| c.id == char_id) {
                                c.collection_id = new_coll_id;
                            }

                            self.reload_characters();
                        }
                        Err(e) => self.set_status(format!("Move Error: {}", e), egui::Color32::RED),
                    }
                }
                UiEvent::ImportFileLoaded(res) => {
                    match res {
                        Ok(json_content) => {
                            if let Ok(mut char_obj) =
                                serde_json::from_str::<Character>(&json_content)
                            {
                                // Clean ID for new import
                                char_obj.id = 0;

                                // Map to ParsedCharacterData for review
                                let parsed = ParsedCharacterData {
                                    name: char_obj.name.clone(),
                                    title: char_obj.char_title.clone(),
                                    personality: char_obj.personality.clone(),
                                    scenario: char_obj.scenario.clone(),
                                    first_message: char_obj.first_message.clone(),
                                    example_dialogue: char_obj.example_dialogue.clone(),
                                    external_tags: char_obj
                                        .external_tags
                                        .iter()
                                        .map(|t| t.name.clone())
                                        .collect(),
                                    app_tags: char_obj
                                        .app_tags
                                        .iter()
                                        .map(|t| t.name.clone())
                                        .collect(),
                                    urls: char_obj.urls.clone(),
                                };

                                // Force "New Character" mode
                                self.selected_character = Some(Character::default());
                                self.mode = AppMode::Characters;

                                self.parsed_data = Some(parsed);
                                self.show_import_modal = true;
                                self.import_text.clear(); // Clear clipboard text if any

                                self.set_status_with_duration(
                                    "File loaded for review.".to_string(),
                                    egui::Color32::GREEN,
                                    Duration::from_secs(10),
                                );
                            } else {
                                self.set_status(
                                    "Failed to parse file structure.".to_string(),
                                    egui::Color32::RED,
                                );
                            }
                        }
                        Err(e) => self.set_status(format!("Read Error: {}", e), egui::Color32::RED),
                    }
                }
                UiEvent::DbExportFinished(res) => match res {
                    Ok(path) => self.set_status(
                        format!("Database exported to: {}", path),
                        egui::Color32::GREEN,
                    ),
                    Err(e) => self.set_status(format!("Export Failed: {}", e), egui::Color32::RED),
                },
                UiEvent::DbReloaded(res) => match res {
                    Ok(new_db) => {
                        self.db = new_db;
                        self.set_status(
                            "Database imported successfully. Reloading view...".to_string(),
                            egui::Color32::GREEN,
                        );
                        self.refresh_all();
                    }
                    Err(e) => {
                        self.set_status(
                            format!("CRITICAL: Database Swap Failed: {}", e),
                            egui::Color32::RED,
                        );
                    }
                },

                UiEvent::TokenCountCalculated(id, tokens, chars) => {
                    self.token_cache.insert(id, (tokens, chars));
                    self.token_calc_in_progress.remove(&id);
                }
                UiEvent::LorebookImported(lb) => {
                    self.set_status(
                        "Lorebook Imported Successfully".to_string(),
                        egui::Color32::GREEN,
                    );
                    self.popup_state = PopupState::None;
                    self.reload_lorebooks();
                    self.selected_lorebook = Some(lb);
                    self.selected_character = None;
                    self.mode = AppMode::Lorebooks;
                    self.active_lorebook_tab = LorebookTab::Entries;
                }
                UiEvent::StatusMessage(msg, color) => {
                    self.set_status(msg, color);
                }
            }
        }

        if received_event {
            ctx.request_repaint();
        }

        // Timer
        if let Some(deadline) = self.status_clear_time {
            if Instant::now() > deadline {
                self.status_message = None;
                self.status_clear_time = None;
            } else {
                ctx.request_repaint();
            }
        }

        // Handle Close Request
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.has_unsaved_changes() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.popup_state = PopupState::UnsavedChanges {
                    target: AppAction::Exit,
                };
            }
        }

        // Side Panel
        side_panel::render_side_panel(self, ctx);

        // Central Panel
        central_panel::render_central_panel(self, ctx);

        // Global Popups
        popups::render_popups(ctx, self);

        // LIGHTBOX OVERLAY
        if let Some(uri) = &self.fullscreen_image {
            let mut close = false;
            let mut next_image = None;

            egui::Area::new("lightbox_overlay".into())
                .order(egui::Order::Foreground)
                .fixed_pos(egui::pos2(0.0, 0.0))
                .interactable(true)
                .show(ctx, |ui| {
                    let screen_rect = ctx.screen_rect();

                    // 1. Dimmed Background
                    let (rect, response) =
                        ui.allocate_exact_size(screen_rect.size(), egui::Sense::click());
                    ui.painter()
                        .rect_filled(rect, 0.0, egui::Color32::from_black_alpha(200));

                    if response.clicked() {
                        close = true;
                    }

                    // 2. Centered Image
                    let max_size = screen_rect.size() * 0.9;
                    let img = egui::Image::new(uri).shrink_to_fit().max_size(max_size);

                    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(screen_rect), |ui| {
                        ui.centered_and_justified(|ui| {
                            ui.add(img);
                        });
                    });
                });

            // Handle Input
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                close = true;
            }

            if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                if let Some(context) = &self.gallery_context {
                    if let Some(idx) = context.iter().position(|u| u == uri) {
                        let new_idx = (idx + 1) % context.len();
                        next_image = Some(context[new_idx].clone());
                    }
                }
            }

            if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                if let Some(context) = &self.gallery_context {
                    if let Some(idx) = context.iter().position(|u| u == uri) {
                        let new_idx = if idx == 0 { context.len() - 1 } else { idx - 1 };
                        next_image = Some(context[new_idx].clone());
                    }
                }
            }

            if close {
                self.fullscreen_image = None;
                self.gallery_context = None;
            }

            if let Some(next) = next_image {
                self.fullscreen_image = Some(next);
            }
        }

        // Watermark: The Library of Snailexandria
        if ctx.screen_rect().width() > 300.0 {
            egui::Area::new("watermark_area".into())
                .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-10.0, -5.0))
                .order(egui::Order::Foreground)
                .interactable(false)
                .show(ctx, |ui| {
                    ui.style_mut().interaction.selectable_labels = false;
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("The Library of Snailexandria")
                                .size(11.0)
                                .color(egui::Color32::from_white_alpha(100))
                                .italics(),
                        );
                    });
                });
        }
    }
}

impl CrapApp {
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
                    match Database::init().await {
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
                        match Database::init().await {
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
                        match Database::init().await {
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
    pub fn create_new_template(&mut self) {
        if self.has_unsaved_changes() {
            self.popup_state = PopupState::UnsavedChanges {
                target: AppAction::CreateNewTemplate,
            };
        } else {
            self.perform_create_new_template();
        }
    }

    pub fn perform_create_new_template(&mut self) {
        self.push_history();
        let new_template = Template::default();
        self.selected_template = Some(new_template.clone());
        self.save_template(new_template);
        self.mode = AppMode::Templates;
        self.selected_character = None;
        self.selected_lorebook = None;
    }

    pub fn save_template(&mut self, mut template: Template) {
        self.is_saving = true;
        self.status_message = None;
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            if let Err(e) = db.upsert_template(&mut template).await {
                let _ = tx.send(UiEvent::TemplateSaved(Err(e.to_string()))).await;
                ctx.request_repaint();
            } else {
                let _ = tx.send(UiEvent::TemplateSaved(Ok(template))).await;
                ctx.request_repaint();
                // Reload
                match db.get_all_templates().await {
                    Ok(templates) => {
                        let _ = tx.send(UiEvent::TemplatesLoaded(Ok(templates))).await;
                    }
                    Err(e) => {
                        let _ = tx.send(UiEvent::TemplatesLoaded(Err(e.to_string()))).await;
                    }
                }
                ctx.request_repaint();
            }
        });
    }

    pub fn delete_template(&self, id: i64) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let res = db.delete_template(id).await;
            let _ = tx
                .send(UiEvent::TemplateDeleted(
                    res.map(|_| id).map_err(|e| e.to_string()),
                ))
                .await;
            ctx.request_repaint();
        });
    }

    pub fn request_template_switch(&mut self, id: i64) {
        if self.has_unsaved_changes() {
            self.popup_state = PopupState::UnsavedChanges {
                target: AppAction::SwitchTemplate(id),
            };
        } else {
            self.perform_template_switch(id);
        }
    }

    pub fn perform_template_switch(&mut self, id: i64) {
        self.push_history();
        if let Some(t) = self.templates.iter().find(|t| t.id == id).cloned() {
            self.selected_template = Some(t);
            self.mode = AppMode::Templates;
            self.selected_character = None;
            self.selected_lorebook = None;
        }
    }
}

pub fn cleanup_avatar(path_str: &str) {
    let path = std::path::Path::new(path_str);
    // Security check: Only delete if inside "data/avatars"
    // Normalize logic loosely by checking components or starts_with
    if path_str.replace("\\", "/").contains("data/avatars/") {
        if path.exists() {
            if let Err(e) = std::fs::remove_file(path) {
                eprintln!("Failed to delete old avatar {}: {}", path_str, e);
            } else {
                println!("Deleted old avatar: {}", path_str);
            }
        }
    }
}

static CURRENT_DIR: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

pub fn get_image_uri(path: &str) -> String {
    if path.starts_with("file://") || path.contains("://") {
        return path.to_string();
    }

    // Check if absolute
    let path_obj = std::path::Path::new(path);
    if path_obj.is_absolute() {
        return format!("file://{}", path);
    }

    // Relative path: Resolve against cached current dir
    let cwd = CURRENT_DIR
        .get_or_init(|| std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")));

    let abs = cwd.join(path);
    format!("file://{}", abs.to_string_lossy())
}
