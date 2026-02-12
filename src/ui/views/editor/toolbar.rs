use crate::models::Character;
use crate::ui::CrapApp;
use eframe::egui;

/// Result of toolbar interactions
pub struct ToolbarAction {
    pub back_history_requested: bool,
    pub back_to_collection: Option<Option<i64>>,
    pub save_requested: bool,
    pub template_requested: bool,
}

impl Default for ToolbarAction {
    fn default() -> Self {
        Self {
            back_history_requested: false,
            back_to_collection: None,
            save_requested: false,
            template_requested: false,
        }
    }
}

/// Renders the top toolbar with navigation, export/import, and save buttons
pub fn render_toolbar(
    ui: &mut egui::Ui,
    app: &mut CrapApp,
    character: &Character,
    is_dirty: bool,
    trigger_import: &mut bool,
) -> ToolbarAction {
    let mut action = ToolbarAction::default();

    ui.horizontal(|ui| {
        let back_btn = ui.button("⬅ Back");
        if back_btn.clicked() {
            action.back_history_requested = true;
        }
        back_btn.context_menu(|ui| {
            ui.label("Navigation History");
            ui.separator();
            let history_len = app.navigation_history.len();
            let start_index = history_len.saturating_sub(5);
            let history_items: Vec<(usize, String)> = app
                .navigation_history
                .iter()
                .enumerate()
                .skip(start_index)
                .rev()
                .map(|(i, state)| (i, app.describe_state(state)))
                .collect();

            for (i, label) in history_items {
                if ui.button(label).clicked() {
                    if is_dirty {
                        app.popup_state = crate::ui::PopupState::UnsavedChanges {
                            target: crate::ui::AppAction::GoToHistory(i),
                        };
                    } else {
                        app.go_to_history(i);
                    }
                    ui.close_menu();
                }
            }
            if history_len == 0 {
                ui.label(egui::RichText::new("No history").italics().weak());
            }
        });
        if ui.button("⬆ Up").clicked() {
            action.back_to_collection = Some(character.collection_id);
        }
        // Handle Esc key for Back navigation
        if ui.memory(|m| m.focused().is_none()) && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            action.back_to_collection = Some(character.collection_id);
        }
        ui.heading(format!("Edit Character ({})", character.name));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Export menu
            super::export::render_export_menu(ui, character);
            // Import menu
            super::export::render_import_menu(ui, character, &app.tx, trigger_import);

            ui.add_space(10.0);
            if ui.button("APPLY TEMPLATE").clicked() {
                action.template_requested = true;
            }

            ui.add_space(10.0);
            if app.is_saving {
                ui.spinner();
                ui.label("Saving...");
            } else {
                let mut save_color = ui.visuals().widgets.inactive.bg_fill;
                if is_dirty {
                    save_color = egui::Color32::from_rgb(200, 100, 50); // Orange/Red
                }

                // Check for Ctrl+S
                if ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::S)) {
                    action.save_requested = true;
                }

                if ui
                    .add(egui::Button::new(egui::RichText::new("SAVE").strong()).fill(save_color))
                    .clicked()
                {
                    action.save_requested = true;
                }
                if let Some((msg, color)) = &app.status_message {
                    ui.colored_label(*color, msg);
                }
            }
        });
    });

    action
}
