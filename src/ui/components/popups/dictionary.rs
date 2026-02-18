use crate::ui::{CrapApp, PopupState};
use eframe::egui;

pub fn render_dictionary_edit_popup(ctx: &egui::Context, app: &mut CrapApp, state: &PopupState) {
    let mut open = true;
    let mut should_close = false;

    if let PopupState::DictionaryEdit { new_word_input } = state {
        // We need a local mutable copy of the input for the text field
        let mut local_input = new_word_input.clone();

        egui::Window::new("Edit Dictionary")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .fixed_size([300.0, 400.0])
            .show(ctx, |ui| {
                ui.label("Add or remove words from your personal dictionary.");
                ui.add_space(8.0);

                // Add New Word Section
                ui.horizontal(|ui| {
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut local_input)
                            .hint_text("Enter word to ignore...")
                            .desired_width(200.0),
                    );

                    let add_clicked = ui.button("Add").clicked();
                    let enter_pressed = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                    if (add_clicked || enter_pressed) && !local_input.trim().is_empty() {
                         if let Some(checker) = &app.spell_checker {
                            checker.add_word(local_input.trim());
                            local_input.clear();
                            
                            // Force repaint to show changes
                             // Clear cache to force re-check
                            let glitches_id = egui::Id::new("code_editor").with("glitches");
                             ui.data_mut(|d| {
                                d.remove_temp::<std::sync::Arc<(Vec<(usize, usize)>, Vec<usize>)>>(glitches_id)
                            });
                            ctx.request_repaint();
                        }
                    }
                });
                
                ui.separator();
                ui.add_space(8.0);

                // List of Words
                if let Some(checker) = &app.spell_checker {
                    let words = checker.get_ignored_words();
                    
                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            if words.is_empty() {
                                ui.label(egui::RichText::new("No custom words.").italics().color(egui::Color32::GRAY));
                            } else {
                                for word in &words {
                                    ui.horizontal(|ui| {
                                        if ui.button("🗑").on_hover_text("Remove word").clicked() {
                                            checker.remove_word(word);
                                            // Force repaint
                                            let glitches_id = egui::Id::new("code_editor").with("glitches");
                                             ui.data_mut(|d| {
                                                d.remove_temp::<std::sync::Arc<(Vec<(usize, usize)>, Vec<usize>)>>(glitches_id)
                                            });
                                            ctx.request_repaint();
                                        }
                                        ui.label(word);
                                    });
                                }
                            }
                        });
                }
                
                ui.add_space(10.0);
                ui.separator();
                if ui.button("Done").clicked() {
                    should_close = true;
                }
            });

        // Sync local input back to state if changed (though for this simple popup, we might just update it on every frame or keep it local)
        // With egui immediate mode, if we re-create the state every frame, the input clears. 
        // We need to update the state in the app.
        if local_input != *new_word_input {
             app.popup_state = PopupState::DictionaryEdit {
                new_word_input: local_input,
            };
        }
    }

    if !open || should_close {
        app.popup_state = PopupState::None;
    }
}
