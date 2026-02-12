use super::state::CrapApp;
use crate::models::{Character, Lorebook, Template};
use eframe::egui;
use std::time::{Duration, Instant};

impl CrapApp {
    pub fn set_status(&mut self, msg: String, color: egui::Color32) {
        self.set_status_with_duration(msg, color, Duration::from_secs(3));
    }

    pub fn set_status_with_duration(
        &mut self,
        msg: String,
        color: egui::Color32,
        duration: Duration,
    ) {
        self.status_message = Some((msg, color));
        self.status_clear_time = Some(Instant::now() + duration);
    }

    pub fn has_unsaved_changes(&self) -> bool {
        if let Some(selected) = &self.selected_character {
            if selected.id == 0 {
                // For new character, check if it has content different from default
                !selected.content_eq(&Character::default())
            } else {
                // For existing, compare with cached db version
                if let Some(original) = self.characters.iter().find(|c| c.id == selected.id) {
                    !selected.content_eq(original)
                } else {
                    false
                }
            }
        } else if let Some(selected_book) = &self.selected_lorebook {
            if selected_book.id == 0 {
                !selected_book.content_eq(&Lorebook::default())
            } else {
                if let Some(original) = self.lorebooks.iter().find(|l| l.id == selected_book.id) {
                    !selected_book.content_eq(original)
                } else {
                    false
                }
            }
        } else if let Some(selected_template) = &self.selected_template {
            if selected_template.id == 0 {
                !selected_template.content_eq(&Template::default())
            } else {
                if let Some(original) = self.templates.iter().find(|t| t.id == selected_template.id)
                {
                    !selected_template.content_eq(original)
                } else {
                    false
                }
            }
        } else {
            false
        }
    }

    pub fn describe_state(&self, state: &crate::ui::NavigationState) -> String {
        match state.central_view {
            crate::ui::CentralView::Editor => {
                match state.mode {
                    crate::ui::AppMode::Characters => {
                        if let Some(id) = state.selected_character_id {
                            if let Some(c) = self.characters.iter().find(|c| c.id == id) {
                                return format!("Character: {}", c.name);
                            }
                            return "Character Editor".to_string();
                        }
                    }
                    crate::ui::AppMode::Lorebooks => {
                        if let Some(id) = state.selected_lorebook_id {
                            if let Some(l) = self.lorebooks.iter().find(|l| l.id == id) {
                                let mut base = format!("Lorebook: {}", l.title);
                                if let Some(entry_name) = &state.selected_lorebook_entry_name {
                                    base.push_str(&format!(" ({})", entry_name));
                                } else if let Some(entry_id) = state.selected_lorebook_entry_id {
                                    if let Some(entry) = l.entries.iter().find(|e| e.id == entry_id)
                                    {
                                        base.push_str(&format!(" ({})", entry.name));
                                    }
                                }
                                return base;
                            }
                            return "Lorebook Editor".to_string();
                        }
                    }
                    _ => {}
                }
                "Editor".to_string()
            }
            crate::ui::CentralView::Browser => {
                if let Some(id) = state.selected_collection_id {
                    let path = self.get_collection_path(id);
                    if path.is_empty() {
                        "Browser (Root)".to_string()
                    } else {
                        format!("Folder: {}", path)
                    }
                } else {
                    "Browser".to_string()
                }
            }
        }
    }

    pub fn get_collection_path(&self, mut col_id: i64) -> String {
        let mut path = Vec::new();
        for _ in 0..10 {
            if let Some(col) = self.collections.iter().find(|c| c.id == col_id) {
                path.push(col.name.clone());
                if let Some(pid) = col.parent_id {
                    col_id = pid;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        path.reverse();
        path.join(" / ")
    }
}
