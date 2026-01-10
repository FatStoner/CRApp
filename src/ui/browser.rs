use crate::models::Tag;
use crate::ui::{CrapApp, SortDirection, SortMode};
use eframe::egui;

enum BrowserAction {
    MoveCharacter(i64, Option<i64>),
    DeleteCharacter(i64),
    RenameCollection(i64, String),
    DeleteCollection(i64),
    CreateCharacter(Option<i64>),
    CreateCollection(Option<i64>),
}

fn render_collection_move_menu(
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

    // Clone collections for context menu usage
    let all_collections = app.collections.clone();

    let collection_name = if viewing_all {
        "All Characters (Flat View)".to_string()
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

            // 1. Rename Button (To the left of Sorting)
            if !viewing_all {
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
                }
            }
        });
    });
    ui.add_space(10.0);

    let mut subfolders: Vec<crate::models::Collection> = if viewing_all {
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
        ui.horizontal_wrapped(|ui| {
            // Render Subfolders
            // Render Subfolders
            for folder in subfolders {
                let card_width = 180.0;
                let card_height = 260.0;

                let (rect, response) = ui
                    .allocate_exact_size(egui::vec2(card_width, card_height), egui::Sense::click());

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
                let avatar_rect = egui::Rect::from_min_size(
                    content_rect.min,
                    egui::vec2(avatar_size, avatar_size),
                );

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

            // Render Characters
            for char in chars {
                let card_width = 180.0;
                let card_height = 260.0;

                let (rect, response) = ui
                    .allocate_exact_size(egui::vec2(card_width, card_height), egui::Sense::click());

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
                }

                response.context_menu(|ui| {
                    ui.menu_button("Move to...", |ui| {
                        if ui.button("Root (Uncategorized)").clicked() {
                            actions.push(BrowserAction::MoveCharacter(char.id, None));
                            ui.close_menu();
                        }
                        ui.separator();
                        render_collection_move_menu(
                            ui,
                            &all_collections,
                            None,
                            char.id,
                            &mut actions,
                        );
                    });

                    if ui.button("🗑 Delete").clicked() {
                        actions.push(BrowserAction::DeleteCharacter(char.id));
                        ui.close_menu();
                    }
                });

                // Content
                let content_rect = rect.shrink(8.0);

                // Avatar (Top Square)
                let avatar_size = content_rect.width();
                let avatar_rect = egui::Rect::from_min_size(
                    content_rect.min,
                    egui::vec2(avatar_size, avatar_size),
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

                // Text Area
                let text_top = avatar_rect.max.y + 8.0;
                let _text_rect = egui::Rect::from_min_max(
                    egui::pos2(content_rect.min.x, text_top),
                    content_rect.max,
                );

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
                    let tag_galley = ui.painter().layout_no_wrap(
                        tag.name.clone(),
                        tag_font.clone(),
                        egui::Color32::WHITE,
                    );
                    let pad = 4.0;
                    let chip_w = tag_galley.rect.width() + pad * 2.0;

                    if tag_x + chip_w > content_rect.max.x {
                        break;
                    }

                    let chip_rect = egui::Rect::from_min_size(
                        egui::pos2(tag_x, cursor_y),
                        egui::vec2(chip_w, 16.0),
                    );

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
        });

        // Context menu for empty space
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
