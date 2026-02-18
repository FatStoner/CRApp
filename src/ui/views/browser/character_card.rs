use crate::models::Tag;

use eframe::egui;

use super::render_collection_move_menu;

pub fn render_character_card(
    ui: &mut egui::Ui,
    char: &crate::models::Character,
    all_collections: &Vec<crate::models::Collection>,
    actions: &mut Vec<crate::ui::browser::BrowserAction>,
    blur_all_images: bool,
    blur_all_nsfw: bool,
    blur_overrides: &std::collections::HashMap<i64, bool>,
) {
    let card_width = 180.0;
    let card_height = 260.0;

    // 1. Allocate proper space in the parent UI (Critical for wrapping)
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(card_width, card_height), egui::Sense::click());

    // 2. Interaction
    if response.clicked() {
        actions.push(crate::ui::browser::BrowserAction::OpenCharacter(char.id));
    }

    // 3. Hover Effects
    let is_hovered = response.hovered();
    let bg_color = if is_hovered {
        ui.visuals().widgets.hovered.bg_fill
    } else {
        ui.visuals().widgets.noninteractive.bg_fill
    };
    let stroke_color = if is_hovered {
        ui.visuals().widgets.hovered.bg_stroke
    } else {
        ui.visuals().widgets.noninteractive.bg_stroke
    };

    if is_hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    // 4. Paint Background
    ui.painter().rect_filled(rect, 8.0, bg_color);
    ui.painter().rect_stroke(rect, 8.0, stroke_color);

    // 5. Context Menu (NO BLUR OPTIONS HERE)
    response.context_menu(|ui| {
        ui.menu_button("Move to...", |ui| {
            if ui.button("Root (Uncategorized)").clicked() {
                actions.push(crate::ui::browser::BrowserAction::MoveCharacter(
                    char.id, None,
                ));
                ui.close_menu();
            }
            ui.separator();
            render_collection_move_menu(ui, &all_collections, None, char.id, actions);
        });

        if ui.button("🗑 Delete").clicked() {
            actions.push(crate::ui::browser::BrowserAction::DeleteCharacter(char.id));
            ui.close_menu();
        }
    });

    // 6. Content Rendering
    let avatar_rect = egui::Rect::from_min_size(
        rect.min + egui::vec2(8.0, 8.0),
        egui::vec2(card_width - 16.0, card_width - 16.0), // Square
    );

    if let Some(path_str) = &char.avatar_path {
        let uri = crate::ui::utils::get_image_uri(path_str);
        crate::ui::widgets::paint_avatar_crop(ui, avatar_rect, &uri, 4.0);

        // Blur Logic
        let base_blur = blur_all_images || (blur_all_nsfw && char.is_nsfw) || char.blur_avatar;
        let should_blur = if let Some(&override_val) = blur_overrides.get(&char.id) {
            override_val
        } else {
            base_blur
        };

        if should_blur {
            ui.painter().rect_filled(
                avatar_rect,
                4.0,
                egui::Color32::from_black_alpha(255), // Fully opaque black
            );
            // Optional: Icon or Text
            let text = if char.is_nsfw { "NSFW" } else { "BLURRED" };
            ui.painter().text(
                avatar_rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(24.0), // Larger text
                egui::Color32::WHITE,
            );
        }
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
            egui::FontId::proportional(32.0),
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

    // Text Content
    let text_top = avatar_rect.max.y + 8.0;
    let mut cursor_y = text_top;
    let content_left = rect.min.x + 8.0;
    let _content_width = card_width - 16.0;

    // Name
    let name_font = egui::FontId::proportional(16.0);
    let name_galley = ui.painter().layout_no_wrap(
        char.name.clone(),
        name_font.clone(),
        ui.visuals().text_color(),
    );
    ui.painter().galley(
        egui::pos2(content_left, cursor_y),
        name_galley,
        ui.visuals().text_color(),
    );
    cursor_y += 20.0;

    // Title
    if !char.char_title.is_empty() {
        let title_font = egui::FontId::proportional(12.0);
        let first_line = char.char_title.lines().next().unwrap_or("");
        let title_galley = ui.painter().layout_no_wrap(
            first_line.to_string(),
            title_font,
            ui.visuals().text_color().linear_multiply(0.7),
        );
        let mut clip_rect = rect;
        clip_rect.min.y = cursor_y;
        clip_rect.max.y = cursor_y + 14.0;
        ui.painter().with_clip_rect(clip_rect).galley(
            egui::pos2(content_left, cursor_y),
            title_galley,
            ui.visuals().text_color(),
        );
        cursor_y += 16.0;
    } else {
        cursor_y += 16.0;
    }
    cursor_y += 4.0;

    // Tags
    let mut tags_to_show: Vec<&crate::models::Tag> = char.app_tags.iter().collect();
    let mut is_external = false;
    if tags_to_show.is_empty() {
        tags_to_show = char.external_tags.iter().collect();
        is_external = true;
    }

    if !tags_to_show.is_empty() {
        let tag_font = egui::FontId::proportional(10.0);
        let mut tag_x = content_left;
        let bg_color = if is_external {
            egui::Color32::from_rgb(100, 100, 100)
        } else {
            egui::Color32::from_rgb(50, 80, 150)
        };

        for tag in tags_to_show.iter().take(2) {
            let tag_galley = ui.painter().layout_no_wrap(
                tag.name.clone(),
                tag_font.clone(),
                egui::Color32::WHITE,
            );
            let pad = 4.0;
            let chip_w = tag_galley.rect.width() + pad * 2.0;

            if tag_x + chip_w > rect.max.x - 8.0 {
                break;
            }

            let chip_rect =
                egui::Rect::from_min_size(egui::pos2(tag_x, cursor_y), egui::vec2(chip_w, 16.0));

            ui.painter().rect_filled(chip_rect, 8.0, bg_color);
            ui.painter().galley(
                egui::pos2(tag_x + pad, cursor_y + 2.0),
                tag_galley,
                egui::Color32::WHITE,
            );
            tag_x += chip_w + 4.0;
        }
    }
}

pub fn render_tag_chips(ui: &mut egui::Ui, tags: &[Tag], is_external: bool) {
    let tag_font = egui::FontId::proportional(10.0);
    let bg_color = if is_external {
        egui::Color32::from_rgb(100, 100, 100)
    } else {
        egui::Color32::from_rgb(50, 80, 150)
    };

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        ui.spacing_mut().item_spacing.y = 4.0;

        for tag in tags {
            let tag_galley = ui.painter().layout_no_wrap(
                tag.name.clone(),
                tag_font.clone(),
                egui::Color32::WHITE,
            );
            let pad = 4.0;
            let chip_w = tag_galley.rect.width() + pad * 2.0;

            let (rect, _resp) =
                ui.allocate_at_least(egui::vec2(chip_w, 16.0), egui::Sense::hover());

            ui.painter().rect_filled(rect, 8.0, bg_color);
            ui.painter().galley(
                egui::pos2(rect.min.x + pad, rect.min.y + 2.0),
                tag_galley,
                egui::Color32::WHITE,
            );
        }
    });
}
