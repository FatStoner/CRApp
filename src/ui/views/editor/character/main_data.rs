use crate::models::{count_tokens, Character};
use crate::ui::CrapApp;
use eframe::egui;

pub fn render_main_data_tab(
    app: &mut CrapApp,
    ui: &mut egui::Ui,
    character: &mut Character,
    status_update: &mut Option<(String, egui::Color32)>,
    tag_add_request: &mut Option<(i64, String, bool)>,
    tag_remove_request: &mut Option<(i64, i64, bool)>,
) {
    ui.horizontal(|ui| {
        let available_width = ui.available_width();
        let left_width = available_width * 0.66;
        // Right width is remaining

        ui.allocate_ui_with_layout(
            egui::vec2(left_width, ui.available_height()),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.label("Name (File Name)");
                // File Name (character.name) with search highlight
                // File Name (character.name)
                crate::ui::components::CodeEditor::new(
                    &mut character.name,
                    "character_file_name_editor",
                )
                .single_line()
                .highlight(app.editor_search_query.clone())
                .show(
                    ui,
                    &mut app.cosmic_font_system,
                    &mut app.cosmic_swash_cache,
                    &mut app.cosmic_atlas,
                    &mut app.cosmic_editors,
                    &mut app.cosmic_clipboard,
                );

                ui.label("Character Name");
                // Character Name (character.char_name)
                crate::ui::components::CodeEditor::new(
                    &mut character.char_name,
                    "character_real_name_editor",
                )
                .single_line()
                .highlight(app.editor_search_query.clone())
                .show(
                    ui,
                    &mut app.cosmic_font_system,
                    &mut app.cosmic_swash_cache,
                    &mut app.cosmic_atlas,
                    &mut app.cosmic_editors,
                    &mut app.cosmic_clipboard,
                );

                let id = ui.make_persistent_id("title_header");
                egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    id,
                    true,
                )
                .show_header(ui, |ui| {
                    ui.label("Title / Description");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut ignore = character.spell_check_overrides.contains("title");
                        if ui.checkbox(&mut ignore, "Ignore Spell Check").changed() {
                            if ignore {
                                character.spell_check_overrides.insert("title".to_string());
                            } else {
                                character.spell_check_overrides.remove("title");
                            }
                        }
                        if ui.small_button("Copy").clicked() {
                            ui.output_mut(|o| o.copied_text = character.char_title.clone());
                            *status_update = Some((
                                "Copied Title to clipboard".to_string(),
                                egui::Color32::GREEN,
                            ));
                        }
                        ui.label(
                            egui::RichText::new(format!(
                                "Tokens: {} | Chars: {}",
                                count_tokens(&character.char_title),
                                character.char_title.chars().count()
                            ))
                            .size(12.0)
                            .color(egui::Color32::GRAY),
                        );
                    });
                })
                .body(|ui| {
                    crate::ui::components::CodeEditor::new(
                        &mut character.char_title,
                        "char_title_editor",
                    )
                    .desired_lines(1)
                    .highlight(app.editor_search_query.clone())
                    .show(
                        ui,
                        &mut app.cosmic_font_system,
                        &mut app.cosmic_swash_cache,
                        &mut app.cosmic_atlas,
                        &mut app.cosmic_editors,
                        &mut app.cosmic_clipboard,
                    );
                });

                ui.add_space(8.0);

                ui.add_space(8.0);
                let id = ui.make_persistent_id("first_message_header");
                egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    id,
                    true,
                )
                .show_header(ui, |ui| {
                    ui.label("First Message");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut ignore = character.spell_check_overrides.contains("first_message");
                        if ui.checkbox(&mut ignore, "Ignore Spell Check").changed() {
                            if ignore {
                                character
                                    .spell_check_overrides
                                    .insert("first_message".to_string());
                            } else {
                                character.spell_check_overrides.remove("first_message");
                            }
                        }
                        if ui.small_button("Copy").clicked() {
                            ui.output_mut(|o| o.copied_text = character.first_message.clone());
                            *status_update = Some((
                                "Copied First Message to clipboard".to_string(),
                                egui::Color32::GREEN,
                            ));
                        }
                        ui.label(
                            egui::RichText::new(format!(
                                "Tokens: {} | Chars: {}",
                                count_tokens(&character.first_message),
                                character.first_message.chars().count()
                            ))
                            .size(12.0)
                            .color(egui::Color32::GRAY),
                        );
                    });
                })
                .body(|ui| {
                    crate::ui::components::CodeEditor::new(
                        &mut character.first_message,
                        "first_message_editor",
                    )
                    .desired_lines(10)
                    .highlight(app.editor_search_query.clone())
                    .show(
                        ui,
                        &mut app.cosmic_font_system,
                        &mut app.cosmic_swash_cache,
                        &mut app.cosmic_atlas,
                        &mut app.cosmic_editors,
                        &mut app.cosmic_clipboard,
                    );
                });

                ui.add_space(8.0);
                ui.add_space(8.0);
                let id = ui.make_persistent_id("personality_header");
                egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    id,
                    true,
                )
                .show_header(ui, |ui| {
                    ui.label("Personality");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut ignore = character.spell_check_overrides.contains("personality");
                        if ui.checkbox(&mut ignore, "Ignore Spell Check").changed() {
                            if ignore {
                                character
                                    .spell_check_overrides
                                    .insert("personality".to_string());
                            } else {
                                character.spell_check_overrides.remove("personality");
                            }
                        }
                        if ui.small_button("Copy").clicked() {
                            ui.output_mut(|o| o.copied_text = character.personality.clone());
                            *status_update = Some((
                                "Copied Personality to clipboard".to_string(),
                                egui::Color32::GREEN,
                            ));
                        }
                        ui.label(
                            egui::RichText::new(format!(
                                "Tokens: {} | Chars: {}",
                                count_tokens(&character.personality),
                                character.personality.chars().count()
                            ))
                            .size(12.0)
                            .color(egui::Color32::GRAY),
                        );
                    });
                })
                .body(|ui| {
                    crate::ui::components::CodeEditor::new(
                        &mut character.personality,
                        "personality_editor",
                    )
                    .desired_lines(10)
                    .highlight(app.editor_search_query.clone())
                    .show(
                        ui,
                        &mut app.cosmic_font_system,
                        &mut app.cosmic_swash_cache,
                        &mut app.cosmic_atlas,
                        &mut app.cosmic_editors,
                        &mut app.cosmic_clipboard,
                    );
                });

                ui.add_space(8.0);
                let id = ui.make_persistent_id("scenario_header");
                egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    id,
                    true,
                )
                .show_header(ui, |ui| {
                    ui.label("Scenario");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut ignore = character.spell_check_overrides.contains("scenario");
                        if ui.checkbox(&mut ignore, "Ignore Spell Check").changed() {
                            if ignore {
                                character
                                    .spell_check_overrides
                                    .insert("scenario".to_string());
                            } else {
                                character.spell_check_overrides.remove("scenario");
                            }
                        }
                        if ui.small_button("Copy").clicked() {
                            ui.output_mut(|o| o.copied_text = character.scenario.clone());
                            *status_update = Some((
                                "Copied Scenario to clipboard".to_string(),
                                egui::Color32::GREEN,
                            ));
                        }
                        ui.label(
                            egui::RichText::new(format!(
                                "Tokens: {} | Chars: {}",
                                count_tokens(&character.scenario),
                                character.scenario.chars().count()
                            ))
                            .size(12.0)
                            .color(egui::Color32::GRAY),
                        );
                    });
                })
                .body(|ui| {
                    crate::ui::components::CodeEditor::new(
                        &mut character.scenario,
                        "scenario_editor",
                    )
                    .desired_lines(8)
                    .highlight(app.editor_search_query.clone())
                    .show(
                        ui,
                        &mut app.cosmic_font_system,
                        &mut app.cosmic_swash_cache,
                        &mut app.cosmic_atlas,
                        &mut app.cosmic_editors,
                        &mut app.cosmic_clipboard,
                    );
                });

                ui.add_space(8.0);
                let id = ui.make_persistent_id("example_dialogue_header");
                egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    id,
                    true,
                )
                .show_header(ui, |ui| {
                    ui.label("Example Dialogue");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut ignore = character.spell_check_overrides.contains("example");
                        if ui.checkbox(&mut ignore, "Ignore Spell Check").changed() {
                            if ignore {
                                character
                                    .spell_check_overrides
                                    .insert("example".to_string());
                            } else {
                                character.spell_check_overrides.remove("example");
                            }
                        }
                        if ui.small_button("Copy").clicked() {
                            ui.output_mut(|o| o.copied_text = character.example_dialogue.clone());
                            *status_update = Some((
                                "Copied Example Dialogue to clipboard".to_string(),
                                egui::Color32::GREEN,
                            ));
                        }
                        ui.label(
                            egui::RichText::new(format!(
                                "Tokens: {} | Chars: {}",
                                count_tokens(&character.example_dialogue),
                                character.example_dialogue.chars().count()
                            ))
                            .size(12.0)
                            .color(egui::Color32::GRAY),
                        );
                    });
                })
                .body(|ui| {
                    crate::ui::components::CodeEditor::new(
                        &mut character.example_dialogue,
                        "example_dialogue_editor",
                    )
                    .desired_lines(8)
                    .highlight(app.editor_search_query.clone())
                    .show(
                        ui,
                        &mut app.cosmic_font_system,
                        &mut app.cosmic_swash_cache,
                        &mut app.cosmic_atlas,
                        &mut app.cosmic_editors,
                        &mut app.cosmic_clipboard,
                    );
                });

                egui::CollapsingHeader::new("Tags & Metadata")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            // App Tags
                            ui.label(
                                egui::RichText::new("CRApp Tags")
                                    .strong()
                                    .color(egui::Color32::from_rgb(100, 150, 255)),
                            );
                            ui.horizontal(|ui| {
                                let mut app_tags_sorted: Vec<_> =
                                    character.app_tags.iter().collect();
                                app_tags_sorted.sort_by(|a, b| {
                                    a.name.to_lowercase().cmp(&b.name.to_lowercase())
                                });
                                for tag in app_tags_sorted {
                                    egui::Frame::none()
                                        .fill(egui::Color32::from_rgb(50, 80, 150))
                                        .rounding(12.0)
                                        .inner_margin(4.0)
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    egui::RichText::new(&tag.name)
                                                        .color(egui::Color32::WHITE)
                                                        .size(12.0),
                                                );
                                                if ui.small_button("x").clicked() {
                                                    *tag_remove_request =
                                                        Some((character.id, tag.id, false));
                                                }
                                            });
                                        });
                                }
                            });
                            ui.horizontal(|ui| {
                                let response = ui.text_edit_singleline(&mut app.app_tag_input);
                                if (ui.button("Add").clicked()
                                    || (response.lost_focus()
                                        && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                                    && !app.app_tag_input.is_empty()
                                {
                                    *tag_add_request =
                                        Some((character.id, app.app_tag_input.clone(), false));
                                    app.app_tag_input.clear();
                                    response.request_focus();
                                }
                            });

                            ui.add_space(8.0);

                            // External Tags
                            ui.label(
                                egui::RichText::new("External Tags")
                                    .strong()
                                    .color(egui::Color32::GRAY),
                            );
                            ui.horizontal(|ui| {
                                let mut ext_tags_sorted: Vec<_> =
                                    character.external_tags.iter().collect();
                                ext_tags_sorted.sort_by(|a, b| {
                                    a.name.to_lowercase().cmp(&b.name.to_lowercase())
                                });
                                for tag in ext_tags_sorted {
                                    egui::Frame::none()
                                        .fill(egui::Color32::from_gray(80))
                                        .rounding(12.0)
                                        .inner_margin(4.0)
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    egui::RichText::new(&tag.name)
                                                        .color(egui::Color32::WHITE)
                                                        .size(12.0),
                                                );
                                                if ui.small_button("x").clicked() {
                                                    *tag_remove_request =
                                                        Some((character.id, tag.id, true));
                                                }
                                            });
                                        });
                                }
                            });
                            ui.horizontal(|ui| {
                                let response = ui.text_edit_singleline(&mut app.ext_tag_input);
                                if (ui.button("Add").clicked()
                                    || (response.lost_focus()
                                        && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                                    && !app.ext_tag_input.is_empty()
                                {
                                    *tag_add_request =
                                        Some((character.id, app.ext_tag_input.clone(), true));
                                    app.ext_tag_input.clear();
                                    response.request_focus();
                                }
                            });
                        });
                    });
            },
        );

        ui.add_space(8.0);

        ui.vertical(|ui| {
            ui.label("Avatar");

            // Show image preview if available
            if let Some(path_str) = character.avatar_path.clone() {
                let uri = crate::ui::utils::get_image_uri(&path_str);

                // Calculate preview size based on available width in this column
                let preview_width = ui.available_width() - 8.0;
                let response = ui.add(
                    egui::Image::new(uri)
                        .rounding(egui::Rounding::same(4.0))
                        .fit_to_original_size(0.5) // Adjust scaling logic if needed or use max_width
                        .max_width(preview_width)
                        .sense(egui::Sense::click()),
                );

                response.context_menu(|ui| {
                    if ui.button("Copy to Clipboard").clicked() {
                        match std::fs::read(&path_str) {
                            Ok(bytes) => match image::load_from_memory(&bytes) {
                                Ok(dynamic_img) => {
                                    let rgba = dynamic_img.to_rgba8();
                                    let img_data = arboard::ImageData {
                                        width: rgba.width() as usize,
                                        height: rgba.height() as usize,
                                        bytes: std::borrow::Cow::from(rgba.into_raw()),
                                    };

                                    match arboard::Clipboard::new() {
                                        Ok(mut clipboard) => {
                                            if let Err(e) = clipboard.set_image(img_data) {
                                                *status_update = Some((
                                                    format!("Failed to copy to clipboard: {}", e),
                                                    egui::Color32::RED,
                                                ));
                                            } else {
                                                *status_update = Some((
                                                    "Avatar copied to clipboard!".to_string(),
                                                    egui::Color32::GREEN,
                                                ));
                                            }
                                        }
                                        Err(e) => {
                                            *status_update = Some((
                                                format!("Clipboard access failed: {}", e),
                                                egui::Color32::RED,
                                            ));
                                        }
                                    }
                                }
                                Err(e) => {
                                    *status_update = Some((
                                        format!("Failed to load image: {}", e),
                                        egui::Color32::RED,
                                    ));
                                }
                            },
                            Err(e) => {
                                *status_update = Some((
                                    format!("Failed to read avatar file: {}", e),
                                    egui::Color32::RED,
                                ));
                            }
                        }
                        ui.close_menu();
                    }

                    if ui.button("Open Folder").clicked() {
                        #[cfg(target_os = "windows")]
                        {
                            let _ = std::process::Command::new("explorer")
                                .arg("/select,")
                                .arg(path_str.replace("/", "\\"))
                                .spawn();
                        }

                        #[cfg(target_os = "linux")]
                        {
                            if let Ok(abs_path) = std::fs::canonicalize(&path_str) {
                                let file_uri = format!("file://{}", abs_path.to_string_lossy());
                                // Try D-Bus for selection first (standard modern Linux)
                                let status = std::process::Command::new("dbus-send")
                                    .args(&[
                                        "--session",
                                        "--dest=org.freedesktop.FileManager1",
                                        "--type=method_call",
                                        "/org/freedesktop/FileManager1",
                                        "org.freedesktop.FileManager1.ShowItems",
                                        &format!("array:string:{}", file_uri),
                                        "string:\"\"",
                                    ])
                                    .status();

                                if status.is_err() || !status.unwrap().success() {
                                    // Fallback to just opening the parent directory
                                    if let Some(parent) = abs_path.parent() {
                                        let _ = std::process::Command::new("xdg-open")
                                            .arg(parent)
                                            .spawn();
                                    }
                                }
                            }
                        }

                        #[cfg(target_os = "macos")]
                        {
                            let _ = std::process::Command::new("open")
                                .arg("-R")
                                .arg(&path_str)
                                .spawn();
                        }
                        ui.close_menu();
                    }

                    if ui.button("Change Avatar").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("image", &["png", "jpg", "jpeg", "webp"])
                            .pick_file()
                        {
                            if let Some(avatar_path) =
                                app.update_avatar_from_file(path, character.id)
                            {
                                character.avatar_path = Some(avatar_path);
                            }
                        }
                        ui.close_menu();
                    }
                });

                ui.label(&path_str);

                ui.horizontal(|ui| {
                    if ui.button("Copy to Clipboard").clicked() {
                        match std::fs::read(&path_str) {
                            Ok(bytes) => match image::load_from_memory(&bytes) {
                                Ok(dynamic_img) => {
                                    let rgba = dynamic_img.to_rgba8();
                                    let img_data = arboard::ImageData {
                                        width: rgba.width() as usize,
                                        height: rgba.height() as usize,
                                        bytes: std::borrow::Cow::from(rgba.into_raw()),
                                    };

                                    match arboard::Clipboard::new() {
                                        Ok(mut clipboard) => {
                                            if let Err(e) = clipboard.set_image(img_data) {
                                                *status_update = Some((
                                                    format!("Failed to copy to clipboard: {}", e),
                                                    egui::Color32::RED,
                                                ));
                                            } else {
                                                *status_update = Some((
                                                    "Avatar copied to clipboard!".to_string(),
                                                    egui::Color32::GREEN,
                                                ));
                                            }
                                        }
                                        Err(e) => {
                                            *status_update = Some((
                                                format!("Clipboard access failed: {}", e),
                                                egui::Color32::RED,
                                            ));
                                        }
                                    }
                                }
                                Err(e) => {
                                    *status_update = Some((
                                        format!("Failed to load image: {}", e),
                                        egui::Color32::RED,
                                    ));
                                }
                            },
                            Err(e) => {
                                *status_update = Some((
                                    format!("Failed to read avatar file: {}", e),
                                    egui::Color32::RED,
                                ));
                            }
                        }
                    }

                    if ui.button("Open Folder").clicked() {
                        #[cfg(target_os = "windows")]
                        {
                            let _ = std::process::Command::new("explorer")
                                .arg("/select,")
                                .arg(path_str.replace("/", "\\"))
                                .spawn();
                        }

                        #[cfg(target_os = "linux")]
                        {
                            if let Ok(abs_path) = std::fs::canonicalize(&path_str) {
                                let file_uri = format!("file://{}", abs_path.to_string_lossy());
                                // Try D-Bus for selection first (standard modern Linux)
                                let status = std::process::Command::new("dbus-send")
                                    .args(&[
                                        "--session",
                                        "--dest=org.freedesktop.FileManager1",
                                        "--type=method_call",
                                        "/org/freedesktop/FileManager1",
                                        "org.freedesktop.FileManager1.ShowItems",
                                        &format!("array:string:{}", file_uri),
                                        "string:\"\"",
                                    ])
                                    .status();

                                if status.is_err() || !status.unwrap().success() {
                                    // Fallback to just opening the parent directory
                                    if let Some(parent) = abs_path.parent() {
                                        let _ = std::process::Command::new("xdg-open")
                                            .arg(parent)
                                            .spawn();
                                    }
                                }
                            }
                        }

                        #[cfg(target_os = "macos")]
                        {
                            let _ = std::process::Command::new("open")
                                .arg("-R")
                                .arg(&path_str)
                                .spawn();
                        }
                    }
                });
            } else {
                ui.label(egui::RichText::new("No avatar selected").italics());
            }
            ui.horizontal(|ui| {
                if ui.button("Browse Avatar").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("image", &["png", "jpg", "jpeg", "webp"])
                        .pick_file()
                    {
                        if let Some(avatar_path) = app.update_avatar_from_file(path, character.id) {
                            character.avatar_path = Some(avatar_path);
                        }
                    }
                }
                if ui.button("Paste from Clipboard").clicked() {
                    match app.paste_avatar_from_clipboard(character.id) {
                        Ok(avatar_path) => {
                            character.avatar_path = Some(avatar_path);
                            *status_update = Some((
                                "Avatar pasted successfully!".to_string(),
                                egui::Color32::GREEN,
                            ));
                        }
                        Err(e) => {
                            *status_update = Some((e, egui::Color32::RED));
                        }
                    }
                }
            });

            ui.add_space(8.0);

            // Token Summary
            let t_name = if app.count_name_in_total {
                count_tokens(&character.name)
            } else {
                0
            };
            let t_first = if app.count_first_message_in_total {
                count_tokens(&character.first_message)
            } else {
                0
            };
            let t_pers = if app.count_personality_in_total {
                count_tokens(&character.personality)
            } else {
                0
            };
            let t_scen = if app.count_scenario_in_total {
                count_tokens(&character.scenario)
            } else {
                0
            };
            let t_ex = if app.count_example_in_total {
                count_tokens(&character.example_dialogue)
            } else {
                0
            };
            let t_title = if app.count_title_in_total {
                count_tokens(&character.char_title)
            } else {
                0
            };

            let total_tokens = t_name + t_first + t_pers + t_scen + t_ex + t_title;
            let perm_tokens = t_pers + t_scen;

            ui.label(
                egui::RichText::new(format!(
                    "Total Tokens: {} (Permanent: {})",
                    total_tokens, perm_tokens
                ))
                .strong()
                .color(egui::Color32::WHITE),
            );

            let c_name = if app.count_name_in_total {
                character.name.chars().count()
            } else {
                0
            };
            let c_first = if app.count_first_message_in_total {
                character.first_message.chars().count()
            } else {
                0
            };
            let c_pers = if app.count_personality_in_total {
                character.personality.chars().count()
            } else {
                0
            };
            let c_scen = if app.count_scenario_in_total {
                character.scenario.chars().count()
            } else {
                0
            };
            let c_ex = if app.count_example_in_total {
                character.example_dialogue.chars().count()
            } else {
                0
            };
            let c_title = if app.count_title_in_total {
                character.char_title.chars().count()
            } else {
                0
            };

            let total_chars = c_name + c_first + c_pers + c_scen + c_ex + c_title;
            let perm_chars = c_pers + c_scen;

            ui.label(
                egui::RichText::new(format!(
                    "Total Chars: {} (Permanent: {})",
                    total_chars, perm_chars
                ))
                .strong()
                .color(egui::Color32::WHITE),
            );
        });
    });
}
