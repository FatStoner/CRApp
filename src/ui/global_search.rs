use crate::ui::{AppMode, CrapApp, SearchResultKind};
use eframe::egui;

pub fn render_deep_search(app: &mut CrapApp, ui: &mut egui::Ui) {
    ui.heading("Deep Global Search");
    ui.horizontal(|ui| {
        ui.label("Query:");
        if ui
            .text_edit_singleline(&mut app.deep_search_query)
            .lost_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter))
        {
            app.perform_deep_search();
        }
        if ui.button("Search").clicked() {
            app.perform_deep_search();
        }
    });

    // Folder filter
    ui.horizontal(|ui| {
        ui.label("Folder:");

        egui::ComboBox::from_id_salt("deep_search_folder_filter")
            .selected_text(if let Some(coll_id) = app.deep_search_filter_collection {
                app.collections
                    .iter()
                    .find(|c| c.id == coll_id)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Unknown")
            } else {
                "All Folders"
            })
            .show_ui(ui, |ui| {
                // "All Folders" option
                if ui
                    .selectable_value(&mut app.deep_search_filter_collection, None, "All Folders")
                    .clicked()
                {
                    // Trigger re-search if query exists
                    if !app.deep_search_query.trim().is_empty() {
                        app.perform_deep_search();
                    }
                }

                ui.separator();

                // List all root collections and their children
                let root_collections: Vec<_> = app
                    .collections
                    .iter()
                    .filter(|c| c.parent_id.is_none())
                    .cloned()
                    .collect();

                for collection in root_collections {
                    render_collection_option(app, ui, &collection, 0);
                }
            });
    });

    if app.is_deep_searching {
        ui.spinner();
        ui.label("Searching database...");
    } else {
        ui.separator();
        ui.label(format!("Found {} results", app.deep_search_results.len()));

        let mut nav_action: Option<(SearchResultKind, i64)> = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
            for res in &app.deep_search_results {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    // Header
                    let icon = match res.kind {
                        SearchResultKind::Character => "👤",
                        SearchResultKind::Lorebook => "📖",
                    };
                    if ui
                        .link(
                            egui::RichText::new(format!("{} {}", icon, res.display_name))
                                .heading()
                                .strong(),
                        )
                        .clicked()
                    {
                        nav_action = Some((res.kind.clone(), res.id));
                    }

                    // Snippets
                    for (field, snippet) in &res.matches {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("{}:", field))
                                    .strong()
                                    .size(11.0),
                            );
                            ui.label(egui::RichText::new(snippet).italics());
                        });
                    }
                });
                ui.add_space(4.0);
            }
        });

        // Handle Navigation
        if let Some((kind, id)) = nav_action {
            match kind {
                SearchResultKind::Character => {
                    app.load_character(id);
                }
                SearchResultKind::Lorebook => {
                    if let Some(l) = app.lorebooks.iter().find(|x| x.id == id).cloned() {
                        app.selected_lorebook = Some(l);
                        app.mode = AppMode::Lorebooks;
                    }
                }
            }
        }
    }

    ui.add_space(10.0);
    if ui.button("Back").clicked() {
        app.mode = AppMode::Characters;
    }
}

// Helper function to render collection hierarchy in dropdown
fn render_collection_option(
    app: &mut CrapApp,
    ui: &mut egui::Ui,
    collection: &crate::models::Collection,
    depth: usize,
) {
    let indent = "  ".repeat(depth);
    let label = format!("{}{}", indent, collection.name);

    if ui
        .selectable_value(
            &mut app.deep_search_filter_collection,
            Some(collection.id),
            label,
        )
        .clicked()
    {
        if !app.deep_search_query.trim().is_empty() {
            app.perform_deep_search();
        }
    }

    // Render children
    let children: Vec<crate::models::Collection> = app
        .collections
        .iter()
        .filter(|c| c.parent_id == Some(collection.id))
        .cloned()
        .collect();

    for child in children {
        render_collection_option(app, ui, &child, depth + 1);
    }
}
