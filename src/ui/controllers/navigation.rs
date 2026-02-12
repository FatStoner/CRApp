use super::state::CrapApp;
use crate::ui::types::{AppAction, NavigationState};
use crate::ui::types::{AppMode, CentralView};
use crate::ui::PopupState;
use eframe::egui;

impl CrapApp {
    pub fn push_history(&mut self) {
        let state = NavigationState {
            mode: self.mode,
            central_view: self.central_view,
            selected_character_id: self.selected_character.as_ref().map(|c| c.id),
            selected_lorebook_id: self.selected_lorebook.as_ref().map(|l| l.id),
            selected_collection_id: self.selected_collection_id,
            selected_lorebook_entry_id: self.selected_entry.as_ref().map(|e| e.id),
            selected_lorebook_entry_name: self.selected_entry.as_ref().map(|e| e.name.clone()),
            active_char_tab: self.active_char_tab,
            active_lorebook_tab: self.active_lorebook_tab,
        };
        // Avoid pushing duplicates if nothing changed
        if let Some(last) = self.navigation_history.last() {
            if last.mode == state.mode
                && last.central_view == state.central_view
                && last.selected_character_id == state.selected_character_id
                && last.selected_lorebook_id == state.selected_lorebook_id
                && last.selected_collection_id == state.selected_collection_id
                && last.selected_lorebook_entry_id == state.selected_lorebook_entry_id
                && last.selected_lorebook_entry_name == state.selected_lorebook_entry_name
            {
                return;
            }
        }
        self.navigation_history.push(state);
    }

    pub fn go_back(&mut self) {
        if let Some(state) = self.navigation_history.pop() {
            self.mode = state.mode;
            self.central_view = state.central_view;
            self.selected_collection_id = state.selected_collection_id;
            self.active_char_tab = state.active_char_tab;
            self.active_lorebook_tab = state.active_lorebook_tab;

            // Restore Selection
            if let Some(char_id) = state.selected_character_id {
                if let Some(c) = self.characters.iter().find(|c| c.id == char_id).cloned() {
                    self.selected_character = Some(c);
                    self.load_links(char_id);
                    self.load_tags(char_id);
                }
            } else {
                self.selected_character = None;
            }

            if let Some(lore_id) = state.selected_lorebook_id {
                if let Some(book) = self.lorebooks.iter().find(|l| l.id == lore_id).cloned() {
                    self.selected_lorebook = Some(book);
                    self.load_lorebook_entries(lore_id);
                    self.load_lorebook_tags(lore_id);

                    if let Some(entry_id) = state.selected_lorebook_entry_id {
                        if let Some(b) = &self.selected_lorebook {
                            if let Some(entry) =
                                b.entries.iter().find(|e| e.id == entry_id).cloned()
                            {
                                self.selected_entry = Some(entry);
                            } else {
                                self.selected_entry = None;
                            }
                        }
                    } else {
                        self.selected_entry = None;
                    }
                }
            } else {
                self.selected_lorebook = None;
                self.selected_entry = None;
            }
        }
    }

    pub fn go_to_history(&mut self, index: usize) {
        if index < self.navigation_history.len() {
            self.navigation_history.truncate(index + 1);
            self.go_back();
        }
    }

    pub fn request_back(&mut self) {
        if self.has_unsaved_changes() {
            self.popup_state = PopupState::UnsavedChanges {
                target: AppAction::GoBack,
            };
        } else {
            self.go_back();
        }
    }

    pub fn request_collection_switch(&mut self, id: Option<i64>) {
        self.push_history();
        if self.has_unsaved_changes() {
            self.popup_state = PopupState::UnsavedChanges {
                target: AppAction::SwitchCollection(id),
            };
        } else {
            self.viewing_all_characters = false;
            self.viewing_favorites = false;
            self.selected_collection_id = id;
            self.mode = AppMode::Characters;
            self.central_view = CentralView::Browser;
            self.selected_character = None;
            self.reload_collections();
        }
    }

    pub fn request_view_all(&mut self) {
        if self.has_unsaved_changes() {
            self.popup_state = PopupState::UnsavedChanges {
                target: AppAction::SwitchToAll,
            };
        } else {
            self.viewing_all_characters = true;
            self.viewing_favorites = false;
            self.selected_collection_id = None;
            self.mode = AppMode::Characters;
            self.central_view = CentralView::Browser;
            self.selected_character = None;
            self.reload_characters();
        }
    }

    pub fn request_switch_to_templates(&mut self) {
        if self.has_unsaved_changes() {
            self.popup_state = PopupState::UnsavedChanges {
                target: AppAction::SwitchToTemplates,
            };
        } else {
            self.mode = AppMode::Templates;
            self.selected_character = None;
            self.selected_lorebook = None;
        }
    }

    pub fn request_character_switch(&mut self, id: i64) {
        if self.has_unsaved_changes() {
            self.popup_state = PopupState::UnsavedChanges {
                target: AppAction::SwitchCharacter(id),
            };
        } else {
            self.load_character(id);
        }
    }

    pub fn request_lorebook_switch(&mut self, id: i64) {
        if self.has_unsaved_changes() {
            self.popup_state = PopupState::UnsavedChanges {
                target: AppAction::SwitchLorebook(id),
            };
        } else {
            self.load_lorebook(id);
        }
    }

    pub fn request_template_switch(&mut self, id: i64) {
        if self.has_unsaved_changes() {
            self.popup_state = PopupState::UnsavedChanges {
                target: AppAction::SwitchTemplate(id),
            };
        } else {
            self.perform_template_switch(id);
        }
    }

    pub fn perform_template_switch(&mut self, id: i64) {
        self.push_history();
        if let Some(t) = self.templates.iter().find(|t| t.id == id).cloned() {
            self.selected_template = Some(t);
            self.mode = AppMode::Templates;
            self.selected_character = None;
            self.selected_lorebook = None;
        }
    }

    pub fn perform_action(&mut self, action: AppAction, ctx: &egui::Context) {
        match action {
            AppAction::SwitchCharacter(id) => self.load_character(id),
            AppAction::SwitchCollection(id) => {
                self.push_history();
                self.viewing_all_characters = false;
                self.selected_collection_id = id;
                self.mode = AppMode::Characters;
                self.central_view = CentralView::Browser;
                self.selected_character = None;
                self.reload_collections();
            }
            AppAction::SwitchLorebook(id) => self.load_lorebook(id),
            AppAction::SwitchToAll => {
                self.viewing_all_characters = true;
                self.selected_collection_id = None;
                self.mode = AppMode::Characters;
                self.central_view = CentralView::Browser;
                self.selected_character = None;
                self.reload_characters();
            }
            AppAction::Exit => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            AppAction::GoBack => {
                self.go_back();
            }
            AppAction::GoToHistory(index) => {
                self.go_to_history(index);
            }
            AppAction::CreateNewCharacter(coll_id) => {
                self.perform_create_new_character(coll_id);
            }
            AppAction::CreateNewLorebook => {
                self.perform_create_new_lorebook();
            }
            AppAction::CreateNewTemplate => {
                self.perform_create_new_template();
            }
            AppAction::SwitchTemplate(id) => {
                self.perform_template_switch(id);
            }
            AppAction::SwitchToTemplates => {
                self.mode = AppMode::Templates;
                self.selected_character = None;
                self.selected_lorebook = None;
            }
        }
    }
}
