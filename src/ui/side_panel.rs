use crate::models::{Character, Collection, ThemeMode};
use crate::ui::{AppMode, CrapApp, SortDirection, SortMode};
use eframe::egui;

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
            ui.separator();

            // Theme Toggle
            ui.horizontal(|ui| {
                ui.label("Theme:");
                let theme_txt = match app.theme {
                    ThemeMode::System => "🌗 Auto",
                    ThemeMode::Light => "☀️ Light",
                    ThemeMode::Dark => "🌙 Dark",
                };
                if ui.button(theme_txt).clicked() {
                    let new_theme = match app.theme {
                        ThemeMode::System => ThemeMode::Light,
                        ThemeMode::Light => ThemeMode::Dark,
                        ThemeMode::Dark => ThemeMode::System,
                    };
                    app.set_theme(new_theme);
                }

                ui.separator();

                ui.label("Scale:");
                let current_scale = (app.ui_scale * 100.0).round() as i32;
                let mut selected = current_scale;

                egui::ComboBox::from_id_salt("scale_combo")
                    .selected_text(format!("{}%", current_scale))
                    .show_ui(ui, |ui| {
                        for p in (80..=150).step_by(10) {
                            if ui
                                .selectable_value(&mut selected, p, format!("{}%", p))
                                .clicked()
                            {
                                app.set_scale(selected as f32 / 100.0);
                            }
                        }
                    });
            });

            if let Some(err) = &app.loading_error {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
                if ui.button("Retry").clicked() {
                    app.refresh_all();
                }
            } else {
                // Sorting specific to Characters
                if app.mode == AppMode::Characters {
                    ui.horizontal(|ui| {
                        ui.label("Sort:");

                        let mut sort_btn = |mode: SortMode, label: &str| {
                            let is_selected = app.sort_mode == mode;
                            let mut display_label = label.to_string();
                            if is_selected {
                                match app.sort_direction {
                                    SortDirection::Ascending => display_label.push_str(" v"),
                                    SortDirection::Descending => display_label.push_str(" ^"),
                                }
                            }

                            if ui.selectable_label(is_selected, display_label).clicked() {
                                if is_selected {
                                    app.sort_direction = match app.sort_direction {
                                        SortDirection::Ascending => SortDirection::Descending,
                                        SortDirection::Descending => SortDirection::Ascending,
                                    };
                                } else {
                                    app.sort_mode = mode;
                                    app.sort_direction = SortDirection::Ascending;
                                }
                            }
                        };

                        sort_btn(SortMode::Alphabetical, "A-Z");
                        sort_btn(SortMode::NewestFirst, "New");
                        sort_btn(SortMode::RecentlyUpdated, "Upd");
                    });
                    ui.separator();
                }

                // Search Bar
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut app.search_query)
                            .hint_text("Search name/tag..."),
                    );
                    if !app.search_query.is_empty() && ui.button("X").clicked() {
                        app.search_query.clear();
                    }
                });

                // Deep Search Trigger
                if ui.link("🔍 Deep Search (Global)").clicked() {
                    app.mode = if app.mode == AppMode::DeepSearch {
                        AppMode::Characters
                    } else {
                        AppMode::DeepSearch
                    };
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
                            if ui
                                .selectable_label(app.viewing_all_characters, "📁 All Characters")
                                .clicked()
                            {
                                actions.push(TreeAction::SwitchToAll);
                            }

                            ui.separator();

                            let is_uncategorized =
                                app.selected_collection_id.is_none() && !app.viewing_all_characters;
                            let response =
                                ui.selectable_label(is_uncategorized, "📁 Uncategorized");
                            if response.clicked() {
                                actions.push(TreeAction::DeselectCollection);
                            }

                            if let Some(_) = response.dnd_hover_payload::<i64>() {
                                ui.painter().rect_stroke(
                                    response.rect,
                                    2.0,
                                    egui::Stroke::new(2.0, egui::Color32::GREEN),
                                );
                            }
                            if let Some(dropped_id) = response.dnd_release_payload::<i64>() {
                                actions.push(TreeAction::MoveCharacter(*dropped_id, None));
                            }

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
                                app.sort_direction,
                                &app.search_query,
                            );

                            // Fill empty space for context menu
                            let available = ui.available_height();
                            let height = if available.is_finite() && available > 0.0 {
                                available
                            } else {
                                150.0
                            };

                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width(), height.max(50.0)),
                                egui::Sense::click(),
                            );

                            // Context menu handled below
                            // response.clicked() logic removed per user request to avoid accidental deselection
                            // actions.push(TreeAction::DeselectCollection);

                            response.context_menu(|ui| {
                                if ui.button("➕ New Character").clicked() {
                                    actions.push(TreeAction::CreateNewCharacter(None));
                                    ui.close_menu();
                                }
                                if ui.button("📁 New Folder").clicked() {
                                    actions.push(TreeAction::CreateRootFolder);
                                    ui.close_menu();
                                }
                            });
                        }
                        AppMode::Lorebooks => {
                            // Simple list for Lorebooks for now (no implementation in original for tree)
                            for book in &app.lorebooks {
                                if ui
                                    .selectable_label(
                                        app.selected_lorebook.as_ref().map(|l| l.id)
                                            == Some(book.id),
                                        &book.title,
                                    )
                                    .clicked()
                                {
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
                            app.request_character_switch(c.id);
                        }
                        TreeAction::SelectCollection(id) => {
                            app.request_collection_switch(Some(id));
                        }
                        TreeAction::DeselectCollection => {
                            app.request_collection_switch(None);
                        }
                        TreeAction::RenameCollection(id, current_name) => {
                            app.popup_state = crate::ui::PopupState::Renaming {
                                id,
                                name: current_name,
                            };
                        }
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
                            let child_colls = app
                                .collections
                                .iter()
                                .filter(|c| c.parent_id == Some(id))
                                .count();
                            let child_chars = app
                                .characters
                                .iter()
                                .filter(|c| c.collection_id == Some(id))
                                .count();
                            if child_colls + child_chars > 0 {
                                app.popup_state = crate::ui::PopupState::DeleteWarning {
                                    id,
                                    count: child_colls + child_chars,
                                };
                            } else {
                                app.delete_collection(id);
                                ctx.request_repaint();
                            }
                        }
                        TreeAction::CreateSubfolder(parent_id) => {
                            app.save_collection(0, "New Folder".to_string(), Some(parent_id));
                        }
                        TreeAction::CreateRootFolder => {
                            app.save_collection(0, "New Folder".to_string(), None);
                        }
                        TreeAction::MoveCharacter(char_id, target_id) => {
                            app.move_character(char_id, target_id);
                        }
                        TreeAction::RequestDeleteCharacter(char_id) => {
                            if let Some(c) = app.characters.iter().find(|c| c.id == char_id) {
                                app.popup_state =
                                    crate::ui::PopupState::DeleteCharacterConfirmation {
                                        id: char_id,
                                        name: c.name.clone(),
                                    };
                            }
                        }
                        TreeAction::SwitchToAll => {
                            app.request_view_all();
                        }
                        TreeAction::CreateNewCharacter(target_coll_id) => {
                            app.create_new_character(target_coll_id.or(app.selected_collection_id));
                        }
                    }
                }

                // Bottom: Add Buttons
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        if ui.button("➕ New Character").clicked() {
                            app.create_new_character(app.selected_collection_id);
                        }
                        if ui.button("➕ New Lorebook").clicked() {
                            app.selected_lorebook = Some(crate::models::Lorebook::default());
                            app.mode = AppMode::Lorebooks;
                        }
                    });
                    if app.mode == AppMode::Characters {
                        if ui.button("📁 New Root Folder").clicked() {
                            app.save_collection(0, "New Folder".to_string(), None);
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
    SwitchToAll,
    MoveCharacter(i64, Option<i64>),
    RequestDeleteCharacter(i64),
    CreateNewCharacter(Option<i64>),
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
    sort_direction: SortDirection,
    search_query: &str,
) {
    let query_lower = search_query.to_lowercase();
    let is_search_active = !search_query.is_empty();

    // 1. Render Sub-collections
    let node_colls: Vec<&Collection> = collections
        .iter()
        .filter(|c| c.parent_id == parent_id)
        .collect();
    for col in node_colls {
        let has_visible_descendants = if is_search_active {
            has_matches(col.id, collections, characters, &query_lower)
        } else {
            true
        };

        let is_selected = Some(col.id) == selected_coll_id;

        // Auto-expand if this collection is an ancestor of the selected one
        let mut is_ancestor = false;
        if let Some(sid) = selected_coll_id {
            let mut curr = sid;
            while let Some(parent) = collections
                .iter()
                .find(|c| c.id == curr)
                .and_then(|c| c.parent_id)
            {
                if parent == col.id {
                    is_ancestor = true;
                    break;
                }
                curr = parent;
            }
        }

        let id_str = ui.make_persistent_id(col.id);
        let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            id_str,
            false,
        );
        let was_open = state.is_open();

        if (is_search_active && has_visible_descendants) || is_ancestor {
            state.set_open(true);
        }

        let mut toggle = false;
        let header_res = state.show_header(ui, |ui| {
            let alpha = if has_visible_descendants { 255 } else { 100 };
            let text_color = if is_selected {
                egui::Color32::WHITE
            } else {
                ui.visuals()
                    .text_color()
                    .linear_multiply(alpha as f32 / 255.0)
            };

            let label = egui::RichText::new(format!("📁 {}", col.name))
                .strong()
                .color(text_color);
            let mut response = ui.selectable_label(is_selected, label);

            if !has_visible_descendants {
                response = response.on_hover_text("No matching characters in this folder");
            }

            if response.clicked() {
                actions.push(TreeAction::SelectCollection(col.id));
                toggle = true;
            }

            // Drag and Drop Target
            if let Some(_) = response.dnd_hover_payload::<i64>() {
                ui.painter().rect_stroke(
                    response.rect,
                    2.0,
                    egui::Stroke::new(2.0, egui::Color32::GREEN),
                );
            }

            if let Some(dropped_id) = response.dnd_release_payload::<i64>() {
                actions.push(TreeAction::MoveCharacter(*dropped_id, Some(col.id)));
                // If dropped, we might want to make sure we don't accidentally toggle if we clicked?
                // But dnd release usually doesn't trigger clicked.
                // However, we set toggle = true above if clicked.
                // Let's ensure toggle logic is safe.
                // If dnd release happens, clicked() should be false for standard buttons,
                // but selectable_label might be tricky.
                toggle = false;
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
                if ui.button("➕ New Character").clicked() {
                    actions.push(TreeAction::CreateNewCharacter(Some(col.id)));
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
            render_tree(
                ui,
                collections,
                characters,
                Some(col.id),
                selected_char_id,
                selected_coll_id,
                actions,
                sort_mode,
                sort_direction,
                search_query,
            );
        });

        if toggle {
            if let Some(mut state) =
                egui::collapsing_header::CollapsingState::load(ui.ctx(), id_str)
            {
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
    let mut node_chars: Vec<&Character> = characters
        .iter()
        .filter(|c| c.collection_id == parent_id)
        .collect();

    if is_search_active {
        node_chars.retain(|c| {
            let in_name = c.name.to_lowercase().contains(&query_lower);
            let in_title = c.char_title.to_lowercase().contains(&query_lower);
            let in_app_tags = c
                .app_tags
                .iter()
                .any(|t| t.name.to_lowercase().contains(&query_lower));
            let in_ext_tags = c
                .external_tags
                .iter()
                .any(|t| t.name.to_lowercase().contains(&query_lower));
            in_name || in_title || in_app_tags || in_ext_tags
        });
    }

    match sort_mode {
        SortMode::Alphabetical => {
            node_chars.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        }
        SortMode::NewestFirst => node_chars.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
        SortMode::RecentlyUpdated => node_chars.sort_by(|a, b| b.updated_at.cmp(&a.updated_at)),
    }

    if sort_direction == SortDirection::Descending {
        node_chars.reverse();
    }

    for char in node_chars {
        let is_selected = Some(char.id) == selected_char_id;

        let item_height = 48.0;
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), item_height),
            egui::Sense::click_and_drag(),
        );

        if response.clicked() {
            actions.push(TreeAction::SelectChar(char.clone()));
        }

        // Cursor change on hover removed per user request.
        // if response.hovered() {
        //    ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        // }

        if response.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            response.dnd_set_drag_payload(char.id);

            // Tooltip removed to fix compilation error. Cursor icon provides feedback.
        }

        if is_selected {
            ui.painter()
                .rect_filled(rect, 4.0, ui.visuals().selection.bg_fill);
        } else if response.hovered() {
            ui.painter()
                .rect_filled(rect, 4.0, ui.visuals().widgets.hovered.bg_fill);
        }

        let thumb_size = 40.0;
        let thumb_rect = egui::Rect::from_min_size(
            rect.min + egui::vec2(4.0, 4.0),
            egui::vec2(thumb_size, thumb_size),
        );

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

            crate::ui::widgets::paint_avatar_crop(ui, thumb_rect, &uri, 4.0);
        } else {
            ui.painter()
                .rect_filled(thumb_rect, 4.0, egui::Color32::from_gray(70));
            let initial = char
                .name
                .chars()
                .next()
                .unwrap_or('?')
                .to_uppercase()
                .to_string();
            ui.painter().text(
                thumb_rect.center(),
                egui::Align2::CENTER_CENTER,
                initial,
                egui::FontId::proportional(20.0),
                egui::Color32::WHITE,
            );
        }

        let text_left = thumb_rect.max.x + 8.0;
        let name_font = egui::FontId::proportional(15.0);
        let name_color = if is_selected {
            egui::Color32::WHITE
        } else {
            ui.visuals().text_color()
        };

        let name_galley = ui
            .painter()
            .layout_no_wrap(char.name.clone(), name_font, name_color);
        let name_pos = egui::pos2(text_left, rect.min.y + 4.0);

        ui.painter()
            .with_clip_rect(rect)
            .galley(name_pos, name_galley, egui::Color32::WHITE);

        if !char.char_title.is_empty() {
            let title_font = egui::FontId::proportional(12.0);
            let title_color = if is_selected {
                egui::Color32::from_white_alpha(200)
            } else {
                ui.visuals().text_color().linear_multiply(0.7)
            };

            let title_galley =
                ui.painter()
                    .layout_no_wrap(char.char_title.clone(), title_font, title_color);
            let title_pos = egui::pos2(text_left, rect.min.y + 24.0);

            ui.painter()
                .with_clip_rect(rect)
                .galley(title_pos, title_galley, egui::Color32::WHITE);
        }

        response.context_menu(|ui| {
            ui.menu_button("Move to...", |ui| {
                if ui.button("📁 Uncategorized").clicked() {
                    actions.push(TreeAction::MoveCharacter(char.id, None));
                    ui.close_menu();
                }
                ui.separator();
                // Recursive helper to render collection options
                fn render_collection_options(
                    ui: &mut egui::Ui,
                    collections: &[Collection],
                    parent_id: Option<i64>,
                    actions: &mut Vec<TreeAction>,
                    char_id: i64,
                ) {
                    for col in collections.iter().filter(|c| c.parent_id == parent_id) {
                        ui.menu_button(format!("📁 {}", col.name), |ui| {
                            if ui.button("Move Here").clicked() {
                                actions.push(TreeAction::MoveCharacter(char_id, Some(col.id)));
                                ui.close_menu();
                            }
                            render_collection_options(
                                ui,
                                collections,
                                Some(col.id),
                                actions,
                                char_id,
                            );
                        });
                    }
                }
                render_collection_options(ui, collections, None, actions, char.id);
            });

            ui.separator();

            if ui.button("Delete").clicked() {
                actions.push(TreeAction::RequestDeleteCharacter(char.id));
                ui.close_menu();
            }
        });
    }
}

pub fn has_matches(
    collection_id: i64,
    collections: &[Collection],
    characters: &[Character],
    query: &str,
) -> bool {
    if characters.iter().any(|c| {
        c.collection_id == Some(collection_id) && {
            let name_match = c.name.to_lowercase().contains(query);
            let title_match = c.char_title.to_lowercase().contains(query);
            let app_tag_match = c
                .app_tags
                .iter()
                .any(|t| t.name.to_lowercase().contains(query));
            let ext_tag_match = c
                .external_tags
                .iter()
                .any(|t| t.name.to_lowercase().contains(query));
            name_match || title_match || app_tag_match || ext_tag_match
        }
    }) {
        return true;
    }

    let sub_colls: Vec<&Collection> = collections
        .iter()
        .filter(|c| c.parent_id == Some(collection_id))
        .collect();
    for sub in sub_colls {
        if has_matches(sub.id, collections, characters, query) {
            return true;
        }
    }

    false
}
