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

            ui.heading("Background");
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                let mut show_bg = app.show_background;
                if ui.checkbox(&mut show_bg, "Show Background").changed() {
                    app.set_background_visibility(show_bg);
                }
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                let mut show_watermark = app.show_watermark;
                if ui.checkbox(&mut show_watermark, "Show Watermark").changed() {
                    app.set_watermark_visibility(show_watermark);
                }
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                let mut use_custom = app.use_custom_background;
                if ui
                    .checkbox(&mut use_custom, "Use Custom Background")
                    .changed()
                {
                    app.set_custom_background_mode(use_custom);
                }
            });

            ui.add_space(5.0);
            if ui.button("Select Custom Image...").clicked() {
                let ctx = ctx.clone();
                std::thread::spawn(move || {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                        .pick_file()
                    {
                        let target_dir = std::path::Path::new("data/background");
                        if !target_dir.exists() {
                            let _ = std::fs::create_dir_all(target_dir);
                        }
                        let target = target_dir.join("custom.png");

                        if std::fs::copy(path, &target).is_ok() {
                            // Invalidate cache
                            let uri = crate::ui::get_image_uri("data/background/custom.png");
                            ctx.forget_image(&uri);
                            ctx.request_repaint();
                        }
                    }
                });
            }

            ui.add_space(20.0);
            ui.separator();

            ui.heading("About");
            ui.add_space(4.0);
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("Created by JustJam").strong());
                ui.label(
                    egui::RichText::new("Special thanks to The Library of Snailexandra")
                        .size(11.0)
                        .italics(),
                );

                ui.add_space(8.0);

                // Version
                let version = env!("CARGO_PKG_VERSION");
                ui.label(egui::RichText::new(format!("Version: {}", version)).size(12.0));

                ui.add_space(4.0);

                // GitHub Link
                let repo_url = env!("CARGO_PKG_REPOSITORY");
                ui.horizontal(|ui| {
                    ui.label("GitHub:");
                    let link = ui.hyperlink(repo_url);
                    link.context_menu(|ui| {
                        if ui.button("📋 Copy URL").clicked() {
                            ui.output_mut(|o| o.copied_text = repo_url.to_string());
                            ui.close_menu();
                        }
                    });
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
