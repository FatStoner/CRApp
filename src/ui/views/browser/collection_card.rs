use crate::ui::CrapApp;
use eframe::egui;

use super::BrowserAction;

pub fn render_subfolder_card(
    ui: &mut egui::Ui,
    _app: &mut CrapApp,
    folder: &crate::models::Collection,
    actions: &mut Vec<BrowserAction>,
) {
    let card_width = 180.0;
    let card_height = 260.0;

    // 1. Allocate space and interact
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(card_width, card_height), egui::Sense::click());

    // 2. Visuals (Hover effect)
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

    // 3. Paint Background
    ui.painter().rect_filled(rect, 8.0, bg_color);
    ui.painter().rect_stroke(rect, 8.0, stroke_color);

    // 4. Render Content (Manual Painting)
    let center_x = rect.center().x;

    // Icon Area
    let icon_size = 140.0;
    let icon_top = rect.min.y + 24.0;
    let icon_rect = egui::Rect::from_min_size(
        egui::pos2(center_x - icon_size / 2.0, icon_top),
        egui::vec2(icon_size, icon_size),
    );

    if let Some(path) = &folder.image_path {
        let uri = if path.contains("://") {
            path.clone()
        } else {
            if let Ok(abs) = std::fs::canonicalize(path) {
                format!("file://{}", abs.to_string_lossy())
            } else {
                path.clone()
            }
        };
        crate::ui::widgets::paint_avatar_crop(ui, icon_rect, &uri, 8.0);
    } else {
        let icon_bg = ui
            .visuals()
            .widgets
            .noninteractive
            .bg_stroke
            .color
            .linear_multiply(0.5);
        ui.painter().rect_filled(icon_rect, 8.0, icon_bg);
        ui.painter().text(
            icon_rect.center(),
            egui::Align2::CENTER_CENTER,
            "📁",
            egui::FontId::proportional(64.0),
            ui.visuals().text_color(),
        );
    }

    // Text Area
    let text_top = icon_rect.max.y + 16.0;
    let text_pos = egui::pos2(center_x, text_top);

    // Manual layout for centered text
    ui.painter().text(
        text_pos,
        egui::Align2::CENTER_TOP,
        &folder.name,
        egui::FontId::proportional(16.0),
        ui.visuals().text_color(),
    );

    // 5. Interaction Logic
    if response.clicked() {
        actions.push(BrowserAction::OpenCollection(folder.id));
    }

    response.context_menu(|ui| {
        if ui.button("Rename Folder").clicked() {
            actions.push(BrowserAction::RenameCollection(
                folder.id,
                folder.name.clone(),
            ));
            ui.close_menu();
        }
        if ui.button("Change Icon").clicked() {
            actions.push(BrowserAction::UpdateCollectionIcon(folder.id));
            ui.close_menu();
        }
        if ui.button("Delete Folder").clicked() {
            actions.push(BrowserAction::DeleteCollection(folder.id));
            ui.close_menu();
        }
        if ui.button("📤 Export Folder").clicked() {
            actions.push(BrowserAction::ExportCollection(folder.id));
            ui.close_menu();
        }
    });
}

pub fn render_subfolder_list_item(
    ui: &mut egui::Ui,
    _app: &mut CrapApp,
    folder: &crate::models::Collection,
    count: usize,
    actions: &mut Vec<BrowserAction>,
) {
    ui.add_space(8.0);

    // List Item Hover Interaction
    let id = ui.make_persistent_id(format!("folder_list_{}", folder.id));
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
        actions.push(BrowserAction::OpenCollection(folder.id));
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

    let frame_response = egui::Frame::group(ui.style())
        .fill(bg_color)
        .stroke(stroke)
        .rounding(4.0)
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                // Folder Icon
                let size = 80.0;
                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());

                if response.clicked() {
                    actions.push(BrowserAction::OpenCollection(folder.id));
                }

                response.context_menu(|ui| {
                    if ui.button("Rename Folder").clicked() {
                        actions.push(BrowserAction::RenameCollection(
                            folder.id,
                            folder.name.clone(),
                        ));
                        ui.close_menu();
                    }
                    if ui.button("Change Icon").clicked() {
                        actions.push(BrowserAction::UpdateCollectionIcon(folder.id));
                        ui.close_menu();
                    }
                    if ui.button("Delete Folder").clicked() {
                        actions.push(BrowserAction::DeleteCollection(folder.id));
                        ui.close_menu();
                    }
                    if ui.button("📤 Export Folder").clicked() {
                        actions.push(BrowserAction::ExportCollection(folder.id));
                        ui.close_menu();
                    }
                });

                if let Some(path) = &folder.image_path {
                    let uri = if path.contains("://") {
                        path.clone()
                    } else {
                        if let Ok(abs) = std::fs::canonicalize(path) {
                            format!("file://{}", abs.to_string_lossy())
                        } else {
                            path.clone()
                        }
                    };
                    crate::ui::widgets::paint_avatar_crop(ui, rect, &uri, 4.0);
                } else {
                    let icon_bg = ui
                        .visuals()
                        .widgets
                        .noninteractive
                        .bg_stroke
                        .color
                        .linear_multiply(0.5);
                    ui.painter().rect_filled(rect, 4.0, icon_bg);
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "📁",
                        egui::FontId::proportional(40.0),
                        ui.visuals().text_color(),
                    );
                }

                ui.add_space(10.0);

                // Info
                ui.vertical(|ui| {
                    ui.heading(&folder.name);
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(format!("Contains {} characters", count))
                            .color(egui::Color32::GRAY),
                    );
                });
            });
        })
        .response;

    ui.data_mut(|d| d.insert_temp(id, frame_response.rect));
}
