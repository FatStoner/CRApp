use super::app::CrapApp;
use super::types::UiEvent;
use crate::models::Character;
use crate::ui::{AppMode, CentralView, LorebookTab, ParsedCharacterData, PopupState};
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
            UiEvent::SpellCheckSettingLoaded(enabled) => {
                app.enable_spell_check = enabled;
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
                    // Update selected
                    if let Some(l) = &mut app.selected_lorebook {
                        if l.id == lid {
                            l.entries = entries;
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
            UiEvent::ImportFileLoaded(res) => {
                match res {
                    Ok(json_content) => {
                        if let Ok(mut char_obj) = serde_json::from_str::<Character>(&json_content) {
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
                                avatar_path: char_obj.avatar_path.clone(),
                            };

                            // Force "New Character" mode
                            app.selected_character = Some(Character::default());
                            app.mode = AppMode::Characters;

                            app.parsed_data = Some(parsed);
                            app.show_import_modal = true;
                            app.import_text.clear(); // Clear clipboard text if any

                            app.set_status_with_duration(
                                "File loaded for review.".to_string(),
                                egui::Color32::GREEN,
                                Duration::from_secs(10),
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
            UiEvent::ImportCharacterData(res) => {
                match res {
                    Ok(parsed) => {
                        // Force "New Character" mode
                        app.selected_character = Some(Character::default());
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
        }
    }

    if received_event {
        ctx.request_repaint();
    }
}
