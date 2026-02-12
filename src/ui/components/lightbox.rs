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
                    // .shrink_to_fit() // Removed to allow zooming in
                    // .max_size(max_size) // Removed to allow zooming in
                    .sense(egui::Sense::click_and_drag()); // Changed to click_and_drag for panning support

                // Handle Mouse Wheel Zoom
                let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
                if scroll_delta != 0.0 {
                    let zoom_factor = if scroll_delta > 0.0 { 1.1 } else { 0.9 };
                    app.gallery_zoom = (app.gallery_zoom * zoom_factor).clamp(0.1, 5.0);
                }

                match img.load_for_size(ui.ctx(), max_size) {
                    Ok(egui::load::TexturePoll::Ready { texture, .. }) => {
                        let img_size = texture.size;
                        // Initial scale to fit screen
                        let width_ratio = max_size.x / img_size.x;
                        let height_ratio = max_size.y / img_size.y;
                        let base_scale = width_ratio.min(height_ratio).min(1.0);

                        // Apply zoom
                        let current_scale = base_scale * app.gallery_zoom;
                        let final_size = img_size * current_scale;

                        // Apply Pan
                        let center_pos = screen_rect.center() + app.gallery_pan;
                        let img_rect = egui::Rect::from_center_size(center_pos, final_size);

                        // Render Image
                        let img_response = ui.put(img_rect, img);

                        // Handle Panning (if zoomed in or just generally)
                        if img_response.dragged() {
                            app.gallery_pan += img_response.drag_delta();
                        }
                    }
                    Ok(egui::load::TexturePoll::Pending { .. }) => {
                        ui.spinner();
                    }
                    Err(_) => {
                        ui.label("Failed to load image");
                    }
                }

                // 3. Navigation Zones (Highest Priority)
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

                // 4. Zoom Controls (Overlay)
                let control_area_size = egui::vec2(160.0, 50.0);
                let control_rect = egui::Rect::from_center_size(
                    egui::pos2(screen_rect.center().x, screen_rect.max.y - 50.0),
                    control_area_size,
                );

                // Background for controls
                ui.painter()
                    .rect_filled(control_rect, 20.0, egui::Color32::from_black_alpha(200));

                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(control_rect), |ui| {
                    ui.horizontal_centered(|ui| {
                        if ui.button("➖").clicked() {
                            app.gallery_zoom = (app.gallery_zoom - 0.1).max(0.1);
                        }
                        ui.label(format!("{:.0}%", app.gallery_zoom * 100.0));
                        if ui.button("➕").clicked() {
                            app.gallery_zoom = (app.gallery_zoom + 0.1).min(5.0);
                        }
                        if ui.button("Reset").clicked() {
                            app.gallery_zoom = 1.0;
                            app.gallery_pan = egui::vec2(0.0, 0.0);
                        }
                    });
                });
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
            // Reset zoom/pan on navigation
            app.gallery_zoom = 1.0;
            app.gallery_pan = egui::vec2(0.0, 0.0);
        }
    }
}
