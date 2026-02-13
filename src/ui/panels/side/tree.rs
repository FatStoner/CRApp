use crate::models::{Character, Collection, Lorebook};
use crate::ui::{SortDirection, SortMode};
use eframe::egui;

pub enum TreeAction {
    SelectChar(Character),
    SelectCollection(i64),
    DeselectCollection,
    RenameCollection(i64, String),
    RequestDeleteCollection(i64),
    CreateSubfolder(i64),
    CreateRootFolder,
    SwitchToAll,
    SwitchToFavorites,
    MoveCharacter(i64, Option<i64>),
    RequestDeleteCharacter(i64),
    CreateNewCharacter(Option<i64>),
    MoveCollection(i64, bool), // id, move_up
    ToggleFavorite(i64),
    FoldAllFolders,
    UnfoldAllFolders,
    ExportCollection(i64),
    ShowStatisticsCollection(i64),
    ShowStatisticsAll,
    ShowStatisticsFavorites,
}

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
    char_lore_map: &std::collections::HashMap<i64, Vec<i64>>,
    lorebooks: &[Lorebook],
) {
    let query_lower = search_query.to_lowercase();
    let is_search_active = !search_query.is_empty();

    // 1. Render Sub-collections
    let node_colls: Vec<&Collection> = collections
        .iter()
        .filter(|c| c.parent_id == parent_id)
        .collect();
    for col in &node_colls {
        let has_visible_descendants = if is_search_active {
            has_matches(
                col.id,
                collections,
                characters,
                &query_lower,
                char_lore_map,
                lorebooks,
            )
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

        let id_str = egui::Id::new(("folder", col.id));
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

                // Sorting Buttons
                // Only show if possible
                let index = node_colls.iter().position(|c| c.id == col.id).unwrap_or(0);
                if index > 0 {
                    if ui.button("⬆ Move Up").clicked() {
                        actions.push(TreeAction::MoveCollection(col.id, true));
                        ui.close_menu();
                    }
                }
                if index < node_colls.len() - 1 {
                    if ui.button("⬇ Move Down").clicked() {
                        actions.push(TreeAction::MoveCollection(col.id, false));
                        ui.close_menu();
                    }
                }

                ui.separator();

                if ui.button("📁 New Folder").clicked() {
                    actions.push(TreeAction::CreateSubfolder(col.id));
                    ui.close_menu();
                }
                if ui.button("📤 Export Collection").clicked() {
                    actions.push(TreeAction::ExportCollection(col.id));
                    ui.close_menu();
                }
                if ui.button("📊 Statistics").clicked() {
                    actions.push(TreeAction::ShowStatisticsCollection(col.id));
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
                char_lore_map,
                lorebooks,
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

            let in_lore = if let Some(lore_ids) = char_lore_map.get(&c.id) {
                lore_ids.iter().any(|&lid| {
                    lorebooks
                        .iter()
                        .any(|lb| lb.id == lid && lb.title.to_lowercase().contains(&query_lower))
                })
            } else {
                false
            };

            in_name || in_title || in_app_tags || in_ext_tags || in_lore
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

        // Culling: If the item is not visible, skip painting to save huge amounts of resources
        if !ui.is_rect_visible(rect) {
            continue;
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
            let uri = crate::ui::utils::get_image_uri(path_str);

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

        let display_name = if char.is_favorite {
            format!("{} \u{2764}", char.name)
        } else {
            char.name.clone()
        };

        let name_galley = ui
            .painter()
            .layout_no_wrap(display_name, name_font, name_color);
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

            let fav_label = if char.is_favorite {
                "Remove from Favorites"
            } else {
                "Add to Favorites"
            };
            if ui.button(fav_label).clicked() {
                actions.push(TreeAction::ToggleFavorite(char.id));
                ui.close_menu();
            }

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
    char_lore_map: &std::collections::HashMap<i64, Vec<i64>>,
    lorebooks: &[Lorebook],
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

            let lore_match = if let Some(lore_ids) = char_lore_map.get(&c.id) {
                lore_ids.iter().any(|&lid| {
                    lorebooks
                        .iter()
                        .any(|lb| lb.id == lid && lb.title.to_lowercase().contains(query))
                })
            } else {
                false
            };

            name_match || title_match || app_tag_match || ext_tag_match || lore_match
        }
    }) {
        return true;
    }

    let sub_colls: Vec<&Collection> = collections
        .iter()
        .filter(|c| c.parent_id == Some(collection_id))
        .collect();
    for sub in sub_colls {
        if has_matches(
            sub.id,
            collections,
            characters,
            query,
            char_lore_map,
            lorebooks,
        ) {
            return true;
        }
    }

    false
}
