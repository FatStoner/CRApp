use eframe::egui;
use crate::models::{Character, Collection};
use crate::ui::{CrapApp, AppMode, SortMode};

pub fn render_side_panel(app: &mut CrapApp, ctx: &egui::Context) {
    egui::SidePanel::left("side_panel")
        .min_width(250.0)
        .show(ctx, |ui| {
            ui.add_space(4.0);
            
            // Mode Switcher
            ui.horizontal(|ui| {
                ui.selectable_value(&mut app.mode, AppMode::Characters, "Characters");
                ui.selectable_value(&mut app.mode, AppMode::Lorebooks, "Lorebooks");
            });
            ui.separator();

            if let Some(err) = &app.loading_error {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
                if ui.button("Retry").clicked() { app.refresh_all(); }
            } else {
                 // Sorting specific to Characters
                 if app.mode == AppMode::Characters {
                     ui.horizontal(|ui| {
                         ui.label("Sort:");
                         ui.selectable_value(&mut app.sort_mode, SortMode::Alphabetical, "A-Z");
                         ui.selectable_value(&mut app.sort_mode, SortMode::NewestFirst, "New");
                         ui.selectable_value(&mut app.sort_mode, SortMode::RecentlyUpdated, "Upd");
                     });
                     ui.separator();
                 }

                // Search Bar
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut app.search_query).hint_text("Search name/tag..."));
                    if !app.search_query.is_empty() && ui.button("X").clicked() {
                        app.search_query.clear();
                    }
                });
                
                // Deep Search Trigger
                if ui.link("🔍 Deep Search (Global)").clicked() {
                    app.mode = if app.mode == AppMode::DeepSearch { AppMode::Characters } else { AppMode::DeepSearch };
                    // Auto-fill query if present
                    if !app.search_query.is_empty() {
                         app.deep_search_query = app.search_query.clone();
                    }
                }
                
                ui.separator();

                // Collection Tree / List
                let mut actions = Vec::new();
                egui::ScrollArea::vertical().show(ui, |ui| {
                     match app.mode {
                         AppMode::Characters => {
                             // Root Characters & Collections
                             // We start with parent_id: None
                             super::side_panel::render_tree(
                                 ui, 
                                 &app.collections, 
                                 &app.characters, 
                                 None, 
                                 app.selected_character.as_ref().map(|c| c.id), 
                                 app.selected_collection_id, 
                                 &mut actions,
                                 app.sort_mode,
                                 &app.search_query
                             );
                         },
                         AppMode::Lorebooks => {
                             // Simple list for Lorebooks for now (no implementation in original for tree)
                             for book in &app.lorebooks {
                                 if ui.selectable_label(app.selected_lorebook.as_ref().map(|l| l.id) == Some(book.id), &book.title).clicked() {
                                      app.selected_lorebook = Some(book.clone());
                                      app.mode = AppMode::Lorebooks; // Ensure mode
                                      app.selected_character = None; // Deselect char
                                 }
                             }
                         }
                         _ => {}
                     }
                });
                
                // Handle Actions from Tree
                for action in actions {
                    match action {
                        TreeAction::SelectChar(c) => {
                            app.selected_character = Some(c);
                            app.selected_lorebook = None;
                            app.mode = AppMode::Characters;
                            // Trigger loading details (tags, links)
                            if let Some(c) = &app.selected_character {
                                app.load_tags(c.id);
                                app.load_links(c.id);
                            }
                            app.central_view = crate::ui::CentralView::Editor;
                        },
                        TreeAction::SelectCollection(id) => {
                             app.selected_collection_id = Some(id);
                             app.selected_character = None;
                             app.central_view = crate::ui::CentralView::Browser;
                        },
                        TreeAction::DeselectCollection => {
                            app.selected_collection_id = None;
                            app.central_view = crate::ui::CentralView::Browser;
                        },
                        TreeAction::RenameCollection(id, current_name) => {
                             app.popup_state = crate::ui::PopupState::Renaming { id, name: current_name };
                        },
                        TreeAction::RequestDeleteCollection(id) => {
                             // Logic to check contents handled in update
                             // We can trigger it here via event or direct app method if exposed?
                             // But we can't call async self methods easily here if mutable borrow.
                             // We set a flag or handle it directly.
                             // Original code handled it in update() via popup state check helpers?
                             // Actually original check_delete_contents was local in update.
                             // We need to signal this up.
                             // Let's use a temporary field in App or return an enum?
                             // Return enum is cleaner but heavy refactor.
                             // Let's modify App state directly as we have &mut App.
                             // We can set a "check_delete_request"
                             // Just mimic the logic:
                             let child_colls = app.collections.iter().filter(|c| c.parent_id == Some(id)).count();
                             let child_chars = app.characters.iter().filter(|c| c.collection_id == Some(id)).count();
                             if child_colls + child_chars > 0 {
                                  app.popup_state = crate::ui::PopupState::DeleteWarning { id, count: child_colls + child_chars };
                             } else {
                                  // Direct delete attempt (no children) - requires async DB call.
                                  // We can spawn it here? Access to DB is inside App.
                                  // Yes:
                                   let tx = app.tx.clone();
                                   let db = app.db.clone();
                                   tokio::spawn(async move {
                                        let res = db.delete_collection(id).await;
                                        let _ = tx.send(crate::ui::UiEvent::CollectionDeleted(res.map(|_| id).map_err(|e| e.to_string())));
                                   });
                             }
                        },
                        TreeAction::CreateSubfolder(parent_id) => {
                             app.save_collection("New Folder".to_string(), Some(parent_id));
                        },
                        TreeAction::CreateRootFolder => {
                             app.save_collection("New Folder".to_string(), None);
                        }
                    }
                }
                
                // Bottom: Add Buttons
                 ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                     ui.add_space(5.0);
                     ui.horizontal(|ui| {
                         if ui.button("➕ New Character").clicked() {
                              app.selected_character = Some(Character::default());
                              app.selected_character.as_mut().unwrap().collection_id = app.selected_collection_id;
                              app.selected_character.as_mut().unwrap().collection_id = app.selected_collection_id;
                              app.mode = AppMode::Characters;
                              app.central_view = crate::ui::CentralView::Editor;
                         }
                         if ui.button("➕ New Lorebook").clicked() {
                              app.selected_lorebook = Some(crate::models::Lorebook::default());
                              app.mode = AppMode::Lorebooks;
                         }
                     });
                     if app.mode == AppMode::Characters {
                          if ui.button("📁 New Root Folder").clicked() {
                               app.save_collection("New Folder".to_string(), None);
                          }
                     }
                 });
            }
        });
}

pub enum TreeAction {
    SelectChar(Character),
    SelectCollection(i64),
    DeselectCollection,
    RenameCollection(i64, String),
    RequestDeleteCollection(i64),
    CreateSubfolder(i64),
    CreateRootFolder,
}

// Move render_tree here
pub fn render_tree(
    ui: &mut egui::Ui,
    collections: &[Collection],
    characters: &[Character],
    parent_id: Option<i64>,
    selected_char_id: Option<i64>,
    selected_coll_id: Option<i64>,
    actions: &mut Vec<TreeAction>,
    sort_mode: SortMode,
    search_query: &str,
) {
    let query_lower = search_query.to_lowercase();
    let is_search_active = !search_query.is_empty();

    // 1. Render Sub-collections
    let node_colls: Vec<&Collection> = collections.iter().filter(|c| c.parent_id == parent_id).collect();
    for col in node_colls {
        let has_visible_descendants = if is_search_active {
             has_matches(col.id, collections, characters, &query_lower)
        } else {
             true
        };
        
        let is_selected = Some(col.id) == selected_coll_id;
        let id_str = ui.make_persistent_id(col.id);
        
        let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id_str, false);
        let was_open = state.is_open();
        
        if is_search_active && has_visible_descendants {
             state.set_open(true);
        }

        let mut toggle = false;
        let header_res = state.show_header(ui, |ui| {
             let alpha = if has_visible_descendants { 255 } else { 100 };
             let text_color = if is_selected { egui::Color32::WHITE } else {
                  ui.visuals().text_color().linear_multiply(alpha as f32 / 255.0)
             };

             let label = egui::RichText::new(format!("📁 {}", col.name)).strong().color(text_color);
             let mut response = ui.selectable_label(is_selected, label);
             
             if !has_visible_descendants {
                 response = response.on_hover_text("No matching characters in this folder");
             }
             
             if response.clicked() {
                 actions.push(TreeAction::SelectCollection(col.id));
                 toggle = true;
             }
             
             response.context_menu(|ui| {
                 if ui.button("Rename").clicked() {
                     actions.push(TreeAction::RenameCollection(col.id, col.name.clone()));
                     ui.close_menu();
                 }
                 if ui.button("New Subfolder").clicked() {
                     actions.push(TreeAction::CreateSubfolder(col.id));
                     ui.close_menu();
                 }
                 ui.separator();
                 if ui.button("Delete").clicked() {
                     actions.push(TreeAction::RequestDeleteCollection(col.id));
                     ui.close_menu();
                 }
             });
        });
        
        header_res.body(|ui| {
            render_tree(ui, collections, characters, Some(col.id), selected_char_id, selected_coll_id, actions, sort_mode, search_query);
        });

        if toggle {
             if let Some(mut state) = egui::collapsing_header::CollapsingState::load(ui.ctx(), id_str) {
                 let is_open_now = state.is_open();
                 if was_open == is_open_now {
                      state.toggle(ui);
                      state.store(ui.ctx());
                      ui.ctx().request_repaint();
                 }
             }
        }
    }

    // 2. Render Characters
    let mut node_chars: Vec<&Character> = characters.iter().filter(|c| c.collection_id == parent_id).collect();
    
    if is_search_active {
         node_chars.retain(|c| {
             let in_name = c.name.to_lowercase().contains(&query_lower);
             let in_title = c.char_title.to_lowercase().contains(&query_lower);
             let in_app_tags = c.app_tags.iter().any(|t| t.name.to_lowercase().contains(&query_lower));
             let in_ext_tags = c.external_tags.iter().any(|t| t.name.to_lowercase().contains(&query_lower));
             in_name || in_title || in_app_tags || in_ext_tags
         });
    }

    match sort_mode {
        SortMode::Alphabetical => node_chars.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
        SortMode::NewestFirst => node_chars.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
        SortMode::RecentlyUpdated => node_chars.sort_by(|a, b| b.updated_at.cmp(&a.updated_at)),
    }

    for char in node_chars {
        let is_selected = Some(char.id) == selected_char_id;
        
        let item_height = 48.0;
        let (rect, response) = ui.allocate_exact_size(
             egui::vec2(ui.available_width(), item_height), 
             egui::Sense::click()
        );

        if response.clicked() {
             actions.push(TreeAction::SelectChar(char.clone()));
        }
        
        if is_selected {
             ui.painter().rect_filled(rect, 4.0, ui.visuals().selection.bg_fill);
        } else if response.hovered() {
             ui.painter().rect_filled(rect, 4.0, ui.visuals().widgets.hovered.bg_fill);
        }

        let thumb_size = 40.0;
        let thumb_rect = egui::Rect::from_min_size(rect.min + egui::vec2(4.0, 4.0), egui::vec2(thumb_size, thumb_size));
        
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
                 .rounding(4.0)
                 .paint_at(ui, thumb_rect);
        } else {
             ui.painter().rect_filled(thumb_rect, 4.0, egui::Color32::from_gray(70));
             let initial = char.name.chars().next().unwrap_or('?').to_uppercase().to_string();
             ui.painter().text(
                 thumb_rect.center(), 
                 egui::Align2::CENTER_CENTER, 
                 initial, 
                 egui::FontId::proportional(20.0), 
                 egui::Color32::WHITE
             );
        }
        
        let text_left = thumb_rect.max.x + 8.0;
        let name_font = egui::FontId::proportional(15.0);
        let name_color = if is_selected { egui::Color32::WHITE } else { ui.visuals().text_color() };
        
        let name_galley = ui.painter().layout_no_wrap(char.name.clone(), name_font, name_color);
        let name_pos = egui::pos2(text_left, rect.min.y + 4.0);
        
        ui.painter().with_clip_rect(rect).galley(name_pos, name_galley, egui::Color32::WHITE);
        
        if !char.char_title.is_empty() {
             let title_font = egui::FontId::proportional(12.0);
             let title_color = if is_selected { 
                 egui::Color32::from_white_alpha(200) 
             } else { 
                 ui.visuals().text_color().linear_multiply(0.7) 
             };
             
             let title_galley = ui.painter().layout_no_wrap(char.char_title.clone(), title_font, title_color);
             let title_pos = egui::pos2(text_left, rect.min.y + 24.0);
             
             ui.painter().with_clip_rect(rect).galley(title_pos, title_galley, egui::Color32::WHITE);
        }
    }
}

pub fn has_matches(collection_id: i64, collections: &[Collection], characters: &[Character], query: &str) -> bool {
    if characters.iter().any(|c| c.collection_id == Some(collection_id) && {
        let name_match = c.name.to_lowercase().contains(query);
        let title_match = c.char_title.to_lowercase().contains(query);
        let app_tag_match = c.app_tags.iter().any(|t| t.name.to_lowercase().contains(query));
        let ext_tag_match = c.external_tags.iter().any(|t| t.name.to_lowercase().contains(query));
        name_match || title_match || app_tag_match || ext_tag_match
    }) {
        return true;
    }
    
    let sub_colls: Vec<&Collection> = collections.iter().filter(|c| c.parent_id == Some(collection_id)).collect();
    for sub in sub_colls {
        if has_matches(sub.id, collections, characters, query) {
            return true;
        }
    }
    
    false
}
