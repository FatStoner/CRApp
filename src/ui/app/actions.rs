use super::state::CrapApp;
use crate::models::{Character, Lorebook, Tag};
use crate::ui::types::{AppAction, AppMode, CentralView, UiEvent};
use crate::ui::utils::cleanup_avatar;
use crate::ui::PopupState;
use eframe::egui;
use std::collections::{HashMap, HashSet};

impl CrapApp {
    pub fn delete_collection(&self, id: i64) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let _ctx = self.ctx.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let res = db.delete_collection(id).await;
            let _ = tx
                .send(UiEvent::CollectionDeleted(
                    res.map(|_| id).map_err(|e| e.to_string()),
                ))
                .await;
            ctx.request_repaint();
        });
    }

    pub fn update_collection_icon(&self, id: i64, path: Option<String>) {
        if let Some(col) = self.collections.iter().find(|c| c.id == id).cloned() {
            let mut new_col = col.clone();
            new_col.image_path = path;

            let tx = self.tx.clone();
            let db = self.db.clone();
            tokio::spawn(async move {
                let _ = db.upsert_collection(&new_col).await;
                // We reuse CollectionSaved event to trigger reload
                let _ = tx.send(UiEvent::CollectionSaved(Ok(id))).await;
            });
        }
    }

    pub fn save_character(&mut self, mut character: Character) {
        self.is_saving = true;
        self.status_message = None;

        // Check for avatar change to cleanup old file
        let mut old_avatar_to_delete: Option<String> = None;
        if character.id != 0 {
            if let Some(old) = self.characters.iter().find(|c| c.id == character.id) {
                if old.avatar_path != character.avatar_path {
                    old_avatar_to_delete = old.avatar_path.clone();
                }
            }
        }

        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let is_new = character.id == 0;
            if let Err(e) = db.upsert_character(&mut character).await {
                let _ = tx.send(UiEvent::CharacterSaved(Err(e.to_string()))).await;
                // FEEDBACK RESTORED
                let _ = tx
                    .send(UiEvent::StatusMessage(
                        format!("Save Failed: {}", e),
                        egui::Color32::RED,
                    ))
                    .await;
                ctx.request_repaint();
            } else {
                // Sync Tags (For both New and Existing)
                // We wipe and re-insert to match the editor state exactly (Overwrite behavior)
                // This handles deletions logic implicitly.
                let cid = character.id;

                // 1. External Tags
                let _ = db.remove_all_tags_from_character(cid, true).await;
                for tag in &character.external_tags {
                    let _ = db.add_tag_to_character(cid, &tag.name, true).await;
                }

                // 2. App Tags
                let _ = db.remove_all_tags_from_character(cid, false).await;
                for tag in &character.app_tags {
                    let _ = db.add_tag_to_character(cid, &tag.name, false).await;
                }

                if !is_new {
                    // Cleanup old avatar if changed
                    if let Some(path) = old_avatar_to_delete {
                        cleanup_avatar(&path);
                    }
                }

                // Reload tags to ensure we have correct database IDs (otherwise dirty check fails)
                // UPSERT handles URLs, but we manually handled Tags above, so we must reload them to get IDs.
                if let Ok(saved_app_tags) = db.get_tags_for_character(cid, false).await {
                    character.app_tags = saved_app_tags;
                }
                if let Ok(saved_ext_tags) = db.get_tags_for_character(cid, true).await {
                    character.external_tags = saved_ext_tags;
                }

                let _ = tx.send(UiEvent::CharacterSaved(Ok(character))).await;
                // FEEDBACK RESTORED
                let _ = tx
                    .send(UiEvent::StatusMessage(
                        "Character Saved!".to_string(),
                        egui::Color32::GREEN,
                    ))
                    .await;
                ctx.request_repaint();
                let mut chars = db.get_all_characters().await.map_err(|e| e.to_string());
                if let Ok(ref mut characters) = chars {
                    let app_tags_res = db.get_all_tags_flat(false).await;
                    let ext_tags_res = db.get_all_tags_flat(true).await;
                    let urls_res = db.get_all_character_urls_flat().await;

                    if let (Ok(app_flat), Ok(ext_flat), Ok(urls_flat)) =
                        (app_tags_res, ext_tags_res, urls_res)
                    {
                        let mut app_map: HashMap<i64, Vec<Tag>> = HashMap::new();
                        for (cid, tag) in app_flat {
                            app_map.entry(cid).or_default().push(tag);
                        }

                        let mut ext_map: HashMap<i64, Vec<crate::models::Tag>> = HashMap::new();
                        for (cid, tag) in ext_flat {
                            ext_map.entry(cid).or_default().push(tag);
                        }

                        let mut url_map: HashMap<i64, Vec<crate::models::CharacterUrl>> =
                            HashMap::new();
                        for url in urls_flat {
                            url_map.entry(url.character_id).or_default().push(url);
                        }

                        for c in characters {
                            if let Some(tags) = app_map.remove(&c.id) {
                                c.app_tags = tags;
                            }
                            if let Some(tags) = ext_map.remove(&c.id) {
                                c.external_tags = tags;
                            }
                            if let Some(urls) = url_map.remove(&c.id) {
                                c.urls = urls;
                            }
                        }
                    }
                }

                let _ = tx.send(UiEvent::CharactersLoaded(chars)).await;
                ctx.request_repaint();
            }
        });
    }

    pub fn create_new_lorebook(&mut self) {
        if self.has_unsaved_changes() {
            self.popup_state = PopupState::UnsavedChanges {
                target: AppAction::CreateNewLorebook,
            };
        } else {
            self.perform_create_new_lorebook();
        }
    }

    pub fn perform_create_new_lorebook(&mut self) {
        self.push_history();
        let new_book = Lorebook::default();
        // Optimistic update so UI shows it immediately
        self.selected_lorebook = Some(new_book.clone());
        self.save_lorebook(new_book);
        self.mode = AppMode::Lorebooks;
        self.selected_character = None;
    }

    pub fn save_lorebook(&mut self, mut lorebook: Lorebook) {
        self.is_saving = true;
        self.status_message = None;
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();

        tokio::spawn(async move {
            if let Err(e) = db.upsert_lorebook(&mut lorebook).await {
                let _ = tx.send(UiEvent::LorebookSaved(Err(e.to_string()))).await;
                ctx.request_repaint();
            } else {
                let lid = lorebook.id;

                // 1. Sync Tags
                // Simple wipe and replace strategy for correctness with editor state
                // This assumes `lorebook.tags` is the source of truth
                let get_tags_res = db.get_tags_for_lorebook(lid).await;
                if let Ok(existing_tags) = get_tags_res {
                    for t in existing_tags {
                        let _ = db.remove_tag_from_lorebook(lid, t.id).await;
                    }
                }
                for tag in &lorebook.tags {
                    let _ = db.add_tag_to_lorebook(lid, &tag.name).await;
                }

                // 2. Sync Entries
                // We need to handle:
                // - Updates (existing ID)
                // - Inserts (ID 0)
                // - Deletions (ID exists in DB but not in `lorebook.entries`)
                let get_entries_res = db.get_entries_for_lorebook(lid).await;
                if let Ok(existing_entries) = get_entries_res {
                    let current_ids: HashSet<i64> = lorebook
                        .entries
                        .iter()
                        .filter(|e| e.id != 0)
                        .map(|e| e.id)
                        .collect();

                    for existing in existing_entries {
                        if !current_ids.contains(&existing.id) {
                            let _ = db.delete_lorebook_entry(existing.id).await;
                        }
                    }
                }

                let mut updated_entries = Vec::new();
                for entry in &mut lorebook.entries {
                    entry.lorebook_id = lid; // Ensure consistency
                    match if entry.id == 0 {
                        db.add_entry_to_lorebook(entry).await
                    } else {
                        db.update_lorebook_entry(entry).await.map(|_| entry.id)
                    } {
                        Ok(new_id) => {
                            entry.id = new_id;
                            updated_entries.push(entry.clone());
                        }
                        Err(e) => eprintln!("Failed to save entry: {}", e),
                    }
                }
                lorebook.entries = updated_entries;

                // Reload tags
                if let Ok(tags) = db.get_tags_for_lorebook(lid).await {
                    lorebook.tags = tags;
                }

                let _ = tx.send(UiEvent::LorebookSaved(Ok(lorebook))).await;
                ctx.request_repaint();

                // Reload list
                let res = db.get_all_lorebooks().await;
                if let Ok(mut books) = res {
                    let tags_res = db.get_all_lorebook_tags_flat().await;
                    if let Ok(tags_flat) = tags_res {
                        let mut tag_map: HashMap<i64, Vec<crate::models::Tag>> = HashMap::new();
                        for (lid, tag) in tags_flat {
                            tag_map.entry(lid).or_default().push(tag);
                        }
                        for b in &mut books {
                            if let Some(tags) = tag_map.remove(&b.id) {
                                b.tags = tags;
                            }
                        }
                    }
                    let _ = tx.send(UiEvent::LorebooksLoaded(Ok(books))).await;
                    ctx.request_repaint();
                }
            }
        });
    }

    // Now just a simplified helper that spawns a load
    pub fn load_character(&mut self, id: i64) {
        self.push_history();
        // Find in logic, or reload if needed. Currently we just select from list.
        if let Some(c) = self.characters.iter().find(|c| c.id == id).cloned() {
            self.selected_character = Some(c);
            self.selected_lorebook = None; // Clear other selection
            self.selected_entry = None;
            self.load_links(id);
            self.load_tags(id);
            self.mode = AppMode::Characters;
            self.central_view = CentralView::Editor;
            self.last_active_character_id = Some(id);
        }
    }

    pub fn load_lorebook(&mut self, id: i64) {
        self.push_history();
        if let Some(book) = self.lorebooks.iter().find(|l| l.id == id).cloned() {
            self.selected_lorebook = Some(book);
            self.selected_character = None; // Clear other selection
            self.load_lorebook_entries(id);
            self.load_lorebook_tags(id);
            self.mode = AppMode::Lorebooks;
            self.central_view = CentralView::Editor;
            self.last_active_lorebook_id = Some(id);
        }
    }

    pub fn delete_lorebook(&self, id: i64) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let res = db.delete_lorebook(id).await;
            let _ = tx
                .send(UiEvent::LorebookDeleted(
                    res.map(|_| id).map_err(|e| e.to_string()),
                ))
                .await;
            ctx.request_repaint();
        });
    }

    pub fn delete_character(&self, id: i64) {
        // Capture avatar path for cleanup
        let avatar_to_delete: Option<String> = self
            .characters
            .iter()
            .find(|c| c.id == id)
            .and_then(|c| c.avatar_path.clone());

        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let res = db.delete_character(id).await;
            let success = res.is_ok();

            if success {
                if let Some(ref path) = avatar_to_delete {
                    cleanup_avatar(path);
                }
            }

            let _ = tx
                .send(UiEvent::CharacterDeleted(
                    res.map(|_| id).map_err(|e| e.to_string()),
                ))
                .await;

            if success {
                let _ = tx
                    .send(UiEvent::StatusMessage(
                        "Character Deleted".to_string(),
                        egui::Color32::GREEN,
                    ))
                    .await;
            } else {
                let _ = tx
                    .send(UiEvent::StatusMessage(
                        "Delete Failed".to_string(),
                        egui::Color32::RED,
                    ))
                    .await;
            }
            ctx.request_repaint();
        });
    }

    pub fn move_character(&self, char_id: i64, target_coll_id: Option<i64>) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let res = db.move_character(char_id, target_coll_id).await;
            let _ = tx
                .send(UiEvent::CharacterMoved(
                    res.map(|_| (char_id, target_coll_id))
                        .map_err(|e| e.to_string()),
                ))
                .await;
            ctx.request_repaint();
        });
    }

    pub fn save_collection(&mut self, id: i64, name: String, parent_id: Option<i64>) {
        self.is_saving = true;
        let mut image_path = None;
        if id != 0 {
            if let Some(c) = self.collections.iter().find(|c| c.id == id) {
                image_path = c.image_path.clone();
            }
        }

        let tx = self.tx.clone();
        let db = self.db.clone();
        let col = crate::models::Collection {
            id,
            name,
            parent_id,
            display_order: 0,
            image_path,
        };
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let result = db.upsert_collection(&col).await.map_err(|e| e.to_string());
            let _ = tx.send(UiEvent::CollectionSaved(result)).await;
            ctx.request_repaint();
        });
    }

    pub fn reorder_collection(&self, id: i64, move_up: bool) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            if let Err(e) = db.reorder_collection(id, move_up).await {
                let _ = tx.send(UiEvent::CollectionSaved(Err(e.to_string()))).await;
            } else {
                let _ = tx
                    .send(UiEvent::CollectionSaved(Ok(id))) // Reuse Saved event to trigger reload
                    .await;
            }
            ctx.request_repaint();
        });
    }

    pub fn toggle_lore_link(&mut self, char_id: i64, lore_id: i64, link: bool) {
        if char_id == 0 {
            return;
        }

        // Optimistic UI update
        if link {
            self.lore_links.insert(lore_id);
            self.char_lore_map.entry(char_id).or_default().push(lore_id);
        } else {
            self.lore_links.remove(&lore_id);
            if let Some(links) = self.char_lore_map.get_mut(&char_id) {
                links.retain(|&id| id != lore_id);
            }
        }

        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let res = if link {
                db.link_lore(char_id, lore_id).await
            } else {
                db.unlink_lore(char_id, lore_id).await
            };
            let _ = tx
                .send(UiEvent::LinkUpdated(res.map_err(|e| e.to_string())))
                .await;
            ctx.request_repaint();
        });
    }

    pub fn create_new_character(&mut self, collection_id: Option<i64>) {
        if self.has_unsaved_changes() {
            self.popup_state = PopupState::UnsavedChanges {
                target: AppAction::CreateNewCharacter(collection_id),
            };
        } else {
            self.perform_create_new_character(collection_id);
        }
    }

    pub fn perform_create_new_character(&mut self, collection_id: Option<i64>) {
        self.push_history();
        let mut character = Character::default();
        character.collection_id = collection_id;

        // Immediate save
        self.save_character(character);

        // UI Navigation handled by event loop when CharacterSaved(Ok(c)) returns,
        // but we can set mode here for immediate visual switch if desired.
        // Actually, let's let the event loop handle selection, but switch mode now.
        self.mode = AppMode::Characters;
        self.central_view = CentralView::Editor;
    }

    pub fn toggle_favorite(&mut self, char_id: i64) {
        if let Some(c) = self.characters.iter_mut().find(|c| c.id == char_id) {
            c.is_favorite = !c.is_favorite;
            // Persist
            let char_clone = c.clone();
            // We use save_character which handles upsert.
            // But save_character might be too heavy if it reloads everything?
            // Actually it spawns a task and eventually reloads chars.
            // That's fine for now.
            self.save_character(char_clone);
        }
    }

    pub fn add_tag(&self, char_id: i64, name: String, is_external: bool) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let res = db
                .add_tag_to_character(char_id, &name, is_external)
                .await
                .map_err(|e| e.to_string());
            let _ = tx.send(UiEvent::TagOperationFinished(res)).await;
            ctx.request_repaint();
        });
    }

    pub fn remove_tag(&self, char_id: i64, tag_id: i64, is_external: bool) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let res = db
                .remove_tag_from_character(char_id, tag_id, is_external)
                .await
                .map_err(|e| e.to_string());
            let _ = tx.send(UiEvent::TagOperationFinished(res)).await;
            ctx.request_repaint();
        });
    }

    pub fn add_tag_to_lorebook(&self, lorebook_id: i64, name: String) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let res = db.add_tag_to_lorebook(lorebook_id, &name).await;

            match res {
                Ok(_) => {
                    let tags = db.get_tags_for_lorebook(lorebook_id).await;
                    if let Ok(t) = tags {
                        let _ = tx
                            .send(UiEvent::LorebookTagsLoaded(Ok((lorebook_id, t))))
                            .await;
                    }
                    let _ = tx.send(UiEvent::LorebookTagOperationFinished(Ok(()))).await;
                }
                Err(e) => {
                    let _ = tx
                        .send(UiEvent::LorebookTagOperationFinished(Err(e.to_string())))
                        .await;
                }
            }
            ctx.request_repaint();
        });
    }

    pub fn remove_tag_from_lorebook(&self, lorebook_id: i64, tag_id: i64) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            match db.remove_tag_from_lorebook(lorebook_id, tag_id).await {
                Ok(_) => {
                    let tags = db.get_tags_for_lorebook(lorebook_id).await;
                    if let Ok(t) = tags {
                        let _ = tx
                            .send(UiEvent::LorebookTagsLoaded(Ok((lorebook_id, t))))
                            .await;
                    }
                    let _ = tx.send(UiEvent::LorebookTagOperationFinished(Ok(()))).await;
                }
                Err(e) => {
                    let _ = tx
                        .send(UiEvent::LorebookTagOperationFinished(Err(e.to_string())))
                        .await;
                }
            }
            ctx.request_repaint();
        });
    }

    pub fn add_entry_to_lorebook(&self, lorebook_id: i64) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let mut entry = crate::models::LorebookEntry::default();
            entry.lorebook_id = lorebook_id;

            match db.add_entry_to_lorebook(&entry).await {
                Ok(id) => {
                    let _ = tx.send(UiEvent::LorebookEntryAdded(Ok(id))).await;
                    // Auto-reload
                    match db.get_entries_for_lorebook(lorebook_id).await {
                        Ok(entries) => {
                            let _ = tx
                                .send(UiEvent::LorebookEntriesLoaded(Ok((lorebook_id, entries))))
                                .await;
                        }
                        Err(_) => {}
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(UiEvent::LorebookEntryAdded(Err(e.to_string())))
                        .await;
                }
            }
            ctx.request_repaint();
        });
    }

    pub fn save_lorebook_entry(&self, entry: crate::models::LorebookEntry) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            match db.update_lorebook_entry(&entry).await {
                Ok(_) => {
                    let _ = tx.send(UiEvent::LorebookEntrySaved(Ok(()))).await;
                    if let Ok(entries) = db.get_entries_for_lorebook(entry.lorebook_id).await {
                        let _ = tx
                            .send(UiEvent::LorebookEntriesLoaded(Ok((
                                entry.lorebook_id,
                                entries,
                            ))))
                            .await;
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(UiEvent::LorebookEntrySaved(Err(e.to_string())))
                        .await;
                }
            }
            ctx.request_repaint();
        });
    }

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
        let new_template = crate::models::Template::default();
        self.selected_template = Some(new_template.clone());
        self.save_template(new_template);
        self.mode = AppMode::Templates;
        self.selected_character = None;
        self.selected_lorebook = None;
    }

    pub fn save_template(&mut self, mut template: crate::models::Template) {
        self.is_saving = true;
        self.status_message = None;
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            if let Err(e) = db.upsert_template(&mut template).await {
                let _ = tx.send(UiEvent::TemplateSaved(Err(e.to_string()))).await;
                let _ = tx
                    .send(UiEvent::StatusMessage(
                        format!("Template Save Failed: {}", e),
                        egui::Color32::RED,
                    ))
                    .await;
                ctx.request_repaint();
            } else {
                let _ = tx.send(UiEvent::TemplateSaved(Ok(template))).await;
                let _ = tx
                    .send(UiEvent::StatusMessage(
                        "Template Saved!".to_string(),
                        egui::Color32::GREEN,
                    ))
                    .await;
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
            let success = res.is_ok();

            let _ = tx
                .send(UiEvent::TemplateDeleted(
                    res.map(|_| id).map_err(|e| e.to_string()),
                ))
                .await;

            if success {
                let _ = tx
                    .send(UiEvent::StatusMessage(
                        "Template Deleted".to_string(),
                        egui::Color32::GREEN,
                    ))
                    .await;
            } else {
                let _ = tx
                    .send(UiEvent::StatusMessage(
                        "Delete Failed".to_string(),
                        egui::Color32::RED,
                    ))
                    .await;
            }
            ctx.request_repaint();
        });
    }
}
