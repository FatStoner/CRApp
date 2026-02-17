use crate::models::Character;
use crate::ui::CrapApp;
use eframe::egui;

pub fn render_gallery_tab(
    app: &mut CrapApp,
    ui: &mut egui::Ui,
    character: &mut Character,
    status_update: &mut Option<(String, egui::Color32)>,
) {
    ui.heading("Character Gallery");
    ui.label(
        egui::RichText::new("Images associated with this character.")
            .size(11.0)
            .color(egui::Color32::GRAY),
    );
    ui.add_space(8.0);

    // Async Loading Logic
    let files = if let Some(cached_files) = app.gallery_cache.get(&character.id) {
        cached_files.clone()
    } else {
        // Trigger load if not already loading
        if !app.gallery_loading.contains(&character.id) {
            app.gallery_loading.insert(character.id);
            app.load_gallery_images_async(character.id);
        }
        std::sync::Arc::new(Vec::new())
    };

    let is_loading = app.gallery_loading.contains(&character.id);

    // Add Image Button
    ui.horizontal(|ui| {
        if ui.button("➕ Add Image").clicked() {
            app.add_gallery_image_async(character.id);
        }
        if ui.button("📋 Paste").clicked() {
            match app.paste_gallery_image_from_clipboard(character.id) {
                Ok(_) => {
                    *status_update =
                        Some(("Image pasted to gallery!".to_string(), egui::Color32::GREEN));
                    ui.ctx().request_repaint();
                }
                Err(e) => {
                    *status_update = Some((e, egui::Color32::RED));
                }
            }
        }
        if ui.button("🔄 Refresh").clicked() {
            // Clear cache and reload
            app.gallery_cache.remove(&character.id);
            app.gallery_loading.insert(character.id);
            app.load_gallery_images_async(character.id);

            // Create a temp list of paths to forget images, using what we had
            for img in files.iter() {
                ui.ctx().forget_image(&img.thumbnail_uri);
                let full_uri = crate::ui::utils::get_image_uri(&img.path);
                ui.ctx().forget_image(&full_uri);
            }
            ui.ctx().request_repaint();
        }
        if ui.button("📂 Open Folder").clicked() {
            #[cfg(target_os = "linux")]
            {
                let gallery_dir = format!("data/gallery/{}", character.id);
                let _ = std::fs::create_dir_all(&gallery_dir);
                if let Ok(abs_path) = std::fs::canonicalize(&gallery_dir) {
                    let _ = std::process::Command::new("xdg-open").arg(abs_path).spawn();
                }
            }
        }

        if is_loading {
            ui.spinner();
            ui.label("Loading...");
        }
    });
    ui.add_space(8.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            for img in files.iter() {
                let size = 150.0;
                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());

                if ui.is_rect_visible(rect) {
                    crate::ui::widgets::paint_gallery_image(ui, rect, &img.thumbnail_uri, 4.0);
                }

                // Hover
                if response.hovered() {
                    ui.painter().rect_stroke(
                        rect,
                        4.0,
                        egui::Stroke::new(2.0, egui::Color32::WHITE),
                    );
                }

                if response.clicked() {
                    let full_uri = crate::ui::utils::get_image_uri(&img.path);
                    app.fullscreen_image = Some(full_uri);
                    // Update gallery context for lightbox navigation
                    app.gallery_context = Some(
                        files
                            .iter()
                            .map(|i| crate::ui::utils::get_image_uri(&i.path))
                            .collect(),
                    );
                    app.gallery_zoom = 1.0;
                    app.gallery_pan = egui::vec2(0.0, 0.0);
                }

                response.context_menu(|ui| {
                    if ui.button("🗑 Delete").clicked() {
                        app.popup_state = crate::ui::PopupState::DeleteGalleryImageConfirmation {
                            path: img.path.clone(),
                        };
                        ui.close_menu();
                    }
                });
            }
        });
    });
}
