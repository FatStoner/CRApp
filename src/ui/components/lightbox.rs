use crate::ui::CrapApp;
use eframe::egui;

pub fn render_lightbox(app: &mut CrapApp, ctx: &egui::Context) {
    if let Some(uri) = &app.fullscreen_image {
        let mut close = false;
        let mut next_image = None;

        egui::Area::new("lightbox_overlay".into())
            .order(egui::Order::Foreground)
            .fixed_pos(egui::pos2(0.0, 0.0))
            .interactable(true)
            .show(ctx, |ui| {
                let screen_rect = ctx.screen_rect();
                let sw = screen_rect.width();
                let sh = screen_rect.height();

                // === PAINT ONLY (no interaction) ===

                // 1. Paint dimmed background
                ui.painter()
                    .rect_filled(screen_rect, 0.0, egui::Color32::from_black_alpha(200));

                // 1b. Background Interaction (Lowest Priority)
                let bg_response = ui.allocate_rect(screen_rect, egui::Sense::click());
                if bg_response.clicked() {
                    close = true;
                }

                // 2. Image Handling
                let max_size = screen_rect.size() * 0.9;
                let img = egui::Image::new(uri)
                    .shrink_to_fit()
                    .max_size(max_size)
                    .sense(egui::Sense::click());

                // Attempt to load image to get dimensions

                match img.load_for_size(ui.ctx(), max_size) {
                    Ok(egui::load::TexturePoll::Ready { texture, .. }) => {
                        let img_size = texture.size;
                        // Scale down if larger than max_size while maintaining aspect ratio
                        let width_ratio = max_size.x / img_size.x;
                        let height_ratio = max_size.y / img_size.y;
                        let scale = width_ratio.min(height_ratio).min(1.0);

                        let final_size = img_size * scale;
                        let img_rect =
                            egui::Rect::from_center_size(screen_rect.center(), final_size);

                        // Render Image (Middle Priority)
                        // ui.put places the widget at the exact rect, consuming clicks there
                        ui.put(img_rect, img);
                    }
                    Ok(egui::load::TexturePoll::Pending { .. }) => {
                        ui.spinner();
                    }
                    Err(_) => {
                        ui.label("Failed to load image");
                    }
                }

                // 3. Navigation Zones (Highest Priority)
                // These overlay everything.

                let nav_width = sw * 0.15;

                // Left Nav
                let left_nav_rect =
                    egui::Rect::from_min_size(screen_rect.min, egui::vec2(nav_width, sh));
                let left_resp = ui.allocate_rect(left_nav_rect, egui::Sense::click());
                if left_resp.hovered() {
                    ui.painter().rect_filled(
                        left_nav_rect,
                        0.0,
                        egui::Color32::from_black_alpha(30),
                    );
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if left_resp.clicked() {
                    if let Some(context) = &app.gallery_context {
                        if let Some(idx) = context.iter().position(|u| u == uri) {
                            let new_idx = if idx == 0 { context.len() - 1 } else { idx - 1 };
                            next_image = Some(context[new_idx].clone());
                        }
                    }
                }

                // Right Nav
                let right_nav_rect = egui::Rect::from_min_size(
                    egui::pos2(screen_rect.max.x - nav_width, screen_rect.min.y),
                    egui::vec2(nav_width, sh),
                );
                let right_resp = ui.allocate_rect(right_nav_rect, egui::Sense::click());
                if right_resp.hovered() {
                    ui.painter().rect_filled(
                        right_nav_rect,
                        0.0,
                        egui::Color32::from_black_alpha(30),
                    );
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if right_resp.clicked() {
                    if let Some(context) = &app.gallery_context {
                        if let Some(idx) = context.iter().position(|u| u == uri) {
                            let new_idx = (idx + 1) % context.len();
                            next_image = Some(context[new_idx].clone());
                        }
                    }
                }
            });

        // Handle Input
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            close = true;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
            if let Some(context) = &app.gallery_context {
                if let Some(idx) = context.iter().position(|u| u == uri) {
                    let new_idx = (idx + 1) % context.len();
                    next_image = Some(context[new_idx].clone());
                }
            }
        }

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
            if let Some(context) = &app.gallery_context {
                if let Some(idx) = context.iter().position(|u| u == uri) {
                    let new_idx = if idx == 0 { context.len() - 1 } else { idx - 1 };
                    next_image = Some(context[new_idx].clone());
                }
            }
        }

        if close {
            app.fullscreen_image = None;
            app.gallery_context = None;
        }

        if let Some(next) = next_image {
            app.fullscreen_image = Some(next);
        }
    }
}
