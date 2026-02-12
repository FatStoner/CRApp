mod deletion;
mod editing;
mod import_export;
mod templates;

use crate::ui::{AppAction, CrapApp};
use eframe::egui;

#[derive(Clone)]
pub enum PopupState {
    None,
    Renaming {
        id: i64,
        name: String,
    },

    DeleteWarning {
        _id: i64,
        count: usize,
    },
    DeleteCharacterConfirmation {
        id: i64,
        name: String,
    },
    DeleteLorebookEntryConfirmation {
        id: i64,
        lorebook_id: i64,
        name: String,
    },
    DeleteLorebookConfirmation {
        id: i64,
        title: String,
    },
    DeleteTemplateConfirmation {
        id: i64,
        name: String,
    },
    UnsavedChanges {
        target: AppAction,
    },
    ImportDbWarning,
    CollectionIconConfirmation {
        id: i64,
        path: String,
        _preview_texture: Option<egui::TextureHandle>,
    },
    LorebookImport {
        source_code: String,
        parsed_data: Option<crate::ui::parsing::ParsedLorebookData>,
    },
    ExportDbSelection,
    TemplateSelector,
    TemplatePreview {
        template_data: crate::models::Template,
        target_char_id: i64,
    },
}

pub fn render_popups(ctx: &egui::Context, app: &mut CrapApp) {
    // We clone the state to avoid mutable borrow conflicts
    let state = app.popup_state.clone();

    match &state {
        PopupState::None => {}

        // Deletion popups
        PopupState::DeleteWarning { .. }
        | PopupState::DeleteCharacterConfirmation { .. }
        | PopupState::DeleteLorebookConfirmation { .. }
        | PopupState::DeleteLorebookEntryConfirmation { .. }
        | PopupState::DeleteTemplateConfirmation { .. } => {
            deletion::render_deletion_popups(ctx, app, &state);
        }

        // Editing popups
        PopupState::Renaming { .. }
        | PopupState::UnsavedChanges { .. }
        | PopupState::CollectionIconConfirmation { .. } => {
            editing::render_editing_popups(ctx, app, &state);
        }

        // Import/Export popups
        PopupState::ImportDbWarning
        | PopupState::LorebookImport { .. }
        | PopupState::ExportDbSelection => {
            import_export::render_import_export_popups(ctx, app, &state);
        }

        // Template popups
        PopupState::TemplateSelector | PopupState::TemplatePreview { .. } => {
            templates::render_template_popups(ctx, app, &state);
        }
    }
}
