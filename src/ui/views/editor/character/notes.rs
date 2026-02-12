use crate::models::Character;
use crate::ui::CrapApp;
use eframe::egui;

pub fn render_notes_tab(app: &mut CrapApp, ui: &mut egui::Ui, character: &mut Character) {
    ui.label("Notes");
    let width = ui.ctx().screen_rect().width() * 2.0 / 3.0;
    let text_edit = egui::TextEdit::multiline(&mut character.author_notes).desired_width(width);
    {
        let mut layouter = crate::ui::spell_layout::create_spell_check_layouter(
            if app.enable_spell_check && !character.spell_check_overrides.contains("notes") {
                app.spell_checker.clone()
            } else {
                None
            },
            app.editor_search_query.clone(),
        );
        let response = ui.add(text_edit.layouter(&mut *layouter));
        crate::ui::widgets::track_text_selection(ui, &response);
        response.context_menu(|ui| {
            crate::ui::widgets::text_context_menu(ui, &mut character.author_notes, response.id);
        });
    }

    ui.add_space(16.0);
    ui.separator();
    ui.heading("Character Source URLs");
    ui.label(
        egui::RichText::new(
            "Links to where this character is hosted (e.g. spicychat.ai, janitor.ai)",
        )
        .size(11.0)
        .color(egui::Color32::GRAY),
    );

    // Ensure there is always one empty slot at the end
    if character.urls.is_empty() || !character.urls.last().unwrap().url.is_empty() {
        character.urls.push(crate::models::CharacterUrl {
            id: 0,
            character_id: character.id,
            url: String::new(),
            label: None,
        });
    }

    let mut urls_to_remove = Vec::new();

    // Iterate with index to allow removal
    for (i, char_url) in character.urls.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.label("URL:");
            let url_resp = ui.add(
                egui::TextEdit::singleline(&mut char_url.url)
                    .desired_width(250.0)
                    .hint_text("https://..."),
            );

            ui.label("Service:");
            let mut label_val = char_url.label.clone().unwrap_or_default();
            let _ = ui.add(
                egui::TextEdit::singleline(&mut label_val)
                    .desired_width(100.0)
                    .hint_text("Auto"),
            );

            if label_val.is_empty() {
                char_url.label = None;
            } else {
                char_url.label = Some(label_val);
            }

            if !char_url.url.is_empty() {
                if ui.button("🌐").on_hover_text("Open in Browser").clicked() {
                    ui.ctx().open_url(egui::OpenUrl::new_tab(&char_url.url));
                }
            }

            // Auto-fill label logic
            if url_resp.changed() || url_resp.lost_focus() {
                if char_url.label.is_none()
                    || char_url
                        .label
                        .as_ref()
                        .map(|s| s.is_empty())
                        .unwrap_or(true)
                {
                    // Try to extract domain
                    if let Ok(parsed) = url::Url::parse(&char_url.url) {
                        if let Some(host) = parsed.host_str() {
                            let clean = host.replace("www.", "");
                            let parts: Vec<&str> = clean.split('.').collect();
                            if !parts.is_empty() {
                                let service_name = parts[0];
                                let mut c = service_name.chars();
                                if let Some(f) = c.next() {
                                    let cap = f.to_uppercase().collect::<String>() + c.as_str();
                                    char_url.label = Some(cap);
                                } else {
                                    char_url.label = Some(service_name.to_string());
                                }
                            }
                        }
                    }
                }
            }

            if !char_url.url.is_empty() {
                if ui.button("🗑").clicked() {
                    urls_to_remove.push(i);
                }
            }
        });
    }

    // Remove deleted
    for i in urls_to_remove.iter().rev() {
        character.urls.remove(*i);
    }
}
