use crate::models::ThemeMode;
use crate::ui::types::{EditorFontFamily, SettingsTab};
use crate::ui::CrapApp;
use eframe::egui;

pub fn render_options_window(app: &mut CrapApp, ctx: &egui::Context) {
    let mut close = false;
    egui::Window::new("Options")
        .collapsible(false)
        .resizable(false)
        .fixed_size([400.0, 500.0])
        .show(ctx, |ui| {
            // Tab Bar
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut app.active_settings_tab,
                    SettingsTab::General,
                    "General",
                );
                ui.selectable_value(&mut app.active_settings_tab, SettingsTab::Tokens, "Tokens");
                ui.selectable_value(
                    &mut app.active_settings_tab,
                    SettingsTab::Updates,
                    "Updates",
                );
                ui.selectable_value(
                    &mut app.active_settings_tab,
                    SettingsTab::Dictionary,
                    "Dictionary",
                );
                ui.selectable_value(&mut app.active_settings_tab, SettingsTab::About, "About");
            });
            ui.separator();
            ui.add_space(10.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                match app.active_settings_tab {
                    SettingsTab::General => {
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

                        ui.horizontal(|ui| {
                            let mut enabled = app.enable_spell_check;
                            if ui.checkbox(&mut enabled, "Enable Spell Check").changed() {
                                app.set_spell_check(enabled);
                            }
                        });

                        ui.add_space(8.0);

                        // Font Selector
                        ui.horizontal(|ui| {
                            ui.label("Field Font:");
                            egui::ComboBox::from_id_source("font_combo_options")
                                .selected_text(app.editor_font.to_string())
                                .show_ui(ui, |ui| {
                                    if ui
                                        .selectable_value(
                                            &mut app.editor_font,
                                            EditorFontFamily::SansSerif,
                                            "SansSerif",
                                        )
                                        .clicked()
                                    {
                                        app.set_editor_font(EditorFontFamily::SansSerif);
                                    }
                                    if ui
                                        .selectable_value(
                                            &mut app.editor_font,
                                            EditorFontFamily::Serif,
                                            "Serif",
                                        )
                                        .clicked()
                                    {
                                        app.set_editor_font(EditorFontFamily::Serif);
                                    }
                                    if ui
                                        .selectable_value(
                                            &mut app.editor_font,
                                            EditorFontFamily::Monospace,
                                            "Monospace",
                                        )
                                        .clicked()
                                    {
                                        app.set_editor_font(EditorFontFamily::Monospace);
                                    }
                                });
                        });

                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            let mut large_font = app.editor_large_font;
                            if ui.checkbox(&mut large_font, "Large Font").changed() {
                                app.set_editor_large_font(large_font);
                            }
                        });

                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            let mut bright = app.editor_bright_mode;
                            if ui.checkbox(&mut bright, "Bright Text").changed() {
                                app.set_editor_bright_mode(bright);
                            }
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);
                        ui.heading("Content Blur");

                        ui.horizontal(|ui| {
                            let mut blur_all = app.blur_all_images;
                            if ui.checkbox(&mut blur_all, "Blur All Images").changed() {
                                app.set_blur_all_images(blur_all);
                            }
                        });

                        ui.horizontal(|ui| {
                            let mut blur_nsfw = app.blur_all_nsfw;
                            if ui
                                .checkbox(&mut blur_nsfw, "Blur All NSFW Characters")
                                .changed()
                            {
                                app.set_blur_all_nsfw(blur_nsfw);
                            }
                        });

                        ui.add_space(8.0);

                        // Scale
                        ui.horizontal(|ui| {
                            ui.label("Scale:");
                            let current_scale = (app.ui_scale * 100.0).round() as i32;
                            let mut selected = current_scale;

                            egui::ComboBox::from_id_source("scale_combo_options")
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

                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            ui.label("Scale:");
                            let mut scale = app.background_scale;
                            if ui
                                .add(egui::Slider::new(&mut scale, 0.1..=1.5).text("Ratio"))
                                .changed()
                            {
                                app.set_background_scale(scale);
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
                                        let uri = crate::ui::utils::get_image_uri(
                                            "data/background/custom.png",
                                        );
                                        ctx.forget_image(&uri);
                                        ctx.request_repaint();
                                    }
                                }
                            });
                        }
                    }
                    SettingsTab::Tokens => {
                        ui.heading("Token Counting");
                        ui.add_space(8.0);
                        ui.label("Select sections to include in the total token count:");

                        ui.add_space(8.0);
                        ui.horizontal_wrapped(|ui| {
                            let mut changed = false;
                            if ui.checkbox(&mut app.count_name_in_total, "Name").changed() {
                                changed = true;
                            }
                            if ui
                                .checkbox(&mut app.count_title_in_total, "Title/Description")
                                .changed()
                            {
                                changed = true;
                            }
                            if ui
                                .checkbox(&mut app.count_first_message_in_total, "First Message")
                                .changed()
                            {
                                changed = true;
                            }
                            if ui
                                .checkbox(&mut app.count_personality_in_total, "Personality")
                                .changed()
                            {
                                changed = true;
                            }
                            if ui
                                .checkbox(&mut app.count_scenario_in_total, "Scenario")
                                .changed()
                            {
                                changed = true;
                            }
                            if ui
                                .checkbox(&mut app.count_example_in_total, "Example Dialogue")
                                .changed()
                            {
                                changed = true;
                            }

                            if changed {
                                app.token_cache.clear();
                                if let Some(c) = app.selected_character.clone() {
                                    app.ensure_token_count(&c);
                                }
                            }
                        });
                    }
                    SettingsTab::Updates => {
                        ui.heading("Updates");
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            let mut check = app.check_updates_at_start;
                            if ui
                                .checkbox(&mut check, "Check for updates at the start")
                                .changed()
                            {
                                app.set_check_updates_at_start(check);
                            }
                        });

                        ui.horizontal(|ui| {
                            if app.is_checking_for_updates {
                                ui.spinner();
                                ui.label("Checking for updates...");
                            } else {
                                if ui.button("Check for updates now").clicked() {
                                    // Manual check
                                    app.is_checking_for_updates = true;
                                    let tx = app.tx.clone();
                                    let ctx = app.ctx.clone();
                                    std::thread::spawn(move || {
                                        match crate::updater::check_for_updates() {
                                            Ok(res) => {
                                                let _ = tx.blocking_send(
                                                    crate::ui::UiEvent::UpdateCheckFinished(
                                                        Ok(res),
                                                        true,
                                                    ),
                                                );
                                                ctx.request_repaint();
                                            }
                                            Err(e) => {
                                                let _ = tx.blocking_send(
                                                    crate::ui::UiEvent::UpdateCheckFinished(
                                                        Err(e.to_string()),
                                                        true,
                                                    ),
                                                );
                                                ctx.request_repaint();
                                            }
                                        }
                                    });
                                }
                            }
                        });
                    }
                    SettingsTab::Dictionary => {
                        ui.heading("Dictionary");
                        ui.add_space(8.0);
                        ui.label("Manage words ignored by the spell checker.");

                        ui.add_space(8.0);

                        if let Some(checker) = &app.spell_checker {
                            let words = checker.get_ignored_words();

                            egui::ScrollArea::vertical()
                                .max_height(300.0)
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    if words.is_empty() {
                                        ui.label(
                                            egui::RichText::new("No custom words added.")
                                                .italics()
                                                .color(egui::Color32::GRAY),
                                        );
                                    } else {
                                        for word in &words {
                                            ui.label(word);
                                        }
                                    }
                                });

                            ui.add_space(10.0);

                            if ui.button("Edit Dictionary").clicked() {
                                app.popup_state = crate::ui::PopupState::DictionaryEdit {
                                    new_word_input: String::new(),
                                };
                            }
                        } else {
                            ui.label(
                                egui::RichText::new("Spell checker is not initialized.")
                                    .color(egui::Color32::RED),
                            );
                        }
                    }
                    SettingsTab::About => {
                        ui.heading("About");
                        ui.add_space(4.0);
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new("Created by JustJam").strong());
                            ui.label(
                                egui::RichText::new(
                                    "Special thanks to The Library of Snailexandra",
                                )
                                .size(11.0)
                                .italics(),
                            );

                            ui.add_space(8.0);

                            // Version
                            let version = env!("CARGO_PKG_VERSION");
                            ui.label(
                                egui::RichText::new(format!("Version: {}", version)).size(12.0),
                            );

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
                    }
                }
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
