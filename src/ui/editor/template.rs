use crate::ui::CrapApp;
use eframe::egui;

pub fn render_template_editor(app: &mut CrapApp, ui: &mut egui::Ui) {
    let mut save = false;
    let mut delete = false;

    // Use take/replace pattern
    if let Some(mut template) = app.selected_template.take() {
        ui.horizontal(|ui| {
            ui.heading("Editing Template");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🗑 Delete").clicked() {
                    delete = true;
                }

                let is_dirty = if template.id == 0 {
                    true
                } else {
                    if let Some(original) = app.templates.iter().find(|t| t.id == template.id) {
                        !template.content_eq(original)
                    } else {
                        true
                    }
                };

                let mut save_color = ui.visuals().widgets.inactive.bg_fill;
                if is_dirty {
                    save_color = egui::Color32::from_rgb(200, 100, 50); // Orange/Red
                }

                // Ctrl+S
                if ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::S)) {
                    save = true;
                }

                if ui
                    .add(
                        egui::Button::new(egui::RichText::new("💾 Save").strong()).fill(save_color),
                    )
                    .clicked()
                {
                    save = true;
                }
            });
        });
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut template.name);
            });
            ui.end_row();

            ui.horizontal(|ui| {
                ui.label("Title:");
                ui.text_edit_singleline(&mut template.title);
            });
            ui.end_row();

            ui.label("First Message:");
            ui.add(
                egui::TextEdit::multiline(&mut template.first_message)
                    .desired_rows(4)
                    .desired_width(f32::INFINITY),
            );
            ui.end_row();

            ui.label("Personality:");
            ui.add(
                egui::TextEdit::multiline(&mut template.personality)
                    .desired_rows(4)
                    .desired_width(f32::INFINITY),
            );
            ui.end_row();

            ui.label("Scenario:");
            ui.add(
                egui::TextEdit::multiline(&mut template.scenario)
                    .desired_rows(4)
                    .desired_width(f32::INFINITY),
            );
            ui.end_row();

            ui.label("Example Dialogue:");
            ui.add(
                egui::TextEdit::multiline(&mut template.example_dialogue)
                    .desired_rows(8)
                    .desired_width(f32::INFINITY),
            );
            ui.end_row();
        });

        // Handle Actions
        if save {
            app.save_template(template.clone());
        }
        if delete {
            app.popup_state = crate::ui::PopupState::DeleteTemplateConfirmation {
                id: template.id,
                name: template.name.clone(),
            };
        }

        // Put it back
        app.selected_template = Some(template);
    } else {
        ui.centered_and_justified(|ui| {
            ui.label("No Template Selected");
        });
    }
}
