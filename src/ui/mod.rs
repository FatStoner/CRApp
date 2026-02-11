use eframe::egui;

use std::time::{Duration, Instant};

pub mod browser;
pub mod central_panel;
pub mod editor;
pub mod global_search;
pub mod popups;
pub mod side_panel;
pub mod text_highlight;
pub mod widgets;

pub use global_search::{CharacterSearchFieldFilters, LorebookSearchFieldFilters};
pub use popups::PopupState;

pub mod options_window;
pub mod spell_check;
pub mod spell_layout;

pub mod parsing;

// Re-export specific items if needed
pub use parsing::ParsedCharacterData;

pub mod types;
pub use types::*;
pub mod app;
pub use app::CrapApp;

pub mod events;
pub mod lightbox;
pub mod utils;

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
        side_panel::render_side_panel(self, ctx);

        // Central Panel
        central_panel::render_central_panel(self, ctx);

        // Global Popups
        popups::render_popups(ctx, self);

        // LIGHTBOX OVERLAY
        // LIGHTBOX OVERLAY
        lightbox::render_lightbox(self, ctx);

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
    }
}
