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

    let gallery_dir = format!("data/gallery/{}", character.id);
    let _ = std::fs::create_dir_all(&gallery_dir);
    let mut refresh_gallery = false;

    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&gallery_dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    let path = entry.path();
                    if let Some(ext) = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|s| s.to_lowercase())
                    {
                        if ["png", "jpg", "jpeg", "webp"].contains(&ext.as_str()) {
                            files.push(path);
                        }
                    }
                }
            }
        }
    }
    files.sort();

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
            for path in &files {
                let path_str = path.to_string_lossy().to_string();
                let uri = crate::ui::utils::get_image_uri(&path_str);
                ui.ctx().forget_image(&uri);
            }
            ui.ctx().request_repaint();
        }
        if ui.button("📂 Open Folder").clicked() {
            #[cfg(target_os = "linux")]
            {
                if let Ok(abs_path) = std::fs::canonicalize(&gallery_dir) {
                    let _ = std::process::Command::new("xdg-open").arg(abs_path).spawn();
                }
            }
        }
    });
    ui.add_space(8.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            for path in &files {
                let path_str = path.to_string_lossy().to_string();
                // Use get_image_uri to handle caching and protocol
                let uri = crate::ui::utils::get_image_uri(&path_str);

                let size = 150.0;
                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());

                if ui.is_rect_visible(rect) {
                    crate::ui::widgets::paint_gallery_image(ui, rect, &uri, 4.0);
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
                    app.fullscreen_image = Some(uri.clone());
                    app.gallery_context = Some(
                        files
                            .iter()
                            .map(|p| crate::ui::utils::get_image_uri(&p.to_string_lossy()))
                            .collect(),
                    );
                }

                response.context_menu(|ui| {
                    if ui.button("🗑 Delete").clicked() {
                        let _ = std::fs::remove_file(path);
                        refresh_gallery = true;
                        ui.close_menu();
                    }
                });
            }
        });
    });

    if refresh_gallery {
        ui.ctx().request_repaint();
    }
}
