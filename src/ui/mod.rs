use eframe::egui;
use std::time::{Duration, Instant};

// Sub-modules (New Structure)
pub mod components;
pub mod controllers;
pub mod events;
pub mod panels;
pub mod parsing;
pub mod types;
pub mod utils;
pub mod views;

// Re-exports for backward compatibility and convenience
pub use components::popups::PopupState;
pub use components::spell_check;
pub use components::spell_layout;
pub use components::widgets;
pub use controllers::CrapApp;

pub use views::browser;

pub use parsing::ParsedCharacterData;
pub use types::*;

impl eframe::App for CrapApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.pointer.button_pressed(egui::PointerButton::Extra1)) {
            self.request_back();
        }

        // Smoother UI Scaling with debouncing
        let (scroll_y, ctrl) = ctx.input(|i| (i.raw_scroll_delta.y, i.modifiers.ctrl));
        if ctrl
            && scroll_y.abs() > 0.01
            && self.last_scroll_time.elapsed() > Duration::from_millis(50)
        {
            self.last_scroll_time = Instant::now();
            let step = 0.05;
            let mut new_scale = self.ui_scale;
            if scroll_y > 0.0 {
                new_scale += step;
            } else {
                new_scale -= step;
            }

            new_scale = (new_scale * 20.0).round() / 20.0; // Snap to 5%
            new_scale = new_scale.clamp(0.5, 2.0);

            if (new_scale - self.ui_scale).abs() > 0.001 {
                self.ui_scale = new_scale;
                self.ctx.set_pixels_per_point(new_scale);
                self.scale_last_updated = Some(Instant::now());
                self.ctx.request_repaint(); // Snappy refresh

                let pct = (new_scale * 100.0).round() as i32;
                self.set_status(
                    format!("UI Scale: {}%", pct),
                    egui::Color32::from_rgb(100, 200, 255),
                );
            }
        }

        // Search Focus Shortcut
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::F)) {
            self.focus_search_field = true;
        }

        // Debounced Save
        if let Some(last_time) = self.scale_last_updated {
            if last_time.elapsed() > Duration::from_millis(1000) {
                self.scale_last_updated = None;
                self.set_scale(self.ui_scale); // Triggers DB save
            }
        }

        // Event Loop
        events::handle_ui_events(self, ctx);

        // Timer
        if let Some(deadline) = self.status_clear_time {
            if Instant::now() > deadline {
                self.status_message = None;
                self.status_clear_time = None;
            } else {
                ctx.request_repaint();
            }
        }

        // Handle Close Request
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.has_unsaved_changes() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.popup_state = PopupState::UnsavedChanges {
                    target: AppAction::Exit,
                };
            }
        }

        // Side Panel
        panels::side::render_side_panel(self, ctx);

        // Central Panel
        panels::central::render_central_panel(self, ctx);

        // Global Popups
        components::popups::render_popups(ctx, self);
        views::statistics::render_statistics_window(self, ctx);

        // LIGHTBOX OVERLAY
        components::lightbox::render_lightbox(self, ctx);

        // Watermark: The Library of Snailexandria
        if self.show_watermark && ctx.screen_rect().width() > 300.0 {
            egui::Area::new("watermark_area".into())
                .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-10.0, -5.0))
                .order(egui::Order::Foreground)
                .interactable(false)
                .show(ctx, |ui| {
                    ui.style_mut().interaction.selectable_labels = false;
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("The Library of Snailexandria")
                                .size(11.0)
                                .color(egui::Color32::from_white_alpha(100))
                                .italics(),
                        );
                    });
                });
        }

        self.cosmic_atlas.trim();
    }
}
