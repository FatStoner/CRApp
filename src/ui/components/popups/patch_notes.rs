use crate::ui::{CrapApp, PopupState};
use eframe::egui;

pub fn render_patch_notes_popup(ctx: &egui::Context, app: &mut CrapApp, state: &PopupState) {
    if let PopupState::PatchNotes { content } = state {
        let mut open = true;

        let screen_height = ctx.screen_rect().height();
        let max_window_height = screen_height * 0.9;

        egui::Window::new("Patch Notes")
            .collapsible(false)
            .resizable(false)
            .min_width(500.0)
            .max_width(500.0)
            .max_height(max_window_height)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .open(&mut open)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Content area with scroll
                    egui::ScrollArea::vertical()
                        .max_height(max_window_height - 80.0)
                        .auto_shrink([false, true]) // Shrink height to fit content if possible
                        .show(ui, |ui| {
                            render_markdown(ui, content);
                        });

                    ui.add_space(5.0);
                    ui.separator();
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Close").clicked() {
                                app.popup_state = PopupState::None;
                            }
                        });
                    });
                });
            });

        if !open {
            app.popup_state = PopupState::None;
        }
    }
}

fn render_markdown(ui: &mut egui::Ui, content: &str) {
    for line in content.lines() {
        if line.starts_with("# ") {
            ui.add_space(10.0);
            ui.heading(line.trim_start_matches("# "));
            ui.separator();
            ui.add_space(5.0);
        } else if line.starts_with("## ") {
            ui.add_space(8.0);
            ui.add(egui::Label::new(
                egui::RichText::new(line.trim_start_matches("## "))
                    .strong()
                    .size(18.0),
            ));
            ui.add_space(2.0);
        } else if line.starts_with("### ") {
            ui.add_space(5.0);
            ui.add(egui::Label::new(
                egui::RichText::new(line.trim_start_matches("### "))
                    .strong()
                    .size(16.0),
            ));
        } else if line.starts_with("- ") {
            ui.horizontal(|ui| {
                ui.label(" • ");
                render_formatted_text(ui, line.trim_start_matches("- "));
            });
        } else if line.is_empty() {
            ui.add_space(5.0);
        } else {
            render_formatted_text(ui, line);
        }
    }
}

fn render_formatted_text(ui: &mut egui::Ui, text: &str) {
    // Very simple bold parser for **text**
    let parts: Vec<&str> = text.split("**").collect();
    if parts.len() > 1 {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            for (i, part) in parts.iter().enumerate() {
                if i % 2 == 1 {
                    ui.label(egui::RichText::new(*part).strong());
                } else {
                    ui.label(*part);
                }
            }
        });
    } else {
        ui.label(text);
    }
}
