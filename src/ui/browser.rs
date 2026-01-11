use crate::models::Tag;
use crate::ui::{CrapApp, SortDirection, SortMode};
use eframe::egui;

pub enum BrowserAction {
    MoveCharacter(i64, Option<i64>),
    DeleteCharacter(i64),
    RenameCollection(i64, String),
    DeleteCollection(i64),
    CreateCharacter(Option<i64>),
    CreateCollection(Option<i64>),
    ToggleFavorite(i64),
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

pub fn render_browser_view(app: &mut CrapApp, ui: &mut egui::Ui) {
    let viewing_all = app.viewing_all_characters;
    let collection_id = app.selected_collection_id;
    let mut actions = Vec::new();

    // Background Image
    if let Ok(abs_path) = std::fs::canonicalize("data/background/default.png") {
        // Attempt to read dimensions to calculate aspect ratio
        if let Ok(reader) = image::io::Reader::open(&abs_path) {
            if let Ok(dims) = reader.into_dimensions() {
                let (img_w, img_h) = dims;
                if img_w > 0 && img_h > 0 {
                    let uri = format!("file://{}", abs_path.to_string_lossy());
                    let rect = ui.available_rect_before_wrap();

                    let avail_w = rect.width();
                    let avail_h = rect.height();

                    let img_aspect = img_w as f32 / img_h as f32;
                    let avail_aspect = avail_w / avail_h;

                    // We want to CONTAIN the image (fit inside), so we take the smaller scale
                    // But then we also want it 10% smaller than that, so 0.9 scale.
                    let scale_factor = if avail_aspect > img_aspect {
                        // Available is wider than image, so height is the limiting factor
                        avail_h / img_h as f32
                    } else {
                        // Available is taller than image, so width is the limiting factor
                        avail_w / img_w as f32
                    };

                    let final_scale = scale_factor * 0.9;

                    let final_w = img_w as f32 * final_scale;
                    let final_h = img_h as f32 * final_scale;

                    let center = rect.center();
                    let final_rect =
                        egui::Rect::from_center_size(center, egui::vec2(final_w, final_h));

                    egui::Image::new(uri)
                        .tint(egui::Color32::WHITE.gamma_multiply(0.5))
                        .paint_at(ui, final_rect);
                }
            }
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
        "Uncategorized (Root)".to_string()
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
            if ui.button("⬅ Back").clicked() {
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
            let label = if app.browser_show_urls {
                "URLs (On)"
            } else {
                "URLs"
            };
            if ui.selectable_label(app.browser_show_urls, label).clicked() {
                app.browser_show_urls = !app.browser_show_urls;
            }
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
            } else {
                // Root View: Show DB Management
                if collection_id.is_none() {
                    if ui.button("📥 Import DB").clicked() {
                        app.popup_state = crate::ui::PopupState::ImportDbWarning;
                    }
                    if ui.button("📤 Export DB").clicked() {
                        app.trigger_db_export();
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
                if app.browser_show_urls {
                    // LIST VIEW (URLs)
                    // Keep folders separate at the top
                    ui.horizontal_wrapped(|ui| {
                        for folder in &subfolders {
                            render_subfolder_card(ui, app, folder, &mut actions);
                        }
                    });

                    ui.vertical(|ui| {
                        for char in &chars {
                            ui.add_space(8.0);
                            egui::Frame::group(ui.style()).show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    // Avatar
                                    let avatar_size = 80.0;
                                    let (rect, response) = ui.allocate_exact_size(
                                        egui::vec2(avatar_size, avatar_size),
                                        egui::Sense::click(),
                                    );

                                    if response.clicked() {
                                        app.selected_character = Some(char.clone());
                                        app.central_view = crate::ui::CentralView::Editor;
                                        app.load_tags(char.id);
                                        app.load_links(char.id);
                                    }

                                    // Avatar Painting
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
                                        crate::ui::widgets::paint_avatar_crop(ui, rect, &uri, 4.0);
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

                                    ui.add_space(10.0);

                                    // Info Vertical
                                    ui.vertical(|ui| {
                                        ui.heading(&char.name);
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
                                                let label = url.label.as_deref().unwrap_or("Link");
                                                ui.label(
                                                    egui::RichText::new(format!("{}:", label))
                                                        .strong(),
                                                );
                                                ui.hyperlink(&url.url);
                                            });
                                        }
                                    });
                                });
                            });
                        }
                    });
                } else {
                    // GRID VIEW
                    // Mix folders and characters in one flow
                    ui.horizontal_wrapped(|ui| {
                        for folder in &subfolders {
                            render_subfolder_card(ui, app, folder, &mut actions);
                        }
                        for char in &chars {
                            render_character_card(ui, app, char, &all_collections, &mut actions);
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
                    app.popup_state = crate::ui::PopupState::DeleteWarning { id, count };
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

pub fn render_character_card(
    ui: &mut egui::Ui,
    app: &mut CrapApp,
    char: &crate::models::Character,
    all_collections: &Vec<crate::models::Collection>,
    actions: &mut Vec<BrowserAction>,
) {
    let card_width = 180.0;
    let card_height = 260.0;

    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(card_width, card_height), egui::Sense::click());

    // Hover Effect
    let bg_color = if response.hovered() {
        ui.visuals().widgets.hovered.bg_fill
    } else {
        ui.visuals().widgets.noninteractive.bg_fill
    };

    ui.painter().rect_filled(rect, 8.0, bg_color);
    ui.painter()
        .rect_stroke(rect, 8.0, ui.visuals().widgets.noninteractive.bg_stroke);

    // Interaction
    if response.clicked() {
        app.selected_character = Some(char.clone());
        app.central_view = crate::ui::CentralView::Editor;
        app.load_tags(char.id);
        app.load_links(char.id);
        // FORCE SWITCH MODE to Characters so sidebar and other UI elements align
        app.mode = crate::ui::AppMode::Characters;
    }

    response.context_menu(|ui| {
        ui.menu_button("Move to...", |ui| {
            if ui.button("Root (Uncategorized)").clicked() {
                actions.push(BrowserAction::MoveCharacter(char.id, None));
                ui.close_menu();
            }
            ui.separator();
            render_collection_move_menu(ui, all_collections, None, char.id, actions);
        });

        ui.separator();
        let fav_label = if char.is_favorite {
            "Remove from Favorites"
        } else {
            "Add to Favorites"
        };
        if ui.button(fav_label).clicked() {
            actions.push(BrowserAction::ToggleFavorite(char.id));
            ui.close_menu();
        }
        ui.separator();

        if ui.button("🗑 Delete").clicked() {
            actions.push(BrowserAction::DeleteCharacter(char.id));
            ui.close_menu();
        }
    });

    // Content
    let content_rect = rect.shrink(8.0);

    // Avatar (Top Square)
    let avatar_size = content_rect.width();
    let avatar_rect =
        egui::Rect::from_min_size(content_rect.min, egui::vec2(avatar_size, avatar_size));

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
        crate::ui::widgets::paint_avatar_crop(ui, avatar_rect, &uri, 4.0);
    } else {
        ui.painter()
            .rect_filled(avatar_rect, 4.0, egui::Color32::from_gray(60));
        let initial = char
            .name
            .chars()
            .next()
            .unwrap_or('?')
            .to_uppercase()
            .to_string();
        ui.painter().text(
            avatar_rect.center(),
            egui::Align2::CENTER_CENTER,
            initial,
            egui::FontId::proportional(40.0),
            egui::Color32::WHITE,
        );
    }

    // Watermark
    if char.is_favorite {
        ui.painter().text(
            if rect.max.x - 8.0 >= rect.min.x && rect.min.y + 32.0 <= rect.max.y {
                egui::pos2(rect.max.x - 8.0, rect.min.y + 8.0)
            } else {
                rect.min
            },
            egui::Align2::RIGHT_TOP,
            "\u{2764}",
            egui::FontId::proportional(20.0),
            egui::Color32::WHITE,
        );
    }

    // Text Area
    let text_top = avatar_rect.max.y + 8.0;
    let _text_rect =
        egui::Rect::from_min_max(egui::pos2(content_rect.min.x, text_top), content_rect.max);

    let mut cursor_y = text_top;

    // Name
    let name_font = egui::FontId::proportional(16.0);
    let name_galley = ui.painter().layout_no_wrap(
        char.name.clone(),
        name_font.clone(),
        ui.visuals().text_color(),
    );
    ui.painter().galley(
        egui::pos2(content_rect.min.x, cursor_y),
        name_galley,
        ui.visuals().text_color(),
    );
    cursor_y += 20.0;

    // Title
    if !char.char_title.is_empty() {
        let title_font = egui::FontId::proportional(12.0);
        let title_galley = ui.painter().layout_no_wrap(
            char.char_title.clone(),
            title_font,
            ui.visuals().text_color().linear_multiply(0.7),
        );
        ui.painter().with_clip_rect(rect).galley(
            egui::pos2(content_rect.min.x, cursor_y),
            title_galley,
            ui.visuals().text_color(),
        );
        cursor_y += 16.0;
    } else {
        cursor_y += 16.0; // Spacer
    }

    cursor_y += 4.0;

    // Tags (Chips)
    let tag_font = egui::FontId::proportional(10.0);
    let mut tag_x = content_rect.min.x;

    let mut tags_to_show: Vec<&Tag> = char.app_tags.iter().collect();
    let mut is_external = false;
    if tags_to_show.is_empty() {
        tags_to_show = char.external_tags.iter().collect();
        is_external = true;
    }

    for tag in tags_to_show.iter().take(3) {
        let tag_galley =
            ui.painter()
                .layout_no_wrap(tag.name.clone(), tag_font.clone(), egui::Color32::WHITE);
        let pad = 4.0;
        let chip_w = tag_galley.rect.width() + pad * 2.0;

        if tag_x + chip_w > content_rect.max.x {
            break;
        }

        let chip_rect =
            egui::Rect::from_min_size(egui::pos2(tag_x, cursor_y), egui::vec2(chip_w, 16.0));

        // Different color for external tags (Grayish vs Blueish)
        let bg_color = if is_external {
            egui::Color32::from_rgb(100, 100, 100)
        } else {
            egui::Color32::from_rgb(50, 80, 150)
        };

        ui.painter().rect_filled(chip_rect, 8.0, bg_color);
        ui.painter().galley(
            egui::pos2(tag_x + pad, cursor_y + 2.0),
            tag_galley,
            egui::Color32::WHITE,
        );

        tag_x += chip_w + 4.0;
    }
}

pub fn render_subfolder_card(
    ui: &mut egui::Ui,
    app: &mut CrapApp,
    folder: &crate::models::Collection,
    actions: &mut Vec<BrowserAction>,
) {
    let card_width = 180.0;
    let card_height = 260.0;

    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(card_width, card_height), egui::Sense::click());

    let bg_color = if response.hovered() {
        ui.visuals().widgets.hovered.bg_fill
    } else {
        ui.visuals().widgets.noninteractive.bg_fill
    };

    ui.painter().rect_filled(rect, 8.0, bg_color);
    ui.painter()
        .rect_stroke(rect, 1.0, ui.visuals().widgets.noninteractive.bg_stroke);

    if response.clicked() {
        app.selected_collection_id = Some(folder.id);
    }

    response.context_menu(|ui| {
        if ui.button("✏ Rename").clicked() {
            actions.push(BrowserAction::RenameCollection(
                folder.id,
                folder.name.clone(),
            ));
            ui.close_menu();
        }
        if ui.button("🗑 Delete").clicked() {
            actions.push(BrowserAction::DeleteCollection(folder.id));
            ui.close_menu();
        }
    });

    // Content
    let content_rect = rect.shrink(8.0);

    // Avatar (Top Square - Folder Icon)
    let avatar_size = content_rect.width();
    let avatar_rect =
        egui::Rect::from_min_size(content_rect.min, egui::vec2(avatar_size, avatar_size));

    // Draw centered folder icon
    ui.painter()
        .rect_filled(avatar_rect, 4.0, egui::Color32::from_rgb(60, 60, 70)); // Darker bg for folder
    ui.painter().text(
        avatar_rect.center(),
        egui::Align2::CENTER_CENTER,
        "📁",
        egui::FontId::proportional(64.0),
        egui::Color32::from_rgb(200, 200, 220),
    );

    // Text Area (Name)
    let text_top = avatar_rect.max.y + 8.0;
    let name_font = egui::FontId::proportional(16.0);
    let name_galley = ui.painter().layout_no_wrap(
        folder.name.clone(),
        name_font.clone(),
        ui.visuals().text_color(),
    );
    ui.painter().galley(
        egui::pos2(content_rect.min.x, text_top),
        name_galley,
        ui.visuals().text_color(),
    );
}
