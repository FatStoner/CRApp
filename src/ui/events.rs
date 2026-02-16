use super::controllers::CrapApp;
use super::types::UiEvent;
use crate::models::Character;
use crate::ui::{AppMode, CentralView, LorebookTab, PopupState};
use eframe::egui;
use std::time::Duration;

pub fn handle_ui_events(app: &mut CrapApp, ctx: &egui::Context) {
    let mut received_event = false;
    while let Ok(event) = app.rx.try_recv() {
        received_event = true;
        match event {
            UiEvent::CharactersLoaded(res) => match res {
                Ok(list) => {
                    app.characters = list;
                    app.loading_error = None;
                }
                Err(e) => {
                    eprintln!("Load error: {}", e);
                    app.loading_error = Some(e);
                }
            },
            UiEvent::LorebooksLoaded(res) => match res {
                Ok(mut books) => {
                    // Preserve entries from existing cache to prevent "dirty" state due to shallow reload
                    for new_book in &mut books {
                        if let Some(existing) = app.lorebooks.iter().find(|b| b.id == new_book.id) {
                            if !existing.entries.is_empty() {
                                new_book.entries = existing.entries.clone();
                            }
                        }
                    }
                    app.lorebooks = books;
                }
                Err(e) => {
                    app.loading_error = Some(e);
                }
            },
            UiEvent::CollectionsLoaded(res) => match res {
                Ok(collections) => app.collections = collections,
                Err(e) => {
                    app.loading_error = Some(e);
                }
            },
            UiEvent::GalleryImageAdded(path) => {
                if let Some(ctx) = &mut app.gallery_context {
                    if !ctx.contains(&path) {
                        ctx.push(path);
                    }
                }
            }
            UiEvent::StatisticsCalculated(data) => {
                if let Some(state) = &mut app.statistics_state {
                    state.data = Some(data);
                    state.is_calculating = false;
                }
                ctx.request_repaint();
            }
            UiEvent::ThemeLoaded(res) => {
                if let Ok(mode) = res {
                    app.theme = mode;
                    app.apply_theme();
                }
            }
            UiEvent::CustomBackgroundLoaded(enabled) => {
                app.use_custom_background = enabled;
            }
            UiEvent::ScaleLoaded(res) => {
                if let Ok(scale) = res {
                    app.ui_scale = scale;
                    app.ctx.set_pixels_per_point(scale);
                }
            }
            UiEvent::LoreLinksLoaded(res) => match res {
                Ok(set) => app.lore_links = set,
                Err(e) => eprintln!("Link load error: {}", e),
            },
            UiEvent::WatermarkLoaded(show) => {
                app.show_watermark = show;
            }
            UiEvent::BackgroundLoaded(show) => {
                app.show_background = show;
            }
            UiEvent::BackgroundScaleLoaded(scale) => {
                app.background_scale = scale;
            }
            UiEvent::SpellCheckSettingLoaded(enabled) => {
                app.enable_spell_check = enabled;
            }
            UiEvent::EditorFontLoaded(font) => {
                app.editor_font = font;
            }
            UiEvent::LoreLinksBulkLoaded(map) => {
                app.char_lore_map = map;
            }
            UiEvent::CharacterSaved(res) => {
                app.is_saving = false;
                match res {
                    Ok(c) => {
                        // Ensure links and tags are loaded (critical for new characters)
                        app.load_links(c.id);
                        app.load_tags(c.id);

                        // UPDATE CACHE (Critical for dirty check)
                        // We do this BEFORE moving c into selected_character, or clone it.
                        if let Some(existing) =
                            app.characters.iter_mut().find(|char| char.id == c.id)
                        {
                            *existing = c.clone();
                        } else {
                            app.characters.push(c.clone());
                        }

                        app.selected_character = Some(c);
                        app.set_status("Character Saved!".to_string(), egui::Color32::GREEN);

                        // Handle pending action if any
                        if let Some(action) = app.pending_action.take() {
                            app.perform_action(action, &ctx);
                        }
                    }
                    Err(e) => {
                        app.set_status(format!("Save Error: {}", e), egui::Color32::RED);
                        app.pending_action = None;
                    }
                }
            }
            UiEvent::LorebookSaved(res) => {
                app.is_saving = false;
                match res {
                    Ok(l) => {
                        // UPDATE CACHE (Critical for dirty check)
                        if let Some(existing) = app.lorebooks.iter_mut().find(|b| b.id == l.id) {
                            *existing = l.clone();
                        } else {
                            app.lorebooks.push(l.clone());
                        }

                        app.selected_lorebook = Some(l);
                        app.set_status("Lorebook Saved!".to_string(), egui::Color32::GREEN);

                        // Handle pending action if any (Fix for Save & Continue)
                        if let Some(action) = app.pending_action.take() {
                            app.perform_action(action, &ctx);
                        }
                    }
                    Err(e) => {
                        app.set_status(format!("Save Error: {}", e), egui::Color32::RED);
                        app.pending_action = None;
                    }
                }
            }
            UiEvent::CollectionSaved(res) => {
                app.is_saving = false;
                match res {
                    Ok(_) => {
                        app.set_status("Collection Saved!".to_string(), egui::Color32::GREEN);
                        app.reload_collections();
                        app.popup_state = PopupState::None;
                    }
                    Err(e) => app.set_status(format!("Save Error: {}", e), egui::Color32::RED),
                }
            }
            UiEvent::CollectionDeleted(res) => {
                app.is_saving = false;
                match res {
                    Ok(id) => {
                        app.set_status("Collection Deleted".to_string(), egui::Color32::GREEN);
                        // Optimistic update
                        app.collections.retain(|c| c.id != id);
                        app.reload_collections();
                        app.reload_characters();
                        if app.selected_collection_id == Some(id) {
                            app.selected_collection_id = None;
                        }
                    }
                    Err(e) => app.set_status(format!("Delete Error: {}", e), egui::Color32::RED),
                }
            }
            UiEvent::LinkUpdated(res) => {
                if let Err(e) = res {
                    app.set_status(format!("Link Error: {}", e), egui::Color32::RED);
                }
            }
            UiEvent::TagsLoaded(res) => match res {
                Ok((id, tags, ext)) => {
                    if let Some(c) = &mut app.selected_character {
                        if c.id == id {
                            c.app_tags = tags;
                            c.external_tags = ext;
                        }
                    }
                }
                Err(e) => app.set_status(format!("Tag Load Error: {}", e), egui::Color32::RED),
            },
            UiEvent::TagOperationFinished(res) => match res {
                Ok(_) => {
                    if let Some(c) = &app.selected_character {
                        app.load_tags(c.id);
                    }
                    app.refresh_all();
                }
                Err(e) => app.set_status(format!("Tag Error: {}", e), egui::Color32::RED),
            },
            UiEvent::LorebookTagsLoaded(res) => match res {
                Ok((id, tags)) => {
                    // Update selected if matches
                    if let Some(l) = &mut app.selected_lorebook {
                        if l.id == id {
                            l.tags = tags.clone();
                        }
                    }
                    // Update cache
                    if let Some(cached) = app.lorebooks.iter_mut().find(|b| b.id == id) {
                        cached.tags = tags;
                    }
                }
                Err(e) => eprintln!("Lorebook tags load error: {}", e),
            },
            UiEvent::LorebookTagOperationFinished(res) => {
                if let Err(e) = res {
                    app.set_status(format!("Tag Error: {}", e), egui::Color32::RED);
                }
            }
            UiEvent::LorebookEntriesLoaded(res) => match res {
                Ok((lid, entries)) => {
                    // Update cache
                    if let Some(l) = app.lorebooks.iter_mut().find(|l| l.id == lid) {
                        l.entries = entries.clone();
                    }

                    // Handle Selection Persistence / Smart Selection
                    if let Some(l) = &mut app.selected_lorebook {
                        if l.id == lid {
                            // 1. Capture current selection index before update
                            let mut current_idx = None;
                            if let Some(selected) = &app.selected_entry {
                                // We check against the OLD entries in the selected lorebook
                                current_idx = l.entries.iter().position(|e| e.id == selected.id);
                            }

                            // 2. Update the entries
                            l.entries = entries;

                            // 3. Try to restore selection or pick nearest
                            if let Some(old_idx) = current_idx {
                                // First, try to find the EXACT same entry by ID (in case of reorder/edit)
                                let new_pos_by_id = l
                                    .entries
                                    .iter()
                                    .position(|e| e.id == app.selected_entry.as_ref().unwrap().id);

                                if let Some(new_idx) = new_pos_by_id {
                                    // It still exists! Just update the selected_entry data directly
                                    app.selected_entry = Some(l.entries[new_idx].clone());
                                } else {
                                    // It's gone! (Deleted)
                                    // Pick the entry at the same index, or the last one if we were at the end
                                    if !l.entries.is_empty() {
                                        let new_idx = if old_idx >= l.entries.len() {
                                            l.entries.len() - 1
                                        } else {
                                            old_idx
                                        };
                                        app.selected_entry = Some(l.entries[new_idx].clone());
                                    } else {
                                        // No entries left
                                        app.selected_entry = None;
                                    }
                                }
                            } else {
                                // No previous selection, or it wasn't in the list?
                                // Just ensure selected_entry is None if the list is now empty, otherwise leave it alone
                                if l.entries.is_empty() {
                                    app.selected_entry = None;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    app.set_status(format!("Failed to load entries: {}", e), egui::Color32::RED)
                }
            },
            UiEvent::LorebookEntryAdded(res) => match res {
                Ok(_) => app.set_status("Entry added".to_string(), egui::Color32::GREEN),
                Err(e) => app.set_status(format!("Failed to add entry: {}", e), egui::Color32::RED),
            },
            UiEvent::LorebookEntrySaved(res) => match res {
                Ok(_) => {} // Silent save
                Err(e) => {
                    app.set_status(format!("Failed to save entry: {}", e), egui::Color32::RED)
                }
            },
            UiEvent::LorebookEntryDeleted(res) => match res {
                Ok(_) => app.set_status("Entry deleted".to_string(), egui::Color32::GREEN),
                Err(e) => {
                    app.set_status(format!("Failed to delete entry: {}", e), egui::Color32::RED)
                }
            },
            UiEvent::LorebookDeleted(res) => match res {
                Ok(id) => {
                    app.set_status("Lorebook Deleted".to_string(), egui::Color32::GREEN);
                    app.lorebooks.retain(|b| b.id != id);
                    if let Some(selected) = &app.selected_lorebook {
                        if selected.id == id {
                            app.selected_lorebook = None;
                        }
                    }
                }
                Err(e) => app.set_status(format!("Delete Error: {}", e), egui::Color32::RED),
            },
            UiEvent::TemplatesLoaded(res) => match res {
                Ok(list) => {
                    app.templates = list;
                }
                Err(e) => {
                    eprintln!("Load error: {}", e);
                    app.loading_error = Some(e);
                }
            },
            UiEvent::TemplateSaved(res) => {
                app.is_saving = false;
                match res {
                    Ok(t) => {
                        app.selected_template = Some(t);
                        app.set_status("Template Saved!".to_string(), egui::Color32::GREEN);
                    }
                    Err(e) => {
                        app.set_status(format!("Save Error: {}", e), egui::Color32::RED);
                    }
                }
            }
            UiEvent::TemplateDeleted(res) => {
                app.is_saving = false;
                match res {
                    Ok(id) => {
                        app.set_status("Template Deleted".to_string(), egui::Color32::GREEN);
                        app.templates.retain(|t| t.id != id);
                        if let Some(selected) = &app.selected_template {
                            if selected.id == id {
                                app.selected_template = None;
                            }
                        }
                    }
                    Err(e) => {
                        app.set_status(format!("Delete Error: {}", e), egui::Color32::RED);
                    }
                }
            }
            UiEvent::DeepSearchCompleted(res) => {
                app.is_deep_searching = false;
                match res {
                    Ok(results) => app.deep_search_results = results,
                    Err(e) => app.set_status(format!("Search failed: {}", e), egui::Color32::RED),
                }
            }
            UiEvent::UiRepaint => {
                // Just wakes the loop, nothing to do
            }
            UiEvent::CharacterDeleted(res) => {
                match res {
                    Ok(id) => {
                        // Optimistic update
                        app.characters.retain(|c| c.id != id);
                        if let Some(selected) = &app.selected_character {
                            if selected.id == id {
                                app.selected_character = None;
                                app.central_view = CentralView::Browser;
                            }
                        }
                        app.set_status("Character Deleted".to_string(), egui::Color32::GREEN);
                    }
                    Err(e) => app.set_status(format!("Delete Error: {}", e), egui::Color32::RED),
                }
            }
            UiEvent::CharacterMoved(res) => {
                match res {
                    Ok((char_id, new_coll_id)) => {
                        app.set_status("Character Moved".to_string(), egui::Color32::GREEN);

                        // 1. Sync Selected Character (Fix for editor desync)
                        if let Some(selected) = &mut app.selected_character {
                            if selected.id == char_id {
                                selected.collection_id = new_coll_id;
                            }
                        }

                        // 2. Optimistic List Update
                        if let Some(c) = app.characters.iter_mut().find(|c| c.id == char_id) {
                            c.collection_id = new_coll_id;
                        }

                        app.reload_characters();
                    }
                    Err(e) => app.set_status(format!("Move Error: {}", e), egui::Color32::RED),
                }
            }
            UiEvent::ImportFileLoaded(res, target_id) => {
                match res {
                    Ok(json_content) => {
                        if let Ok(mut char_obj) =
                            serde_json::from_str::<crate::models::Character>(&json_content)
                        {
                            // ... Existing Character Logic ...
                            // Clean ID for new import only if we are not updating
                            if target_id.is_none() {
                                char_obj.id = 0;
                            } else {
                                // If updating, we keep the ID of the target, NOT the file content
                                char_obj.id = target_id.unwrap() as i64;
                            }

                            // Map to ParsedCharacterData for review
                            let parsed = crate::ui::ParsedCharacterData {
                                name: char_obj.name.clone(),
                                char_name: char_obj.char_name.clone(),
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
                                avatar_path: char_obj.avatar_path.clone(),
                            };

                            if let Some(tid) = target_id {
                                if let Some(existing) =
                                    app.characters.iter().find(|c| c.id == tid as i64)
                                {
                                    app.selected_character = Some(existing.clone());
                                } else {
                                    app.selected_character =
                                        Some(crate::models::Character::default());
                                }
                            } else {
                                app.selected_character = Some(crate::models::Character::default());
                            }

                            app.mode = AppMode::Characters;
                            app.parsed_data = Some(parsed);
                            app.show_import_modal = true;
                            app.import_text.clear();
                            app.set_status_with_duration(
                                "Character loaded for review.".to_string(),
                                egui::Color32::GREEN,
                                std::time::Duration::from_secs(10),
                            );
                        } else {
                            app.set_status(
                                "Failed to parse file structure.".to_string(),
                                egui::Color32::RED,
                            );
                        }
                    }
                    Err(e) => app.set_status(format!("Read Error: {}", e), egui::Color32::RED),
                }
            }
            UiEvent::ImportCharacterData(res, target_id) => {
                match res {
                    Ok(parsed) => {
                        if let Some(tid) = target_id {
                            if let Some(existing) =
                                app.characters.iter().find(|c| c.id == tid as i64)
                            {
                                app.selected_character = Some(existing.clone());
                            } else {
                                // Fallback if ID not found
                                app.selected_character = Some(Character::default());
                            }
                        } else {
                            // Force "New Character" mode
                            app.selected_character = Some(Character::default());
                        }

                        app.mode = AppMode::Characters;

                        app.parsed_data = Some(parsed);
                        app.show_import_modal = true;
                        app.import_text.clear();

                        app.set_status_with_duration(
                            "Character data imported for review.".to_string(),
                            egui::Color32::GREEN,
                            Duration::from_secs(10),
                        );
                    }
                    Err(e) => app.set_status(format!("Import Error: {}", e), egui::Color32::RED),
                }
            }
            UiEvent::DbExportFinished(res) => match res {
                Ok(path) => app.set_status(
                    format!("Database exported to: {}", path),
                    egui::Color32::GREEN,
                ),
                Err(e) => app.set_status(format!("Export Failed: {}", e), egui::Color32::RED),
            },
            UiEvent::DbReloaded(res) => match res {
                Ok(new_db) => {
                    app.db = new_db;
                    app.set_status(
                        "Database imported successfully. Reloading view...".to_string(),
                        egui::Color32::GREEN,
                    );
                    app.refresh_all();
                }
                Err(e) => {
                    app.set_status(
                        format!("CRITICAL: Database Swap Failed: {}", e),
                        egui::Color32::RED,
                    );
                }
            },

            UiEvent::TokenCountCalculated(id, tokens, chars) => {
                app.token_cache.insert(id, (tokens, chars));
                app.token_calc_in_progress.remove(&id);
            }

            UiEvent::LorebookImported(lb) => {
                app.set_status(
                    "Lorebook Imported Successfully".to_string(),
                    egui::Color32::GREEN,
                );
                app.popup_state = PopupState::None;
                app.reload_lorebooks();
                app.selected_lorebook = Some(lb);
                app.selected_character = None;
                app.mode = AppMode::Lorebooks;
                app.active_lorebook_tab = LorebookTab::Entries;
            }

            UiEvent::StatusMessage(msg, color) => {
                app.set_status(msg, color);
            }

            UiEvent::UpdateAvailable(version, tag) => {
                app.popup_state = PopupState::UpdateAvailable { version, tag };
            }

            UiEvent::UpdateStarted => {
                app.popup_state = PopupState::Updating;
            }

            UiEvent::UpdateFailed(error) => {
                app.popup_state = PopupState::UpdateError { error };
            }

            UiEvent::UpdateCheckFinished(res, is_manual) => {
                app.is_checking_for_updates = false;
                match res {
                    Ok(Some((version, tag))) => {
                        app.popup_state = PopupState::UpdateAvailable { version, tag };
                    }
                    Ok(None) => {
                        if is_manual {
                            // Show "Up to Date" popup for manual checks
                            app.popup_state = PopupState::UpToDate;
                        }
                    }
                    Err(e) => {
                        if is_manual {
                            app.set_status(
                                format!("Update check failed: {}", e),
                                egui::Color32::RED,
                            );
                        } else {
                            eprintln!("Background update check failed: {}", e);
                        }
                    }
                }
            }

            UiEvent::CheckUpdatesAtStartLoaded(enabled) => {
                app.check_updates_at_start = enabled;

                // Triggers the check if enabled (only in release or forced)
                // We use a flag or cfg to decide if we run it.
                // Since this event comes from async load at startup, it's a good place to trigger the check.

                #[cfg(not(debug_assertions))]
                if enabled {
                    // Prevent check if already running (though unlikely at very start, but good safety)
                    if !app.is_checking_for_updates {
                        app.is_checking_for_updates = true;
                        let tx = app.tx.clone();
                        std::thread::spawn(move || match crate::updater::check_for_updates() {
                            Ok(res) => {
                                let _ =
                                    tx.blocking_send(UiEvent::UpdateCheckFinished(Ok(res), false));
                            }
                            Err(e) => {
                                let _ = tx.blocking_send(UiEvent::UpdateCheckFinished(
                                    Err(e.to_string()),
                                    false,
                                ));
                            }
                        });
                    }
                }
            }
        }
    }

    if received_event {
        ctx.request_repaint();
    }
}
