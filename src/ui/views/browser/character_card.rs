use crate::models::Tag;
use crate::ui::CrapApp;
use eframe::egui;

use super::{render_collection_move_menu, BrowserAction};

pub fn render_character_card(
    ui: &mut egui::Ui,
    _app: &mut CrapApp,
    char: &crate::models::Character,
    all_collections: &Vec<crate::models::Collection>,
    actions: &mut Vec<BrowserAction>,
) {
    let card_width = 180.0;
    let card_height = 260.0;

    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(card_width, card_height), egui::Sense::click());

    // Culling for Grid View
    if !ui.is_rect_visible(rect) {
        return;
    }

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
        actions.push(BrowserAction::OpenCharacter(char.id));
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
        let uri = crate::ui::utils::get_image_uri(path_str);
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
        // Use only the first line to prevent overlap with tags below
        let first_line = char.char_title.lines().next().unwrap_or("");
        let title_galley = ui.painter().layout_no_wrap(
            first_line.to_string(),
            title_font,
            ui.visuals().text_color().linear_multiply(0.7),
        );

        // Clip to content width to prevent horizontal overlap if it's a single very long line
        let mut title_clip = content_rect;
        title_clip.set_height(14.0);
        title_clip.min.y = cursor_y;
        title_clip.max.y = cursor_y + 14.0;

        ui.painter().with_clip_rect(title_clip).galley(
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
    let mut tags_to_show: Vec<&Tag> = char.app_tags.iter().collect();
    let mut is_external = false;
    if tags_to_show.is_empty() {
        tags_to_show = char.external_tags.iter().collect();
        is_external = true;
    }

    if !tags_to_show.is_empty() {
        let tag_font = egui::FontId::proportional(10.0);
        let mut tag_x = content_rect.min.x;
        let bg_color = if is_external {
            egui::Color32::from_rgb(100, 100, 100)
        } else {
            egui::Color32::from_rgb(50, 80, 150)
        };

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
