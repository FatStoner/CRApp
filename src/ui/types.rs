use crate::db::Database;
use crate::models::{Character, Collection, DeepSearchResult, Lorebook, Tag, Template, ThemeMode};
use eframe::egui;
use std::collections::{HashMap, HashSet};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AppMode {
    Characters,
    Lorebooks,
    Templates,
    DeepSearch,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CharacterTab {
    MainData,
    Notes,
    Lorebooks,
    Gallery,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum LorebookTab {
    Entries,
    Characters,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CentralView {
    Editor,
    Browser,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum SortMode {
    Alphabetical,
    NewestFirst,
    RecentlyUpdated,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum BrowserViewMode {
    Grid,
    List, // Title + URLs
}

#[derive(Clone, Debug, PartialEq)]
pub enum AppAction {
    SwitchCharacter(i64),
    SwitchCollection(Option<i64>),
    SwitchLorebook(i64),
    SwitchToAll,
    Exit,
    GoBack,
    CreateNewCharacter(Option<i64>),
    CreateNewLorebook,
    CreateNewTemplate,
    SwitchTemplate(i64),
    SwitchToTemplates,
}

#[derive(Clone, Debug)]
pub struct NavigationState {
    pub mode: AppMode,
    pub central_view: CentralView,
    pub selected_character_id: Option<i64>,
    pub selected_lorebook_id: Option<i64>,
    pub selected_collection_id: Option<i64>,
    pub active_char_tab: CharacterTab,
    pub active_lorebook_tab: LorebookTab,
}

pub enum UiEvent {
    UiRepaint, // Generic repaint signal
    DeepSearchCompleted(Result<Vec<DeepSearchResult>, String>),
    CharacterDeleted(Result<i64, String>),
    CharacterMoved(Result<(i64, Option<i64>), String>),
    CharactersLoaded(Result<Vec<Character>, String>),
    LorebooksLoaded(Result<Vec<Lorebook>, String>),
    CollectionsLoaded(Result<Vec<Collection>, String>),
    LoreLinksLoaded(Result<HashSet<i64>, String>),
    CharacterSaved(Result<Character, String>),
    LorebookSaved(Result<Lorebook, String>),
    CollectionSaved(Result<i64, String>),
    CollectionDeleted(Result<i64, String>),
    LinkUpdated(Result<(), String>),
    TagsLoaded(Result<(i64, Vec<Tag>, Vec<Tag>), String>),
    LorebookTagsLoaded(Result<(i64, Vec<Tag>), String>),
    TagOperationFinished(Result<(), String>),
    LorebookTagOperationFinished(Result<(), String>),

    // Lorebook Entries
    LorebookEntriesLoaded(Result<(i64, Vec<crate::models::LorebookEntry>), String>),
    LorebookEntrySaved(Result<(), String>),
    LorebookEntryDeleted(Result<i64, String>),
    LorebookEntryAdded(Result<i64, String>), // Returns new ID
    LorebookDeleted(Result<i64, String>),
    TemplatesLoaded(Result<Vec<Template>, String>),
    TemplateSaved(Result<Template, String>),
    TemplateDeleted(Result<i64, String>),

    ImportFileLoaded(Result<String, String>),
    ThemeLoaded(Result<ThemeMode, String>),
    ScaleLoaded(Result<f32, String>),
    DbExportFinished(Result<String, String>),
    DbReloaded(Result<Database, String>),

    LoreLinksBulkLoaded(HashMap<i64, Vec<i64>>),

    TokenCountCalculated(i64, usize, usize), // (CharId, Tokens, Chars)
    LorebookImported(Lorebook),
    StatusMessage(String, egui::Color32),
}
