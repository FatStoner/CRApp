use crate::ui::CrapApp;
use eframe::egui;

/// Render deletion confirmation popups
pub fn render_deletion_popups(ctx: &egui::Context, app: &mut CrapApp, state: &super::PopupState) {
    match state {
        super::PopupState::DeleteWarning { _id: _, count } => {
            let mut close = false;
            egui::Window::new("Cannot Delete Folder")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.colored_label(egui::Color32::RED, "Warning: Folder is not empty.");
                    ui.add_space(5.0);
                    ui.label(format!(
                        "This folder contains {} character(s) or subfolder(s).",
                        count
                    ));
                    ui.label("You must move or delete all contents before deleting this folder.");
                    ui.add_space(10.0);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        if ui.button("OK").clicked() {
                            close = true;
                        }
                    });
                });
            if close {
                app.popup_state = super::PopupState::None;
            }
        }

        super::PopupState::DeleteCharacterConfirmation { id, name } => {
            let mut close = false;
            egui::Window::new("Delete Character?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label(format!("Are you sure you want to delete '{}'?", name));
                    ui.colored_label(egui::Color32::RED, "This action cannot be undone.");
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui
                            .button(egui::RichText::new("Delete").color(egui::Color32::RED))
                            .clicked()
                        {
                            app.delete_character(*id);
                            close = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });
            if close {
                app.popup_state = super::PopupState::None;
            }
        }

        super::PopupState::DeleteLorebookConfirmation { id, title } => {
            let mut close = false;
            egui::Window::new("Delete Lorebook?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label(format!("Are you sure you want to delete '{}'?", title));
                    ui.colored_label(egui::Color32::RED, "This action cannot be undone.");
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui
                            .button(egui::RichText::new("Delete").color(egui::Color32::RED))
                            .clicked()
                        {
                            app.delete_lorebook(*id);
                            close = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });
            if close {
                app.popup_state = super::PopupState::None;
            }
        }

        super::PopupState::DeleteLorebookEntryConfirmation {
            id,
            lorebook_id,
            name,
        } => {
            let mut close = false;
            egui::Window::new("Delete Entry?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label(format!("Are you sure you want to delete '{}'?", name));
                    ui.colored_label(egui::Color32::RED, "This action cannot be undone.");
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui
                            .button(egui::RichText::new("Delete").color(egui::Color32::RED))
                            .clicked()
                        {
                            app.delete_lorebook_entry_async(*id, *lorebook_id);
                            close = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });
            if close {
                app.popup_state = super::PopupState::None;
            }
        }

        super::PopupState::DeleteTemplateConfirmation { id, name } => {
            let mut close = false;
            egui::Window::new("Delete Template?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label(format!("Are you sure you want to delete '{}'?", name));
                    ui.colored_label(egui::Color32::RED, "This action cannot be undone.");
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui
                            .button(egui::RichText::new("Delete").color(egui::Color32::RED))
                            .clicked()
                        {
                            app.delete_template(*id);
                            close = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });
            if close {
                app.popup_state = super::PopupState::None;
            }
        }

        super::PopupState::DeleteGalleryImageConfirmation { path } => {
            let mut close = false;
            let mut deleted = false;
            egui::Window::new("Delete Image?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label("Are you sure you want to delete this image?");
                    ui.colored_label(egui::Color32::RED, "This action cannot be undone.");
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui
                            .button(egui::RichText::new("Delete").color(egui::Color32::RED))
                            .clicked()
                        {
                            let _ = std::fs::remove_file(path);
                            // Clear cache
                            let uri = crate::ui::utils::get_image_uri(path);
                            ctx.forget_image(&uri);

                            deleted = true;
                            close = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });

            if deleted {
                app.set_status("Image deleted".to_string(), egui::Color32::GREEN);
                ctx.request_repaint();
                app.popup_state = super::PopupState::None;
            } else if close {
                app.popup_state = super::PopupState::None;
            }
        }

        _ => {}
    }
}
