mod character_card;
mod collection_card;

pub use character_card::{render_character_card, render_tag_chips};
pub use collection_card::{render_subfolder_card, render_subfolder_list_item};

use crate::ui::{BrowserViewMode, CrapApp, SortDirection, SortMode};
use eframe::egui;

pub enum BrowserAction {
    MoveCharacter(i64, Option<i64>),
    DeleteCharacter(i64),
    RenameCollection(i64, String),
    DeleteCollection(i64),
    CreateCharacter(Option<i64>),
    CreateCollection(Option<i64>),
    ToggleFavorite(i64),
    UpdateCollectionIcon(i64),
    OpenCharacter(i64),
    OpenCollection(i64),
    ExportCollection(i64),
}

pub fn render_collection_move_menu(
    ui: &mut egui::Ui,
    collections: &Vec<crate::models::Collection>,
    parent_id: Option<i64>,
    target_char_id: i64,
    actions: &mut Vec<BrowserAction>,
) {
    let current_level: Vec<&crate::models::Collection> = collections
        .iter()
        .filter(|c| c.parent_id == parent_id)
        .collect();

    for col in current_level {
        ui.menu_button(&col.name, |ui| {
            if ui.button("Move Here").clicked() {
                actions.push(BrowserAction::MoveCharacter(target_char_id, Some(col.id)));
                ui.close_menu();
            }
            render_collection_move_menu(ui, collections, Some(col.id), target_char_id, actions);
        });
    }
}

fn count_recursive(
    collection_id: Option<i64>,
    collections: &[crate::models::Collection],
    characters: &[crate::models::Character],
) -> usize {
    let direct = characters
        .iter()
        .filter(|c| c.collection_id == collection_id)
        .count();
    let sub_folders = collections.iter().filter(|c| c.parent_id == collection_id);
    let mut total = direct;
    for sub in sub_folders {
        total += count_recursive(Some(sub.id), collections, characters);
    }
    total
}

pub fn render_browser_view(app: &mut CrapApp, ui: &mut egui::Ui) {
    let viewing_all = app.viewing_all_characters;
    let collection_id = app.selected_collection_id;
    let mut actions = Vec::new();

    // Background Image
    if app.show_background {
        let bg_path = if app.use_custom_background {
            "data/background/custom.png"
        } else {
            "data/background/default.png"
        };
        let bg_uri = crate::ui::utils::get_image_uri(bg_path);

        // Async load check
        match ui
            .ctx()
            .try_load_image(bg_uri.as_str().into(), Default::default())
        {
            Ok(egui::load::ImagePoll::Ready { image }) => {
                let (img_w, img_h) = (image.size[0] as f32, image.size[1] as f32);
                if img_w > 0.0 && img_h > 0.0 {
                    let rect = ui.available_rect_before_wrap();
                    let avail_w = rect.width();
                    let avail_h = rect.height();

                    let img_aspect = img_w / img_h;
                    let avail_aspect = avail_w / avail_h;

                    // We want to CONTAIN the image (fit inside), so we take the smaller scale
                    // But then we also want it 10% smaller than that, so 0.9 scale.
                    let scale_factor = if avail_aspect > img_aspect {
                        // Available is wider than image, so height is the limiting factor
                        avail_h / img_h
                    } else {
                        // Available is taller than image, so width is the limiting factor
                        avail_w / img_w
                    };

                    let final_scale = scale_factor * app.background_scale;

                    let final_w = img_w * final_scale;
                    let final_h = img_h * final_scale;

                    let center = rect.center();
                    let final_rect =
                        egui::Rect::from_center_size(center, egui::vec2(final_w, final_h));

                    egui::Image::new(bg_uri)
                        .tint(egui::Color32::WHITE.gamma_multiply(0.5))
                        .paint_at(ui, final_rect);
                }
            }
            Ok(egui::load::ImagePoll::Pending { .. }) => {
                // Just wait, egui will repaint when loaded
            }
            _ => {}
        }
    }

    // Clone collections for context menu usage
    let all_collections = app.collections.clone();

    let collection_name = if viewing_all {
        "All Characters (Flat View)".to_string()
    } else if app.viewing_favorites {
        "Favorites".to_string()
    } else if let Some(id) = collection_id {
        app.collections
            .iter()
            .find(|c| c.id == id)
            .map(|c| c.name.clone())
            .unwrap_or("Unknown".to_string())
    } else {
        "Uncategorized".to_string()
    };

    let parent_id = if let Some(id) = collection_id {
        app.collections
            .iter()
            .find(|c| c.id == id)
            .and_then(|c| c.parent_id)
    } else {
        None
    };

    ui.horizontal(|ui| {
        // Back only if in a collection, not in "All" mode which is top level.
        if !viewing_all && collection_id.is_some() {
            let back_btn = ui.button("⬅ Back");
            if back_btn.clicked() {
                app.request_back();
            }
            back_btn.context_menu(|ui| {
                ui.label("Navigation History");
                ui.separator();
                let history_len = app.navigation_history.len();
                let start_index = history_len.saturating_sub(5);
                let history_items: Vec<(usize, String)> = app
                    .navigation_history
                    .iter()
                    .enumerate()
                    .skip(start_index)
                    .rev()
                    .map(|(i, state)| (i, app.describe_state(state)))
                    .collect();

                for (i, label) in history_items {
                    if ui.button(label).clicked() {
                        if app.has_unsaved_changes() {
                            app.popup_state = crate::ui::PopupState::UnsavedChanges {
                                target: crate::ui::AppAction::GoToHistory(i),
                            };
                        } else {
                            app.go_to_history(i);
                        }
                        ui.close_menu();
                    }
                }
                if history_len == 0 {
                    ui.label(egui::RichText::new("No history").italics().weak());
                }
            });
            if ui.button("⬆ Up").clicked() {
                app.request_collection_switch(parent_id);
            }
            // Handle Esc key for Back navigation
            if ui.memory(|m| m.focused().is_none())
                && ui.input(|i| i.key_pressed(egui::Key::Escape))
            {
                app.request_collection_switch(parent_id);
            }
        }
        ui.heading(format!("Browsing: {}", collection_name));

        // Browser Controls (Far Right)
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // 2. Sorting Controls (Far Right)
            let sort_btn = |ui: &mut egui::Ui, app: &mut CrapApp, mode: SortMode, label: &str| {
                let is_selected = app.browser_sort_mode == mode;
                let mut display_label = label.to_string();
                if is_selected {
                    match app.browser_sort_direction {
                        SortDirection::Ascending => display_label.push_str(" v"),
                        SortDirection::Descending => display_label.push_str(" ^"),
                    }
                }

                if ui.selectable_label(is_selected, display_label).clicked() {
                    if is_selected {
                        app.browser_sort_direction = match app.browser_sort_direction {
                            SortDirection::Ascending => SortDirection::Descending,
                            SortDirection::Descending => SortDirection::Ascending,
                        };
                    } else {
                        app.browser_sort_mode = mode;
                        app.browser_sort_direction = SortDirection::Ascending;
                    }
                }
            };

            sort_btn(ui, app, SortMode::RecentlyUpdated, "Upd");
            sort_btn(ui, app, SortMode::NewestFirst, "New");
            sort_btn(ui, app, SortMode::Alphabetical, "A-Z");

            ui.label("Sort:");

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            ui.add_space(8.0);
            ui.add_space(8.0);

            egui::ComboBox::from_id_salt("view_mode_selector")
                .selected_text(match app.browser_view_mode {
                    BrowserViewMode::Grid => "View: Grid",
                    BrowserViewMode::List => "View: List",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut app.browser_view_mode, BrowserViewMode::Grid, "Grid");
                    ui.selectable_value(&mut app.browser_view_mode, BrowserViewMode::List, "List");
                });
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // 1. Rename Button (To the left of Sorting)
            if let Some(id) = collection_id {
                if ui.button("✏ Rename Folder").clicked() {
                    let current_name = app
                        .collections
                        .iter()
                        .find(|c| c.id == id)
                        .map(|c| c.name.clone())
                        .unwrap_or_default();
                    app.popup_state = crate::ui::PopupState::Renaming {
                        id,
                        name: current_name,
                    };
                }
                if ui.button("📤 Export Collection").clicked() {
                    app.popup_state =
                        crate::ui::PopupState::ExportCollectionOptions { collection_id: id };
                }
            } else {
                // Root View: Show DB Management
                if collection_id.is_none() {
                    if ui.button("📥 Import DB").clicked() {
                        app.popup_state = crate::ui::PopupState::ImportDbWarning;
                    }
                    if ui.button("📤 Export DB").clicked() {
                        app.popup_state = crate::ui::PopupState::ExportDbSelection;
                    }
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);
                }
            }
        });
    });
    ui.add_space(10.0);

    let mut subfolders: Vec<crate::models::Collection> = if viewing_all || app.viewing_favorites {
        Vec::new()
    } else {
        app.collections
            .iter()
            .filter(|c| c.parent_id == collection_id)
            .cloned()
            .collect()
    };

    // Sort subfolders
    match app.browser_sort_mode {
        SortMode::Alphabetical => {
            subfolders.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        }
        SortMode::NewestFirst | SortMode::RecentlyUpdated => {
            subfolders.sort_by(|a, b| b.id.cmp(&a.id))
        } // Fallback for folders
    }
    if app.browser_sort_direction == SortDirection::Descending {
        subfolders.reverse();
    }

    let mut chars: Vec<crate::models::Character> = if viewing_all {
        app.characters.clone()
    } else if app.viewing_favorites {
        app.characters
            .iter()
            .filter(|c| c.is_favorite)
            .cloned()
            .collect()
    } else {
        app.characters
            .iter()
            .filter(|c| c.collection_id == collection_id)
            .cloned()
            .collect()
    };

    // Sort characters
    match app.browser_sort_mode {
        SortMode::Alphabetical => {
            chars.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        }
        SortMode::NewestFirst => chars.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
        SortMode::RecentlyUpdated => chars.sort_by(|a, b| b.updated_at.cmp(&a.updated_at)),
    }
    if app.browser_sort_direction == SortDirection::Descending {
        chars.reverse();
    }

    // --- Character Counter Calculations ---
    let direct_count = chars.len();

    let total_count = if viewing_all {
        app.characters.len()
    } else {
        count_recursive(collection_id, &app.collections, &app.characters)
    };

    let counter_text = if direct_count == total_count {
        format!("Characters: {}", direct_count)
    } else {
        format!("Characters: {} ({})", direct_count, total_count)
    };

    if chars.is_empty() && subfolders.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            let _response = ui.label(
                egui::RichText::new("No characters or subfolders in this collection")
                    .size(18.0)
                    .color(egui::Color32::GRAY),
            );
            ui.add_space(10.0);
            if ui
                .button(egui::RichText::new("➕ Add New Character here").size(16.0))
                .clicked()
            {
                app.create_new_character(collection_id);
            }

            // Add context menu to the whole empty area
            let (_rect, resp) = ui.allocate_at_least(ui.available_size(), egui::Sense::click());
            resp.context_menu(|ui| {
                if ui.button("➕ New Character").clicked() {
                    actions.push(BrowserAction::CreateCharacter(collection_id));
                    ui.close_menu();
                }
                if ui.button("📁 New Folder").clicked() {
                    actions.push(BrowserAction::CreateCollection(collection_id));
                    ui.close_menu();
                }
            });
        });

        // Process actions if any (though empty state might not trigger many)
        for action in actions {
            match action {
                BrowserAction::CreateCharacter(cid) => {
                    app.create_new_character(cid);
                }
                BrowserAction::CreateCollection(cid) => {
                    app.save_collection(0, "New Folder".to_string(), cid);
                }
                _ => {}
            }
        }

        // Render counter even when empty
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(counter_text.clone())
                        .size(12.0)
                        .color(egui::Color32::GRAY),
                );
            });
        });

        return;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Context menu for the content area (handles gaps and right side)
        // We use a stateful approach: store the rect from the previous frame and interact with it *before* drawing content.
        // This ensures the interaction is added first (logically behind), so buttons added later will sit on top and capture their own clicks.
        let bg_id = ui.make_persistent_id("browser_content_bg");
        let cached_bg_rect = ui
            .data(|d| d.get_temp::<egui::Rect>(bg_id))
            .unwrap_or(egui::Rect::ZERO);

        if cached_bg_rect.width() > 0.0 && cached_bg_rect.height() > 0.0 {
            let bg_response = ui.interact(cached_bg_rect, bg_id, egui::Sense::click());
            bg_response.context_menu(|ui| {
                if ui.button("➕ New Character").clicked() {
                    actions.push(BrowserAction::CreateCharacter(collection_id));
                    ui.close_menu();
                }
                if ui.button("📁 New Folder").clicked() {
                    actions.push(BrowserAction::CreateCollection(collection_id));
                    ui.close_menu();
                }
            });
        }

        let content_response = egui::Frame::none()
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                if app.browser_view_mode == BrowserViewMode::Grid {
                    // GRID VIEW (Standard)
                    ui.horizontal_wrapped(|ui| {
                        for folder in &subfolders {
                            render_subfolder_card(ui, app, folder, &mut actions);
                        }
                        for char in &chars {
                            render_character_card(ui, app, char, &all_collections, &mut actions);
                        }
                    });
                } else {
                    // LIST VIEW
                    ui.vertical(|ui| {
                        for folder in &subfolders {
                            let count =
                                count_recursive(Some(folder.id), &app.collections, &app.characters);
                            render_subfolder_list_item(ui, app, folder, count, &mut actions);
                        }
                        for char in &chars {
                            ui.add_space(8.0);

                            // List Item Hover Interaction
                            let id = ui.make_persistent_id(format!("char_list_{}", char.id));
                            let prev_rect = ui
                                .data(|d| d.get_temp::<egui::Rect>(id))
                                .unwrap_or(egui::Rect::ZERO);

                            let interact_response = if prev_rect.width() > 0.0 {
                                ui.interact(prev_rect, id, egui::Sense::click())
                            } else {
                                // Dummy response for first frame
                                ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
                            };

                            if interact_response.clicked() {
                                actions.push(BrowserAction::OpenCharacter(char.id));
                            }

                            // Determine colors
                            let bg_color = if interact_response.hovered() {
                                ui.visuals().widgets.hovered.bg_fill
                            } else {
                                ui.visuals().widgets.noninteractive.bg_fill
                            };
                            let stroke = if interact_response.hovered() {
                                ui.visuals().widgets.hovered.bg_stroke
                            } else {
                                ui.visuals().widgets.noninteractive.bg_stroke
                            };

                            // Main card container
                            let frame_response = egui::Frame::group(ui.style())
                                .fill(bg_color)
                                .stroke(stroke)
                                .rounding(4.0)
                                .inner_margin(8.0)
                                .show(ui, |ui| {
                                    ui.set_min_width(ui.available_width());
                                    ui.horizontal(|ui| {
                                        // Avatar (Universal for List)
                                        let avatar_size = 80.0;
                                        let (rect, response) = ui.allocate_exact_size(
                                            egui::vec2(avatar_size, avatar_size),
                                            egui::Sense::click(),
                                        );

                                        // Culling
                                        if !ui.is_rect_visible(rect) {
                                            // Skip painting
                                        } else {
                                            if response.clicked() {
                                                actions.push(BrowserAction::OpenCharacter(char.id));
                                            }

                                            // Avatar Painting
                                            if let Some(path_str) = &char.avatar_path {
                                                let uri = crate::ui::utils::get_image_uri(path_str);
                                                crate::ui::widgets::paint_avatar_crop(
                                                    ui, rect, &uri, 4.0,
                                                );
                                            } else {
                                                ui.painter().rect_filled(
                                                    rect,
                                                    4.0,
                                                    egui::Color32::from_gray(60),
                                                );
                                                let initial = char
                                                    .name
                                                    .chars()
                                                    .next()
                                                    .unwrap_or('?')
                                                    .to_uppercase()
                                                    .to_string();
                                                ui.painter().text(
                                                    rect.center(),
                                                    egui::Align2::CENTER_CENTER,
                                                    initial,
                                                    egui::FontId::proportional(32.0),
                                                    egui::Color32::WHITE,
                                                );
                                            }
                                        }

                                        ui.add_space(10.0);

                                        // Info Vertical
                                        ui.vertical(|ui| {
                                            ui.heading(&char.name);
                                            ui.add_space(4.0);

                                            // LIST VIEW CONTENT (Title + URLs)
                                            if !char.char_title.is_empty() {
                                                ui.label(
                                                    egui::RichText::new(&char.char_title)
                                                        .strong()
                                                        .color(egui::Color32::LIGHT_GRAY),
                                                );
                                            }

                                            ui.add_space(4.0);

                                            if char.urls.is_empty() {
                                                ui.label(
                                                    egui::RichText::new("No URLs")
                                                        .italics()
                                                        .color(egui::Color32::GRAY),
                                                );
                                            }

                                            for url in &char.urls {
                                                ui.horizontal(|ui| {
                                                    let label =
                                                        url.label.as_deref().unwrap_or("Link");
                                                    ui.label(
                                                        egui::RichText::new(format!("{}:", label))
                                                            .strong(),
                                                    );
                                                    let resp = ui.hyperlink(&url.url);
                                                    resp.context_menu(|ui| {
                                                        if ui.button("📋 Copy URL").clicked() {
                                                            ui.output_mut(|o| {
                                                                o.copied_text = url.url.clone()
                                                            });
                                                            ui.close_menu();
                                                        }
                                                    });
                                                });
                                            }

                                            ui.add_space(8.0);

                                            // Character Tags
                                            if !char.app_tags.is_empty() {
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        egui::RichText::new("App Tags:")
                                                            .small()
                                                            .strong(),
                                                    );
                                                    render_tag_chips(ui, &char.app_tags, false);
                                                });
                                            }
                                            if !char.external_tags.is_empty() {
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        egui::RichText::new("External Tags:")
                                                            .small()
                                                            .strong(),
                                                    );
                                                    render_tag_chips(ui, &char.external_tags, true);
                                                });
                                            }

                                            ui.add_space(8.0);

                                            // Token & Char Counts
                                            app.ensure_token_count(char);
                                            if let Some((tokens, chars)) =
                                                app.token_cache.get(&char.id)
                                            {
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "Tokens: {} | Chars: {}",
                                                        tokens, chars
                                                    ))
                                                    .size(11.0)
                                                    .color(egui::Color32::from_gray(100)),
                                                );
                                            } else {
                                                ui.label(
                                                    egui::RichText::new("Calculating...")
                                                        .size(11.0)
                                                        .italics()
                                                        .color(egui::Color32::from_gray(80)),
                                                );
                                            }
                                        });
                                    });
                                })
                                .response;

                            // We need to account for the potential space added before the frame
                            // The rect we want is the frame's rect, which includes the inner margin and content.
                            // However, we want the hover area to include the outer margin if possible, or at least be stable.
                            // frame_response.rect is good.
                            ui.data_mut(|d| d.insert_temp(id, frame_response.rect));
                        }
                    });
                }
            })
            .response;

        // Store the current frame's rect for the next frame's background interaction
        ui.data_mut(|d| d.insert_temp(bg_id, content_response.rect));

        // Context menu for empty space (handles bottom area)
        let available = ui.available_size();
        let (_rect, response) = ui.allocate_at_least(available, egui::Sense::click());
        response.context_menu(|ui| {
            if ui.button("➕ New Character").clicked() {
                actions.push(BrowserAction::CreateCharacter(collection_id));
                ui.close_menu();
            }
            if ui.button("📁 New Folder").clicked() {
                actions.push(BrowserAction::CreateCollection(collection_id));
                ui.close_menu();
            }
        });
    });

    // Handle Actions
    for action in actions {
        match action {
            BrowserAction::MoveCharacter(char_id, target_id) => {
                app.move_character(char_id, target_id);
            }
            BrowserAction::ToggleFavorite(char_id) => {
                app.toggle_favorite(char_id);
            }
            BrowserAction::DeleteCharacter(id) => {
                let name = app
                    .characters
                    .iter()
                    .find(|c| c.id == id)
                    .map(|c| c.name.clone())
                    .unwrap_or_default();
                app.popup_state = crate::ui::PopupState::DeleteCharacterConfirmation { id, name };
            }
            BrowserAction::RenameCollection(id, name) => {
                app.popup_state = crate::ui::PopupState::Renaming { id, name };
            }
            BrowserAction::DeleteCollection(id) => {
                // Calculate count for warning
                let count = app
                    .collections
                    .iter()
                    .filter(|c| c.parent_id == Some(id))
                    .count()
                    + app
                        .characters
                        .iter()
                        .filter(|c| c.collection_id == Some(id))
                        .count();

                if count > 0 {
                    app.popup_state = crate::ui::PopupState::DeleteWarning { _id: id, count };
                } else {
                    app.delete_collection(id);
                }
            }
            BrowserAction::CreateCharacter(cid) => {
                app.create_new_character(cid);
            }
            BrowserAction::CreateCollection(cid) => {
                app.save_collection(0, "New Folder".to_string(), cid);
            }
            BrowserAction::UpdateCollectionIcon(id) => {
                app.popup_state = crate::ui::PopupState::CollectionIconConfirmation {
                    id,
                    path: String::new(),
                    _preview_texture: None,
                };
            }
            BrowserAction::OpenCharacter(id) => {
                app.load_character(id);
            }
            BrowserAction::OpenCollection(id) => {
                app.request_collection_switch(Some(id));
            }
            BrowserAction::ExportCollection(id) => {
                app.popup_state =
                    crate::ui::PopupState::ExportCollectionOptions { collection_id: id };
            }
        }
    }

    ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(counter_text)
                    .size(12.0)
                    .color(egui::Color32::GRAY),
            );
        });
    });
}
