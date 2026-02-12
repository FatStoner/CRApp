use crate::ui::CrapApp;
use eframe::egui;

/// Render template-related popups
pub fn render_template_popups(ctx: &egui::Context, app: &mut CrapApp, state: &super::PopupState) {
    match state {
        super::PopupState::TemplateSelector => {
            render_template_selector(ctx, app);
        }

        super::PopupState::TemplatePreview {
            template_data,
            target_char_id,
        } => {
            render_template_preview(ctx, app, template_data.clone(), *target_char_id);
        }

        _ => {}
    }
}

fn render_template_selector(ctx: &egui::Context, app: &mut CrapApp) {
    use super::PopupState;

    let mut close = false;
    let mut selected_template = None;

    egui::Window::new("Select Template")
        .collapsible(false)
        .resizable(true)
        .min_width(300.0)
        .default_height(400.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label("Choose a template to apply:");
            ui.add_space(5.0);

            if app.templates.is_empty() {
                ui.colored_label(egui::Color32::YELLOW, "No templates found.");
                ui.add_space(5.0);
                if ui.button("➕ Go to Templates to create one").clicked() {
                    app.popup_state = PopupState::None;
                    app.request_switch_to_templates();
                }
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for template in &app.templates {
                        if ui.button(&template.name).clicked() {
                            selected_template = Some(template.clone());
                        }
                    }
                });
            }

            ui.add_space(10.0);
            if ui.button("Cancel").clicked() {
                close = true;
            }
        });

    if let Some(template) = selected_template {
        if let Some(char) = &app.selected_character {
            app.popup_state = PopupState::TemplatePreview {
                template_data: template,
                target_char_id: char.id,
            };
        } else {
            app.popup_state = PopupState::None;
        }
    } else if close {
        app.popup_state = PopupState::None;
    }
}

fn render_template_preview(
    ctx: &egui::Context,
    app: &mut CrapApp,
    mut template_data: crate::models::Template,
    target_char_id: i64,
) {
    use super::PopupState;

    let mut close = false;
    let mut apply = false;

    egui::Window::new("Preview & Edit Template")
        .collapsible(false)
        .resizable(true)
        .min_width(500.0)
        .default_height(600.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label(egui::RichText::new("Review the template data before applying.").strong());
            ui.label(
                "You can edit these fields here. They will overwrite the current character's data.",
            );
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label("Title:");
                ui.text_edit_singleline(&mut template_data.title);
                ui.add_space(5.0);

                ui.label("First Message:");
                ui.add(egui::TextEdit::multiline(&mut template_data.first_message).desired_rows(3));
                ui.add_space(5.0);

                ui.label("Personality:");
                ui.add(egui::TextEdit::multiline(&mut template_data.personality).desired_rows(3));
                ui.add_space(5.0);

                ui.label("Scenario:");
                ui.add(egui::TextEdit::multiline(&mut template_data.scenario).desired_rows(3));
                ui.add_space(5.0);

                ui.label("Example Dialogue:");
                ui.add(
                    egui::TextEdit::multiline(&mut template_data.example_dialogue).desired_rows(3),
                );
            });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Apply Template").clicked() {
                    apply = true;
                }
                if ui.button("Cancel").clicked() {
                    close = true;
                }
            });
        });

    if apply {
        if let Some(char) = &mut app.selected_character {
            if char.id == target_char_id {
                char.char_title = template_data.title;
                char.first_message = template_data.first_message;
                char.personality = template_data.personality;
                char.scenario = template_data.scenario;
                char.example_dialogue = template_data.example_dialogue;
            }
        }
        app.popup_state = PopupState::None;
    } else if close {
        app.popup_state = PopupState::None;
    } else {
        app.popup_state = PopupState::TemplatePreview {
            template_data,
            target_char_id,
        };
    }
}
