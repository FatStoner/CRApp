use crate::models::{count_tokens, Lorebook};
use crate::ui::types::EditorFontFamily;
use crate::ui::CrapApp;
use eframe::egui;
use egui_cosmic_text::cosmic_text::Family;

pub fn render_lorebook_metadata(
    app: &mut CrapApp,
    ui: &mut egui::Ui,
    book: &mut Lorebook,
    status_update: &mut Option<(String, egui::Color32)>,
    tag_add_request: &mut Option<(i64, String)>,
    tag_remove_request: &mut Option<(i64, i64)>,
) {
    let font_family = match app.editor_font {
        EditorFontFamily::SansSerif => Family::SansSerif,
        EditorFontFamily::Serif => Family::Serif,
        EditorFontFamily::Monospace => Family::Monospace,
    };

    egui::ScrollArea::vertical()
        .max_height(ui.available_height() * 0.45)
        .id_source("lorebook_metadata_scroll")
        .show(ui, |ui| {
            let total_width = ui.available_width();
            let right_width = (total_width * 0.35).max(160.0).min(300.0);
            let left_width = total_width - right_width - ui.spacing().item_spacing.x;

            ui.horizontal_top(|ui| {
                // Left Column: Basic Data
                ui.allocate_ui_with_layout(
                    egui::vec2(left_width, 10.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.label("Title");
                        crate::ui::components::CodeEditor::new(
                            &mut book.title,
                            "lorebook_title_editor",
                            font_family,
                        )
                        .single_line()
                        .font_size_offset(if app.editor_large_font { 2.0 } else { 0.0 })
                        .bright_mode(app.editor_bright_mode)
                        .highlight(app.editor_search_query.clone())
                        .spell_check(None)
                        .show(
                            ui,
                            &mut app.cosmic_font_system,
                            &mut app.cosmic_swash_cache,
                            &mut app.cosmic_atlas,
                            &mut app.cosmic_editors,
                            &mut app.cosmic_clipboard,
                        );
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            ui.label("Description");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "Tokens: {} | Chars: {}",
                                            count_tokens(&book.content),
                                            book.content.chars().count()
                                        ))
                                        .size(12.0)
                                        .color(egui::Color32::GRAY),
                                    );
                                },
                            );
                        });

                        crate::ui::components::CodeEditor::new(
                            &mut book.content,
                            "lorebook_content_editor",
                            font_family,
                        )
                        .desired_lines(15)
                        .font_size_offset(if app.editor_large_font { 2.0 } else { 0.0 })
                        .bright_mode(app.editor_bright_mode)
                        .highlight(app.editor_search_query.clone())
                        .spell_check(if app.enable_spell_check {
                            app.spell_checker.clone()
                        } else {
                            None
                        })
                        .show(
                            ui,
                            &mut app.cosmic_font_system,
                            &mut app.cosmic_swash_cache,
                            &mut app.cosmic_atlas,
                            &mut app.cosmic_editors,
                            &mut app.cosmic_clipboard,
                        );
                        ui.add_space(8.0);

                        // Tags Section
                        ui.label("Tags:");
                        ui.horizontal_wrapped(|ui| {
                            for tag in &book.tags {
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new(&tag.name)
                                                .color(egui::Color32::WHITE)
                                                .size(12.0),
                                        );
                                        if ui.small_button("x").clicked() {
                                            *tag_remove_request = Some((book.id, tag.id));
                                        }
                                    });
                                });
                            }
                        });
                        ui.horizontal(|ui| {
                            let response = ui.add(
                                egui::TextEdit::singleline(&mut app.app_tag_input)
                                    .desired_width(120.0),
                            );
                            if (ui.button("Add").clicked()
                                || (response.lost_focus()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                                && !app.app_tag_input.is_empty()
                            {
                                *tag_add_request = Some((book.id, app.app_tag_input.clone()));
                                app.app_tag_input.clear();
                                response.request_focus();
                            }
                        });

                        ui.add_space(8.0);
                    },
                );

                // Right Column: Cover Image
                ui.allocate_ui_with_layout(
                    egui::vec2(right_width, 10.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        let max_total_h = ui.ctx().screen_rect().height() * 0.333;
                        ui.set_max_height(max_total_h);

                        ui.label(egui::RichText::new("Cover Image").strong());

                        if let Some(path_str) = &book.cover_path {
                            let uri = crate::ui::utils::get_image_uri(path_str);

                            // Total height is limited to 1/3 of screen.
                            // Subtracting ~80px for labels and buttons.
                            let image_max_h = (max_total_h - 80.0).max(100.0);
                            let preview_width = ui.available_width();

                            ui.add(
                                egui::Image::new(uri)
                                    .rounding(egui::Rounding::same(4.0))
                                    .max_height(image_max_h)
                                    .max_width(preview_width),
                            );

                            if app.blur_all_images {
                                let rect = ui.min_rect(); // Get the rect of the image we just added
                                                          // Actually, min_rect might be too big if other things are there.
                                                          // But here we just added the image.
                                                          // A better way is to allocate the response or use put.
                                                          // Simple overlay:
                                let overlay_rect = ui.min_rect().intersect(ui.clip_rect());
                                // Approximation
                                // BUT ui.min_rect() covers everything in the scope so far? No, `ui` here is the top_down layout.
                                // The `ui.add(Image)` returns a response.
                                // We can use that response.rect!
                            }
                        } else {
                            // ...
                        }

                        // Retrying with correct logic using response
                        if let Some(path_str) = &book.cover_path {
                            let uri = crate::ui::utils::get_image_uri(path_str);
                            let image_max_h = (max_total_h - 80.0).max(100.0);
                            let preview_width = ui.available_width();

                            let response = ui.add(
                                egui::Image::new(uri)
                                    .rounding(egui::Rounding::same(4.0))
                                    .max_height(image_max_h)
                                    .max_width(preview_width),
                            );

                            if app.blur_all_images {
                                ui.painter().rect_filled(
                                    response.rect,
                                    4.0,
                                    egui::Color32::from_black_alpha(255),
                                );
                                ui.painter().text(
                                    response.rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "BLURRED",
                                    egui::FontId::proportional(20.0),
                                    egui::Color32::WHITE,
                                );
                            }

                            ui.add_space(4.0);
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing.x = 0.0;
                                ui.label(egui::RichText::new("Path: ").weak());
                                ui.label(egui::RichText::new(path_str).weak().italics());
                            });
                        } else {
                            // Empty state placeholder
                            let (rect, _) = ui.allocate_exact_size(
                                egui::vec2(100.0, 100.0),
                                egui::Sense::hover(),
                            );
                            ui.painter().rect_stroke(
                                rect,
                                4.0,
                                (1.0, egui::Color32::from_gray(60)),
                            );
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "No Cover",
                                egui::FontId::proportional(14.0),
                                egui::Color32::from_gray(100),
                            );
                        }

                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 8.0;
                            if ui.button("Browse...").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("image", &["png", "jpg", "jpeg"])
                                    .pick_file()
                                {
                                    if let Some(cover_path) =
                                        app.update_lorebook_cover(book.id, path)
                                    {
                                        book.cover_path = Some(cover_path);
                                    }
                                }
                            }
                            if ui.button("Paste").clicked() {
                                match arboard::Clipboard::new() {
                                    Ok(mut clipboard) => {
                                        if let Ok(img_data) = clipboard.get_image() {
                                            let width = img_data.width as u32;
                                            let height = img_data.height as u32;
                                            let bytes = img_data.bytes.into_owned();
                                            if let Some(image_buffer) = image::ImageBuffer::<
                                                image::Rgba<u8>,
                                                Vec<u8>,
                                            >::from_raw(
                                                width, height, bytes
                                            ) {
                                                let timestamp =
                                                    chrono::Utc::now().timestamp_millis();
                                                let filename =
                                                    format!("pasted_cover_{}.png", timestamp);
                                                let dest_dir = std::path::Path::new("data/covers");
                                                let _ = std::fs::create_dir_all(dest_dir);
                                                let dest_path = dest_dir.join(&filename);
                                                if let Ok(_) = image_buffer.save(&dest_path) {
                                                    book.cover_path = Some(
                                                        dest_path.to_string_lossy().to_string(),
                                                    );
                                                    *status_update = Some((
                                                        "Cover pasted!".to_string(),
                                                        egui::Color32::GREEN,
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                    Err(_) => {}
                                }
                            }
                            if book.cover_path.is_some() {
                                if ui.button("Remove").clicked() {
                                    book.cover_path = None;
                                }
                            }
                        });
                    },
                );
            });
        });
}
