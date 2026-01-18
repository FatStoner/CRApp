use crate::ui::{AppMode, CrapApp, SearchResultKind};
use eframe::egui;

#[derive(Clone, Debug, PartialEq)]
pub struct CharacterSearchFieldFilters {
    pub name: bool,
    pub char_title: bool,
    pub personality: bool,
    pub scenario: bool,
    pub first_message: bool,
    pub example_dialogue: bool,
    pub author_notes: bool,
    pub urls: bool,
    pub tags: bool,
}

impl Default for CharacterSearchFieldFilters {
    fn default() -> Self {
        Self {
            name: true,
            char_title: true,
            personality: true,
            scenario: true,
            first_message: true,
            example_dialogue: true,
            author_notes: true,
            urls: true,
            tags: true,
        }
    }
}

impl CharacterSearchFieldFilters {
    pub fn all_enabled() -> Self {
        Self::default()
    }

    pub fn all_disabled() -> Self {
        Self {
            name: false,
            char_title: false,
            personality: false,
            scenario: false,
            first_message: false,
            example_dialogue: false,
            author_notes: false,
            urls: false,
            tags: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LorebookSearchFieldFilters {
    pub title: bool,
    pub description: bool,
    pub tags: bool,
    pub entry_name: bool,
    pub entry_keywords: bool,
    pub entry_content: bool,
}

impl Default for LorebookSearchFieldFilters {
    fn default() -> Self {
        Self {
            title: true,
            description: true,
            tags: true,
            entry_name: true,
            entry_keywords: true,
            entry_content: true,
        }
    }
}

impl LorebookSearchFieldFilters {
    pub fn all_enabled() -> Self {
        Self::default()
    }

    pub fn all_disabled() -> Self {
        Self {
            title: false,
            description: false,
            tags: false,
            entry_name: false,
            entry_keywords: false,
            entry_content: false,
        }
    }
}

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

    // Field filters
    ui.separator();
    ui.horizontal(|ui| {
        ui.label("Search in:");

        if ui.button("All").clicked() {
            app.deep_search_char_field_filters =
                crate::ui::CharacterSearchFieldFilters::all_enabled();
            app.deep_search_lore_field_filters =
                crate::ui::LorebookSearchFieldFilters::all_enabled();
            if !app.deep_search_query.trim().is_empty() {
                app.perform_deep_search();
            }
        }

        if ui.button("None").clicked() {
            app.deep_search_char_field_filters =
                crate::ui::CharacterSearchFieldFilters::all_disabled();
            app.deep_search_lore_field_filters =
                crate::ui::LorebookSearchFieldFilters::all_disabled();
            if !app.deep_search_query.trim().is_empty() {
                app.perform_deep_search();
            }
        }
    });

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Character Fields:").strong());
        ui.add_space(8.0);
        if ui.button("All").clicked() {
            app.deep_search_char_field_filters =
                crate::ui::CharacterSearchFieldFilters::all_enabled();
            if !app.deep_search_query.trim().is_empty() {
                app.perform_deep_search();
            }
        }
        if ui.button("None").clicked() {
            app.deep_search_char_field_filters =
                crate::ui::CharacterSearchFieldFilters::all_disabled();
            if !app.deep_search_query.trim().is_empty() {
                app.perform_deep_search();
            }
        }
    });
    ui.horizontal_wrapped(|ui| {
        let mut changed = false;

        changed |= ui
            .checkbox(&mut app.deep_search_char_field_filters.name, "Name")
            .changed();
        changed |= ui
            .checkbox(&mut app.deep_search_char_field_filters.char_title, "Title")
            .changed();
        changed |= ui
            .checkbox(
                &mut app.deep_search_char_field_filters.personality,
                "Personality",
            )
            .changed();
        changed |= ui
            .checkbox(&mut app.deep_search_char_field_filters.scenario, "Scenario")
            .changed();
        changed |= ui
            .checkbox(
                &mut app.deep_search_char_field_filters.first_message,
                "First Message",
            )
            .changed();
        changed |= ui
            .checkbox(
                &mut app.deep_search_char_field_filters.example_dialogue,
                "Example Dialogue",
            )
            .changed();
        changed |= ui
            .checkbox(
                &mut app.deep_search_char_field_filters.author_notes,
                "Notes",
            )
            .changed();
        changed |= ui
            .checkbox(&mut app.deep_search_char_field_filters.urls, "URLs")
            .changed();
        changed |= ui
            .checkbox(&mut app.deep_search_char_field_filters.tags, "Tags")
            .changed();

        if changed && !app.deep_search_query.trim().is_empty() {
            app.perform_deep_search();
        }
    });

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Lorebook Fields:").strong());
        ui.add_space(8.0);
        if ui.button("All").clicked() {
            app.deep_search_lore_field_filters =
                crate::ui::LorebookSearchFieldFilters::all_enabled();
            if !app.deep_search_query.trim().is_empty() {
                app.perform_deep_search();
            }
        }
        if ui.button("None").clicked() {
            app.deep_search_lore_field_filters =
                crate::ui::LorebookSearchFieldFilters::all_disabled();
            if !app.deep_search_query.trim().is_empty() {
                app.perform_deep_search();
            }
        }
    });
    ui.horizontal_wrapped(|ui| {
        let mut changed = false;

        changed |= ui
            .checkbox(&mut app.deep_search_lore_field_filters.title, "Title")
            .changed();
        changed |= ui
            .checkbox(
                &mut app.deep_search_lore_field_filters.description,
                "Description",
            )
            .changed();
        changed |= ui
            .checkbox(&mut app.deep_search_lore_field_filters.tags, "Tags")
            .changed();
        changed |= ui
            .checkbox(
                &mut app.deep_search_lore_field_filters.entry_name,
                "Entry Name",
            )
            .changed();
        changed |= ui
            .checkbox(
                &mut app.deep_search_lore_field_filters.entry_keywords,
                "Entry Keywords",
            )
            .changed();
        changed |= ui
            .checkbox(
                &mut app.deep_search_lore_field_filters.entry_content,
                "Entry Content",
            )
            .changed();

        if changed && !app.deep_search_query.trim().is_empty() {
            app.perform_deep_search();
        }
    });
    ui.separator();

    if app.is_deep_searching {
        ui.spinner();
        ui.label("Searching database...");
    } else {
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(format!("Found {} results", app.deep_search_results.len()));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let sort_label = match app.deep_search_sort {
                    None => "Sort: Default",
                    Some(crate::ui::SortDirection::Ascending) => "Sort: A-Z",
                    Some(crate::ui::SortDirection::Descending) => "Sort: Z-A",
                };

                if ui.button(sort_label).clicked() {
                    app.deep_search_sort = match app.deep_search_sort {
                        None => Some(crate::ui::SortDirection::Ascending),
                        Some(crate::ui::SortDirection::Ascending) => {
                            Some(crate::ui::SortDirection::Descending)
                        }
                        Some(crate::ui::SortDirection::Descending) => None,
                    };
                    app.sort_deep_search_results();
                }
            });
        });

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
                    // Transfer search query to editor search for immediate highlighting
                    app.editor_search_query = app.deep_search_query.clone();

                    // Smart tab selection based on match location
                    if !app.editor_search_query.is_empty() {
                        if let Some(character) = app.characters.iter().find(|c| c.id == id) {
                            let query_lower = app.editor_search_query.to_lowercase();

                            // Check if query matches in main fields
                            let in_main_fields = character
                                .first_message
                                .to_lowercase()
                                .contains(&query_lower)
                                || character.personality.to_lowercase().contains(&query_lower)
                                || character.scenario.to_lowercase().contains(&query_lower)
                                || character
                                    .example_dialogue
                                    .to_lowercase()
                                    .contains(&query_lower);

                            // Check if query matches in notes fields
                            let in_notes_fields =
                                character.author_notes.to_lowercase().contains(&query_lower)
                                    || character
                                        .urls
                                        .iter()
                                        .any(|u| u.url.to_lowercase().contains(&query_lower));

                            // If match is ONLY in notes, open Notes tab
                            if !in_main_fields && in_notes_fields {
                                app.active_char_tab = crate::ui::CharacterTab::Notes;
                            } else {
                                // Default to MainData tab
                                app.active_char_tab = crate::ui::CharacterTab::MainData;
                            }
                        }
                    }

                    app.load_character(id);
                }
                SearchResultKind::Lorebook => {
                    // Populate editor search query for lorebooks as well
                    app.editor_search_query = app.deep_search_query.clone();

                    if let Some(l) = app.lorebooks.iter().find(|x| x.id == id).cloned() {
                        // Smart tab selection for lorebooks
                        if !app.editor_search_query.is_empty() {
                            if let Some(res) = app
                                .deep_search_results
                                .iter()
                                .find(|r| r.id == id && r.kind == SearchResultKind::Lorebook)
                            {
                                let has_entry_match = res
                                    .matches
                                    .iter()
                                    .any(|(field, _)| field.starts_with("Entry"));
                                if has_entry_match {
                                    app.active_lorebook_tab = crate::ui::LorebookTab::Entries;
                                } else {
                                    // If no entry match, maybe it's in metadata. Use Entries as default if entries exist?
                                    // Actually, if it's metadata, staying in metadata view (managed by ScrollArea in render_lorebook_editor) is fine.
                                    // But let's check if we want to default to something. The character logic defaults to MainData.
                                    // For lorebooks, metadata is always visible at the top.
                                }
                            }
                        }

                        app.selected_lorebook = Some(l.clone());
                        app.load_lorebook_entries(l.id);
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
