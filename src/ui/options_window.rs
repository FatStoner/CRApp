use crate::models::ThemeMode;
use crate::ui::CrapApp;
use eframe::egui;

pub fn render_options_window(app: &mut CrapApp, ctx: &egui::Context) {
    let mut close = false;
    egui::Window::new("Options")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.heading("Settings");
            ui.separator();
            ui.add_space(10.0);

            // Theme Toggle
            ui.horizontal(|ui| {
                ui.label("Theme:");
                let theme_txt = match app.theme {
                    ThemeMode::System => "🌗 Auto",
                    ThemeMode::Light => "☀️ Light",
                    ThemeMode::Dark => "🌙 Dark",
                };
                if ui.button(theme_txt).clicked() {
                    let new_theme = match app.theme {
                        ThemeMode::System => ThemeMode::Light,
                        ThemeMode::Light => ThemeMode::Dark,
                        ThemeMode::Dark => ThemeMode::System,
                    };
                    app.set_theme(new_theme);
                }
            });

            ui.add_space(8.0);

            // Scale
            ui.horizontal(|ui| {
                ui.label("Scale:");
                let current_scale = (app.ui_scale * 100.0).round() as i32;
                let mut selected = current_scale;

                egui::ComboBox::from_id_salt("scale_combo_options")
                    .selected_text(format!("{}%", current_scale))
                    .show_ui(ui, |ui| {
                        for p in (50..=200).step_by(10) {
                            if ui
                                .selectable_value(&mut selected, p, format!("{}%", p))
                                .clicked()
                            {
                                app.set_scale(selected as f32 / 100.0);
                            }
                        }
                    });
            });

            ui.add_space(20.0);
            ui.separator();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                if ui.button("Close").clicked() {
                    close = true;
                }
            });
        });

    if close {
        app.show_options_window = false;
    }
}
