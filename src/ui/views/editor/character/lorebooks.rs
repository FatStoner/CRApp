use crate::models::Character;
use crate::ui::CrapApp;
use eframe::egui;

pub fn render_lorebooks_tab(
    app: &mut CrapApp,
    ui: &mut egui::Ui,
    character: &mut Character,
    toggle_requests: &mut Vec<(i64, i64, bool)>,
) {
    ui.label("Select relevant lorebooks:");
    let mut go_to_lorebook = None;
    for lore in &app.lorebooks {
        ui.horizontal(|ui| {
            let mut is_linked = app.lore_links.contains(&lore.id);
            if ui.checkbox(&mut is_linked, &lore.title).clicked() {
                if character.id != 0 {
                    toggle_requests.push((character.id, lore.id, is_linked));
                }
            }
            if ui
                .small_button("➡")
                .on_hover_text("Go to Lorebook")
                .clicked()
            {
                go_to_lorebook = Some(lore.clone());
            }
        });
    }

    if let Some(target_lore) = go_to_lorebook {
        // Temporarily restore character ownership so push_history sees it
        app.selected_character = Some(character.clone());
        app.load_lorebook(target_lore.id);
        // Navigation complete, mode switched.
    }
}
