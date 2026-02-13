use crate::ui::{CrapApp, PopupState};
use eframe::egui;

pub fn render_update_popups(ctx: &egui::Context, app: &mut CrapApp, state: &PopupState) {
    if let PopupState::UpdateAvailable { version, tag } = state {
        let mut open = true;
        let version_clone = version.clone();
        let tag_clone = tag.clone();

        egui::Window::new("Update Available")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .open(&mut open)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.heading("A new version is available!");
                    ui.add_space(10.0);
                    ui.label(format!("Version {} is ready to download.", version_clone));
                    ui.add_space(20.0);

                    // Don't ask again checkbox
                    let mut dont_ask = !app.check_updates_at_start; // If it's false, then "don't ask" is true
                    if ui.checkbox(&mut dont_ask, "Don't ask me again").changed() {
                        app.set_check_updates_at_start(!dont_ask);
                    }

                    ui.add_space(20.0);

                    ui.horizontal(|ui| {
                        if ui.button("Later").clicked() {
                            app.popup_state = PopupState::None;
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Update Now").clicked() {
                                // Trigger update
                                let tag_target = tag_clone.clone();
                                app.popup_state = PopupState::Updating;
                                let tx = app.tx.clone();
                                let ctx_clone = ctx.clone();

                                std::thread::spawn(move || {
                                    // Notify start (optional, state already set)
                                    // tx.blocking_send(crate::ui::UiEvent::UpdateStarted);

                                    match crate::updater::perform_update(tag_target) {
                                        Ok(_) => {
                                            // On success, we restart
                                            let _ = crate::updater::restart_application();
                                        }
                                        Err(e) => {
                                            let _ = tx.blocking_send(
                                                crate::ui::UiEvent::UpdateFailed(e.to_string()),
                                            );
                                            ctx_clone.request_repaint();
                                        }
                                    }
                                });
                            }
                        });
                    });
                });
            });

        if !open {
            app.popup_state = PopupState::None;
        }
    } else if let PopupState::Updating = state {
        egui::Window::new("Updating")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .title_bar(false) // No closing while updating
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.spinner();
                    ui.add_space(10.0);
                    ui.heading("Downloading update...");
                    ui.label("Please wait while CRApp updates.");
                    ui.label("The application will restart automatically.");
                    ui.add_space(20.0);
                });
            });
    } else if let PopupState::UpdateError { error } = state {
        let mut open = true;
        egui::Window::new("Update Failed")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .open(&mut open)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.colored_label(egui::Color32::RED, "An error occurred during update:");
                    ui.add_space(10.0);
                    ui.label(error);
                    ui.add_space(20.0);
                    if ui.button("Close").clicked() {
                        app.popup_state = PopupState::None;
                    }
                    ui.add_space(10.0);
                });
            });

        if !open {
            app.popup_state = PopupState::None;
        }
    } else if let PopupState::UpToDate = state {
        let mut open = true;
        egui::Window::new("Updates")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .open(&mut open)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.heading("You are up to date!");
                    ui.add_space(10.0);
                    ui.label("You are running the latest version of CRApp.");
                    ui.add_space(20.0);
                    if ui.button("Close").clicked() {
                        app.popup_state = PopupState::None;
                    }
                    ui.add_space(10.0);
                });
            });

        if !open {
            app.popup_state = PopupState::None;
        }
    }
}
