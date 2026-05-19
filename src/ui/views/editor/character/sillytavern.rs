use crate::models::{count_tokens, Character};
use crate::ui::types::{EditorFontFamily, StTab};
use crate::ui::CrapApp;
use eframe::egui;
use egui_cosmic_text::cosmic_text::Family;
use std::collections::HashSet;

pub fn render_sillytavern_tab(
    app: &mut CrapApp,
    ui: &mut egui::Ui,
    character: &mut Character,
    status_update: &mut Option<(String, egui::Color32)>,
    tag_add_request: &mut Option<(i64, String, bool)>,
    tag_remove_request: &mut Option<(i64, i64, bool)>,
) {
    let font_family = match app.editor_font {
        EditorFontFamily::SansSerif => Family::SansSerif,
        EditorFontFamily::Serif => Family::Serif,
        EditorFontFamily::Monospace => Family::Monospace,
    };

    ui.horizontal(|ui| {
        ui.selectable_value(&mut app.active_st_tab, StTab::Main, "Main");
        ui.selectable_value(&mut app.active_st_tab, StTab::Advanced, "Advanced");
    });
    ui.separator();

    match app.active_st_tab {
        StTab::Main => render_st_main(
            app, ui, character, status_update,
            tag_add_request, tag_remove_request, font_family,
        ),
        StTab::Advanced => render_st_advanced(app, ui, character, status_update, font_family),
    }
}

fn st_field(
    app: &mut CrapApp,
    ui: &mut egui::Ui,
    label: &str,
    field_key: &str,
    value: &mut String,
    spell_check_overrides: &mut HashSet<String>,
    desired_lines: usize,
    status_update: &mut Option<(String, egui::Color32)>,
    font_family: Family,
) {
    let spell_check_key = format!("st_{}", field_key);
    let id = ui.make_persistent_id(format!("st_header_{}", field_key));
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
        .show_header(ui, |ui| {
            ui.label(label);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let mut ignore = spell_check_overrides.contains(&spell_check_key);
                if ui.checkbox(&mut ignore, "Ignore Spell Check").changed() {
                    if ignore {
                        spell_check_overrides.insert(spell_check_key.clone());
                    } else {
                        spell_check_overrides.remove(&spell_check_key);
                    }
                }
                if ui.small_button("Copy").clicked() {
                    ui.output_mut(|o| o.copied_text = value.clone());
                    *status_update = Some((format!("Copied {} to clipboard", label), egui::Color32::GREEN));
                }
                ui.label(
                    egui::RichText::new(format!(
                        "Tokens: {} | Chars: {}",
                        count_tokens(value),
                        value.chars().count()
                    ))
                    .size(12.0)
                    .color(egui::Color32::GRAY),
                );
            });
        })
        .body(|ui| {
            crate::ui::components::CodeEditor::new(value, &format!("st_{}_editor", field_key), font_family)
                .desired_lines(desired_lines)
                .font_size_offset(if app.editor_large_font { 2.0 } else { 0.0 })
                .bright_mode(app.editor_bright_mode)
                .highlight(app.editor_search_query.clone())
                .spell_check(
                    if app.enable_spell_check && !spell_check_overrides.contains(&spell_check_key) {
                        app.spell_checker.clone()
                    } else {
                        None
                    },
                )
                .show(
                    ui,
                    &mut app.cosmic_font_system,
                    &mut app.cosmic_swash_cache,
                    &mut app.cosmic_atlas,
                    &mut app.cosmic_editors,
                    &mut app.cosmic_clipboard,
                );
        });
    ui.add_space(6.0);
}

fn render_st_main(
    app: &mut CrapApp,
    ui: &mut egui::Ui,
    character: &mut Character,
    status_update: &mut Option<(String, egui::Color32)>,
    tag_add_request: &mut Option<(i64, String, bool)>,
    tag_remove_request: &mut Option<(i64, i64, bool)>,
    font_family: Family,
) {
    ui.horizontal(|ui| {
        let available_width = ui.available_width();
        let left_width = available_width * 0.66;

        ui.allocate_ui_with_layout(
            egui::vec2(left_width, ui.available_height()),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.label("Name");
                crate::ui::components::CodeEditor::new(&mut character.st_name, "st_name_editor", font_family)
                    .single_line()
                    .font_size_offset(if app.editor_large_font { 2.0 } else { 0.0 })
                    .bright_mode(app.editor_bright_mode)
                    .highlight(app.editor_search_query.clone())
                    .spell_check(None)
                    .show(ui, &mut app.cosmic_font_system, &mut app.cosmic_swash_cache,
                        &mut app.cosmic_atlas, &mut app.cosmic_editors, &mut app.cosmic_clipboard);
                ui.add_space(6.0);

                st_field(app, ui, "Greeting (First Message)", "first_mes", &mut character.st_first_mes, &mut character.spell_check_overrides, 10, status_update, font_family);
                st_field(app, ui, "Description", "description", &mut character.st_description, &mut character.spell_check_overrides, 10, status_update, font_family);
                st_field(app, ui, "Scenario", "scenario", &mut character.st_scenario, &mut character.spell_check_overrides, 8, status_update, font_family);
                render_alternate_greetings(app, ui, character, status_update, font_family);
                ui.add_space(8.0);
                render_st_tags(ui, app, character, tag_add_request, tag_remove_request);
            },
        );

        ui.add_space(8.0);

        ui.vertical(|ui| {
            render_st_avatar_panel(app, ui, character, status_update);
        });
    });
}

fn render_st_avatar_panel(
    app: &mut CrapApp,
    ui: &mut egui::Ui,
    character: &mut Character,
    status_update: &mut Option<(String, egui::Color32)>,
) {
    ui.label("Avatar");

    if let Some(path_str) = character.avatar_path.clone() {
        let uri = crate::ui::utils::get_image_uri(&path_str);
        let preview_width = ui.available_width() - 8.0;

        let base_blur = app.blur_all_images
            || (app.blur_all_nsfw && character.is_nsfw)
            || character.blur_avatar;
        let should_blur = if let Some(&override_val) = app.blur_overrides.get(&character.id) {
            override_val
        } else {
            base_blur
        };

        let response = ui.add(
            egui::Image::new(&uri)
                .rounding(egui::Rounding::same(4.0))
                .max_width(preview_width)
                .sense(egui::Sense::click()),
        );

        if should_blur {
            ui.painter().rect_filled(response.rect, 4.0, egui::Color32::from_black_alpha(255));
            ui.painter().text(
                response.rect.center(),
                egui::Align2::CENTER_CENTER,
                if character.is_nsfw { "NSFW" } else { "BLURRED" },
                egui::FontId::proportional(32.0),
                egui::Color32::WHITE,
            );
        }

        response.context_menu(|ui| {
            if ui.button("Copy to Clipboard").clicked() {
                copy_avatar_to_clipboard(&path_str, status_update);
                ui.close_menu();
            }
            if ui.button("Open Folder").clicked() {
                open_avatar_folder(&path_str);
                ui.close_menu();
            }
            if ui.button("Change Avatar").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("image", &["png", "jpg", "jpeg", "webp"])
                    .pick_file()
                {
                    if let Some(avatar_path) = app.update_avatar_from_file(path, character.id) {
                        character.avatar_path = Some(avatar_path);
                    }
                }
                ui.close_menu();
            }
            ui.separator();
            if should_blur {
                if ui.button("Unblur Image").clicked() {
                    app.toggle_character_blur(character.id);
                    ui.close_menu();
                }
            } else if ui.button("Blur Image").clicked() {
                app.toggle_character_blur(character.id);
                ui.close_menu();
            }
        });

        ui.label(&path_str);

        ui.horizontal(|ui| {
            if ui.button("Copy to Clipboard").clicked() {
                copy_avatar_to_clipboard(&path_str, status_update);
            }
            if ui.button("Open Folder").clicked() {
                open_avatar_folder(&path_str);
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
                    *status_update = Some(("Avatar pasted successfully!".to_string(), egui::Color32::GREEN));
                }
                Err(e) => {
                    *status_update = Some((e, egui::Color32::RED));
                }
            }
        }
    });

    ui.add_space(4.0);
    if ui.checkbox(&mut character.blur_avatar, "Blur Avatar").changed() {
        app.blur_overrides.remove(&character.id);
    }
    if ui.checkbox(&mut character.is_nsfw, "NSFW").changed() {
        app.blur_overrides.remove(&character.id);
    }

    ui.add_space(8.0);

    let total_tokens = count_tokens(&character.st_first_mes)
        + count_tokens(&character.st_description)
        + count_tokens(&character.st_scenario)
        + count_tokens(&character.st_personality)
        + count_tokens(&character.st_mes_example);

    let total_chars = character.st_first_mes.chars().count()
        + character.st_description.chars().count()
        + character.st_scenario.chars().count()
        + character.st_personality.chars().count()
        + character.st_mes_example.chars().count();

    ui.label(
        egui::RichText::new(format!("Total Tokens: {}", total_tokens))
            .strong()
            .color(egui::Color32::WHITE),
    );
    ui.label(
        egui::RichText::new(format!("Total Chars: {}", total_chars))
            .strong()
            .color(egui::Color32::WHITE),
    );
}

fn copy_avatar_to_clipboard(path_str: &str, status_update: &mut Option<(String, egui::Color32)>) {
    match std::fs::read(path_str) {
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
                            *status_update = Some((format!("Failed to copy to clipboard: {}", e), egui::Color32::RED));
                        } else {
                            *status_update = Some(("Avatar copied to clipboard!".to_string(), egui::Color32::GREEN));
                        }
                    }
                    Err(e) => {
                        *status_update = Some((format!("Clipboard access failed: {}", e), egui::Color32::RED));
                    }
                }
            }
            Err(e) => {
                *status_update = Some((format!("Failed to load image: {}", e), egui::Color32::RED));
            }
        },
        Err(e) => {
            *status_update = Some((format!("Failed to read avatar file: {}", e), egui::Color32::RED));
        }
    }
}

fn open_avatar_folder(path_str: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer")
            .arg("/select,")
            .arg(path_str.replace("/", "\\"))
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(abs_path) = std::fs::canonicalize(path_str) {
            let file_uri = format!("file://{}", abs_path.to_string_lossy());
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
                if let Some(parent) = abs_path.parent() {
                    let _ = std::process::Command::new("xdg-open").arg(parent).spawn();
                }
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg("-R").arg(path_str).spawn();
    }
}

fn render_alternate_greetings(
    app: &mut CrapApp,
    ui: &mut egui::Ui,
    character: &mut Character,
    status_update: &mut Option<(String, egui::Color32)>,
    font_family: Family,
) {
    let id = ui.make_persistent_id("st_alt_greetings_header");
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
        .show_header(ui, |ui| {
            ui.label("Alternate Greetings");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let spell_check_key = "st_alternate_greetings";
                let mut ignore = character.spell_check_overrides.contains(spell_check_key);
                if ui.checkbox(&mut ignore, "Ignore Spell Check").changed() {
                    if ignore {
                        character.spell_check_overrides.insert(spell_check_key.to_string());
                    } else {
                        character.spell_check_overrides.remove(spell_check_key);
                    }
                }
                ui.label(
                    egui::RichText::new(format!("{} greeting(s)", character.st_alternate_greetings.len()))
                        .size(11.0)
                        .color(egui::Color32::GRAY),
                );
            });
        })
        .body(|ui| {
            let mut to_remove: Option<usize> = None;
            for (idx, greeting) in character.st_alternate_greetings.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("#{}", idx + 1))
                            .size(11.0)
                            .color(egui::Color32::from_rgb(100, 180, 255)),
                    );
                    if ui.small_button("✖ Remove").clicked() {
                        to_remove = Some(idx);
                    }
                    ui.label(
                        egui::RichText::new(format!(
                            "Tokens: {} | Chars: {}",
                            count_tokens(greeting),
                            greeting.chars().count()
                        ))
                        .size(10.0)
                        .color(egui::Color32::GRAY),
                    );
                });
                crate::ui::components::CodeEditor::new(
                    greeting,
                    &format!("st_alt_greeting_{}_editor", idx),
                    font_family,
                )
                .desired_lines(6)
                .font_size_offset(if app.editor_large_font { 2.0 } else { 0.0 })
                .bright_mode(app.editor_bright_mode)
                .highlight(app.editor_search_query.clone())
                .spell_check(
                    if app.enable_spell_check
                        && !character.spell_check_overrides.contains("st_alternate_greetings")
                    {
                        app.spell_checker.clone()
                    } else {
                        None
                    },
                )
                .show(ui, &mut app.cosmic_font_system, &mut app.cosmic_swash_cache,
                    &mut app.cosmic_atlas, &mut app.cosmic_editors, &mut app.cosmic_clipboard);
                ui.add_space(4.0);
            }
            if let Some(idx) = to_remove {
                character.st_alternate_greetings.remove(idx);
            }
            if ui.button("+ Add Alternate Greeting").clicked() {
                character.st_alternate_greetings.push(String::new());
                *status_update = Some(("Added alternate greeting".to_string(), egui::Color32::GREEN));
            }
        });
    ui.add_space(6.0);
}

fn render_st_tags(
    ui: &mut egui::Ui,
    app: &mut CrapApp,
    character: &mut Character,
    tag_add_request: &mut Option<(i64, String, bool)>,
    tag_remove_request: &mut Option<(i64, i64, bool)>,
) {
    egui::CollapsingHeader::new("Tags & Metadata")
        .default_open(true)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("CRApp Tags")
                        .strong()
                        .color(egui::Color32::from_rgb(100, 150, 255)),
                );
                ui.horizontal(|ui| {
                    let mut app_tags_sorted: Vec<_> = character.app_tags.iter().collect();
                    app_tags_sorted.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                    for tag in app_tags_sorted {
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(50, 80, 150))
                            .rounding(12.0)
                            .inner_margin(4.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(&tag.name).color(egui::Color32::WHITE).size(12.0));
                                    if ui.small_button("x").clicked() {
                                        *tag_remove_request = Some((character.id, tag.id, false));
                                    }
                                });
                            });
                    }
                });
                ui.horizontal(|ui| {
                    let response = ui.text_edit_singleline(&mut app.app_tag_input);
                    if (ui.button("Add").clicked()
                        || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                        && !app.app_tag_input.is_empty()
                    {
                        *tag_add_request = Some((character.id, app.app_tag_input.clone(), false));
                        app.app_tag_input.clear();
                        response.request_focus();
                    }
                });

                ui.add_space(8.0);

                ui.label(
                    egui::RichText::new("External Tags")
                        .strong()
                        .color(egui::Color32::GRAY),
                );
                ui.horizontal(|ui| {
                    let mut ext_tags_sorted: Vec<_> = character.external_tags.iter().collect();
                    ext_tags_sorted.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                    for tag in ext_tags_sorted {
                        egui::Frame::none()
                            .fill(egui::Color32::from_gray(80))
                            .rounding(12.0)
                            .inner_margin(4.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(&tag.name).color(egui::Color32::WHITE).size(12.0));
                                    if ui.small_button("x").clicked() {
                                        *tag_remove_request = Some((character.id, tag.id, true));
                                    }
                                });
                            });
                    }
                });
                ui.horizontal(|ui| {
                    let response = ui.text_edit_singleline(&mut app.ext_tag_input);
                    if (ui.button("Add").clicked()
                        || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                        && !app.ext_tag_input.is_empty()
                    {
                        *tag_add_request = Some((character.id, app.ext_tag_input.clone(), true));
                        app.ext_tag_input.clear();
                        response.request_focus();
                    }
                });
            });
        });
}

fn render_st_advanced(
    app: &mut CrapApp,
    ui: &mut egui::Ui,
    character: &mut Character,
    status_update: &mut Option<(String, egui::Color32)>,
    font_family: Family,
) {
    // 1. Personality Summary
    st_field(app, ui, "Personality Summary", "personality", &mut character.st_personality, &mut character.spell_check_overrides, 4, status_update, font_family);

    // 2. Depth Prompt
    egui::CollapsingHeader::new("Depth Prompt (Character Note)")
        .default_open(true)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new("Injected into context at the specified depth. Useful for reinforcing character behavior.")
                    .size(11.0)
                    .color(egui::Color32::GRAY),
            );
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Depth:");
                let mut depth_i32 = character.st_depth_prompt_depth as i32;
                if ui.add(egui::DragValue::new(&mut depth_i32).range(0..=100)).changed() {
                    character.st_depth_prompt_depth = depth_i32 as i64;
                }
                ui.add_space(16.0);
                ui.label("Role:");
                egui::ComboBox::from_id_source("st_depth_role_combo")
                    .selected_text(&character.st_depth_prompt_role)
                    .show_ui(ui, |ui| {
                        for role in &["system", "user", "assistant"] {
                            ui.selectable_value(&mut character.st_depth_prompt_role, role.to_string(), *role);
                        }
                    });
            });

            ui.add_space(4.0);
            st_field(app, ui, "Depth Prompt Text", "depth_prompt", &mut character.st_depth_prompt, &mut character.spell_check_overrides, 6, status_update, font_family);
        });

    ui.add_space(8.0);

    // 3. Example Dialogue
    st_field(app, ui, "Example Dialogue", "mes_example", &mut character.st_mes_example, &mut character.spell_check_overrides, 8, status_update, font_family);

    ui.separator();

    // 4. Creator Metadata
    egui::CollapsingHeader::new("Creator Metadata")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Creator:");
                ui.add(egui::TextEdit::singleline(&mut character.st_creator).desired_width(200.0).hint_text("Author name..."));
                ui.add_space(16.0);
                ui.label("Character Version:");
                ui.add(egui::TextEdit::singleline(&mut character.st_character_version).desired_width(120.0).hint_text("e.g. 1.0"));
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Talkativeness:");
                let mut talk_f32 = character.st_talkativeness as f32;
                if ui.add(egui::Slider::new(&mut talk_f32, 0.0..=1.0).step_by(0.05).fixed_decimals(2)).changed() {
                    character.st_talkativeness = talk_f32 as f64;
                }
                ui.label(egui::RichText::new("(0 = quiet, 1 = very talkative)").size(11.0).color(egui::Color32::GRAY));
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("World / Lorebook Name:");
                ui.add(egui::TextEdit::singleline(&mut character.st_world).desired_width(250.0).hint_text("Linked world name..."));
            });

            ui.add_space(4.0);

            st_field(app, ui, "Creator's Notes", "creator_notes", &mut character.st_creator_notes, &mut character.spell_check_overrides, 4, status_update, font_family);
        });
}
