use crate::ui::{CrapApp, PopupState};
use eframe::egui;

pub fn render_update_popups(ctx: &egui::Context, app: &mut CrapApp, state: &PopupState) {
    if let PopupState::UpdateAvailable { version } = state {
        let mut open = true;
        let version_clone = version.clone();

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
                                let version_target = version_clone.clone();
                                std::thread::spawn(move || {
                                    if let Err(e) =
                                        crate::updater::perform_update(Some(version_target))
                                    {
                                        eprintln!("Update failed: {}", e);
                                    } else {
                                        // On success, we restart
                                        let _ = crate::updater::restart_application();
                                    }
                                });
                                app.popup_state = PopupState::None;
                                // Maybe show a "Updating..." modal? For now, the console will show progress and then it restarts.
                                // Improvements: clearer feedback during update.
                            }
                        });
                    });
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
