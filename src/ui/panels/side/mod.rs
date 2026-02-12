mod tree;

pub use tree::{has_matches, render_tree, TreeAction};

use crate::models::Lorebook;
use crate::ui::{AppMode, CrapApp, SortDirection, SortMode};
use eframe::egui;

pub fn render_side_panel(app: &mut CrapApp, ctx: &egui::Context) {
    egui::SidePanel::left("side_panel")
        .min_width(250.0)
        .show(ctx, |ui| {
            ui.add_space(4.0);

            // Mode Switcher
            ui.horizontal(|ui| {
                let char_mode = app.mode == AppMode::Characters;
                if ui.selectable_label(char_mode, "Characters").clicked() {
                    if !char_mode {
                        if let Some(last_id) = app.last_active_character_id {
                            app.load_character(last_id);
                        } else if let Some(first) = app.characters.first().map(|c| c.id) {
                            app.load_character(first);
                        } else {
                            app.mode = AppMode::Characters; // Fallback if no characters
                        }
                    }
                }

                let lore_mode = app.mode == AppMode::Lorebooks;
                if ui.selectable_label(lore_mode, "Lorebooks").clicked() {
                    if !lore_mode {
                        if let Some(last_id) = app.last_active_lorebook_id {
                            app.load_lorebook(last_id);
                        } else if let Some(first) = app.lorebooks.first().map(|l| l.id) {
                            app.load_lorebook(first);
                        } else {
                            app.mode = AppMode::Lorebooks; // Fallback
                        }
                    }
                }

                ui.add_space(8.0);
                if ui.button("Options").clicked() {
                    app.show_options_window = true;
                }
            });
            ui.separator();
            if app.mode == AppMode::Characters || app.mode == AppMode::Templates {
                let is_active = app.mode == AppMode::Templates;
                if ui.selectable_label(is_active, "Templates").clicked() {
                    app.request_switch_to_templates();
                }
                ui.separator();
            }
            ui.separator();

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
                        app.perform_deep_search();
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

                            if ui
                                .selectable_label(
                                    app.viewing_favorites,
                                    format!("\u{2764} Favorites"),
                                )
                                .clicked()
                            {
                                if app.viewing_favorites {
                                    // Toggle off -> go to Uncategorized (DeselectCollection logic effectively)
                                    actions.push(TreeAction::DeselectCollection);
                                } else {
                                    actions.push(TreeAction::SwitchToFavorites);
                                }
                            }

                            ui.separator();

                            let is_uncategorized = app.selected_collection_id.is_none()
                                && !app.viewing_all_characters
                                && !app.viewing_favorites;
                            let response =
                                ui.selectable_label(is_uncategorized, "📁 Uncategorized");
                            if response.clicked() {
                                actions.push(TreeAction::DeselectCollection);
                            }

                            response.context_menu(|ui| {
                                if ui.button("Fold all").clicked() {
                                    actions.push(TreeAction::FoldAllFolders);
                                    ui.close_menu();
                                }
                                if ui.button("Unfold all").clicked() {
                                    actions.push(TreeAction::UnfoldAllFolders);
                                    ui.close_menu();
                                }
                            });

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
                            render_tree(
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
                                &app.char_lore_map,
                                &app.lorebooks,
                            );

                            // Fill empty space for context menu
                            let available = ui.available_height();
                            let height = if available.is_finite() && available > 0.0 {
                                available
                            } else {
                                150.0
                            };

                            let (_rect, response) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width(), height),
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
                            let mut lorebook_to_select = None;
                            let mut delete_req = None;

                            for book in &app.lorebooks {
                                let is_selected =
                                    app.selected_lorebook.as_ref().map(|l| l.id) == Some(book.id);
                                let (rect, response) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), 48.0),
                                    egui::Sense::click(),
                                );

                                // Culling
                                if ui.is_rect_visible(rect) {
                                    let bg_color = if is_selected {
                                        ui.visuals().widgets.active.bg_fill
                                    } else if response.hovered() {
                                        ui.visuals().widgets.hovered.bg_fill
                                    } else {
                                        egui::Color32::TRANSPARENT
                                    };

                                    if bg_color != egui::Color32::TRANSPARENT {
                                        // Shrink rect vertically by 2px top/bottom to match character list logic (44px high)
                                        // and create a clear 4px gap between row highlights.
                                        let highlight_rect = rect.shrink2(egui::vec2(0.0, 2.0));
                                        ui.painter().rect_filled(highlight_rect, 4.0, bg_color);
                                    }

                                    ui.allocate_new_ui(
                                        egui::UiBuilder::new().max_rect(rect).layout(
                                            egui::Layout::left_to_right(egui::Align::Center),
                                        ),
                                        |ui| {
                                            ui.add_space(6.0);
                                            // Thumbnail
                                            let thumb_size = 40.0;
                                            let (thumb_rect, _) = ui.allocate_exact_size(
                                                egui::vec2(thumb_size, thumb_size),
                                                egui::Sense::hover(),
                                            );

                                            render_lorebook_thumbnail(ui, book, thumb_rect);

                                            ui.add_space(8.0);

                                            let mut label_text = egui::RichText::new(&book.title);
                                            if is_selected {
                                                label_text = label_text
                                                    .strong()
                                                    .color(egui::Color32::LIGHT_BLUE);
                                            }
                                            // Ensure title is non-selectable and does not block clicks.
                                            ui.add(
                                                egui::Label::new(label_text)
                                                    .selectable(false)
                                                    .sense(egui::Sense::hover()),
                                            );
                                        },
                                    );

                                    response.context_menu(|ui| {
                                        if ui.button("Delete").clicked() {
                                            delete_req = Some((book.id, book.title.clone()));
                                            ui.close_menu();
                                        }
                                    });

                                    if response.clicked() {
                                        lorebook_to_select = Some(book.id);
                                    }
                                }
                            }

                            if let Some(book_id) = lorebook_to_select {
                                app.request_lorebook_switch(book_id);
                            }

                            if let Some((id, title)) = delete_req {
                                app.popup_state =
                                    crate::ui::PopupState::DeleteLorebookConfirmation { id, title };
                            }
                        }
                        AppMode::Templates => {
                            let mut template_to_select = None;
                            let mut delete_req = None;

                            for template in &app.templates {
                                let is_selected = app.selected_template.as_ref().map(|t| t.id)
                                    == Some(template.id);
                                let (rect, response) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), 48.0),
                                    egui::Sense::click(),
                                );

                                if ui.is_rect_visible(rect) {
                                    let bg_color = if is_selected {
                                        ui.visuals().widgets.active.bg_fill
                                    } else if response.hovered() {
                                        ui.visuals().widgets.hovered.bg_fill
                                    } else {
                                        egui::Color32::TRANSPARENT
                                    };

                                    if bg_color != egui::Color32::TRANSPARENT {
                                        let highlight_rect = rect.shrink2(egui::vec2(0.0, 2.0));
                                        ui.painter().rect_filled(highlight_rect, 4.0, bg_color);
                                    }

                                    ui.allocate_new_ui(
                                        egui::UiBuilder::new().max_rect(rect).layout(
                                            egui::Layout::left_to_right(egui::Align::Center),
                                        ),
                                        |ui| {
                                            ui.add_space(6.0);
                                            // Icon
                                            let thumb_size = 40.0;
                                            let (thumb_rect, _) = ui.allocate_exact_size(
                                                egui::vec2(thumb_size, thumb_size),
                                                egui::Sense::hover(),
                                            );

                                            ui.painter().rect_filled(
                                                thumb_rect,
                                                2.0,
                                                egui::Color32::from_gray(60),
                                            );
                                            ui.painter().text(
                                                thumb_rect.center(),
                                                egui::Align2::CENTER_CENTER,
                                                "T",
                                                egui::FontId::proportional(20.0),
                                                egui::Color32::WHITE,
                                            );

                                            ui.add_space(8.0);
                                            let mut label_text =
                                                egui::RichText::new(&template.name);
                                            if is_selected {
                                                label_text = label_text
                                                    .strong()
                                                    .color(egui::Color32::LIGHT_BLUE);
                                            }
                                            ui.add(egui::Label::new(label_text).selectable(false));
                                        },
                                    );

                                    response.context_menu(|ui| {
                                        if ui.button("Delete").clicked() {
                                            delete_req = Some((template.id, template.name.clone()));
                                            ui.close_menu();
                                        }
                                    });

                                    if response.clicked() {
                                        template_to_select = Some(template.id);
                                    }
                                }
                            }

                            if let Some(tid) = template_to_select {
                                app.request_template_switch(tid);
                            }

                            if let Some((id, name)) = delete_req {
                                app.popup_state =
                                    crate::ui::PopupState::DeleteTemplateConfirmation { id, name };
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
                                    _id: id,
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
                        TreeAction::SwitchToFavorites => {
                            // Manual handling since we don't have a helper for this yet in mod.rs or it's simple enough
                            app.viewing_favorites = true;
                            app.viewing_all_characters = false;
                            app.selected_collection_id = None;
                            app.central_view = crate::ui::CentralView::Browser;
                        }
                        TreeAction::CreateNewCharacter(target_coll_id) => {
                            app.create_new_character(target_coll_id.or(app.selected_collection_id));
                        }
                        TreeAction::MoveCollection(id, move_up) => {
                            app.reorder_collection(id, move_up);
                        }
                        TreeAction::ToggleFavorite(id) => {
                            app.toggle_favorite(id);
                        }
                        TreeAction::FoldAllFolders => {
                            // Removed: app.request_collection_switch(None);
                            // We don't want to reset selection, just fold folders.
                            // Note: If the currently selected character is deep in folders,
                            // the auto-expand logic in render_tree will likely re-open
                            // the path to it on the next frame. This is expected behavior.

                            for col in &app.collections {
                                let id = egui::Id::new(("folder", col.id));
                                if let Some(mut state) =
                                    egui::collapsing_header::CollapsingState::load(ctx, id)
                                {
                                    state.set_open(false);
                                    state.store(ctx);
                                }
                            }
                        }
                        TreeAction::UnfoldAllFolders => {
                            for col in &app.collections {
                                let id = egui::Id::new(("folder", col.id));
                                // load_with_default_open returns CollapsingState directly, not Option
                                let mut state =
                                    egui::collapsing_header::CollapsingState::load_with_default_open(
                                        ctx, id, true,
                                    );
                                state.set_open(true);
                                state.store(ctx);
                            }
                        }
                    }
                }

                // Bottom: Add Buttons
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        if ui.button("➕ New Character").clicked() {
                            app.create_new_character(None);
                        }
                        if ui.button("➕ New Lorebook").clicked() {
                            app.create_new_lorebook();
                        }
                    });

                    if app.mode == AppMode::Templates {
                        if ui.button("➕ New Template").clicked() {
                            app.create_new_template();
                        }
                    }
                    if app.mode == AppMode::Characters {
                        if ui.button("📁 New Folder").clicked() {
                            app.save_collection(0, "New Folder".to_string(), None);
                        }
                    }
                });
            }
        });
}

fn render_lorebook_thumbnail(ui: &mut egui::Ui, book: &Lorebook, thumb_rect: egui::Rect) {
    if let Some(path_str) = &book.cover_path {
        let uri = crate::ui::utils::get_image_uri(path_str);
        crate::ui::widgets::paint_avatar_crop(ui, thumb_rect, &uri, 2.0);
    } else {
        ui.painter()
            .rect_filled(thumb_rect, 2.0, egui::Color32::from_gray(60));
        let initial = book
            .title
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
}
