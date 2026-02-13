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
pub enum SettingsTab {
    General,
    Tokens,
    Updates,
    About,
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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ExportFormat {
    Png,
    V2,
    Native,
    Markdown,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExportTarget {
    Collection(i64),
    All,
    Favorites,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AppAction {
    SwitchCharacter(i64),
    SwitchCollection(Option<i64>),
    SwitchLorebook(i64),
    SwitchToAll,
    Exit,
    GoBack,
    GoToHistory(usize),
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
    pub selected_lorebook_entry_id: Option<i64>,
    pub selected_lorebook_entry_name: Option<String>,
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
    ImportFileLoaded(Result<String, String>, Option<u64>),
    ImportCharacterData(Result<crate::ui::ParsedCharacterData, String>, Option<u64>),
    ThemeLoaded(Result<ThemeMode, String>),
    ScaleLoaded(Result<f32, String>),
    DbExportFinished(Result<String, String>),
    DbReloaded(Result<Database, String>),

    LoreLinksBulkLoaded(HashMap<i64, Vec<i64>>),

    TokenCountCalculated(i64, usize, usize), // (CharId, Tokens, Chars)
    LorebookImported(Lorebook),
    StatusMessage(String, egui::Color32),
    CustomBackgroundLoaded(bool),
    WatermarkLoaded(bool),
    BackgroundLoaded(bool),
    SpellCheckSettingLoaded(bool),
    BackgroundScaleLoaded(f32),
    GalleryImageAdded(String),
    StatisticsCalculated(StatisticsData),
    UpdateAvailable(String, String), // (version, tag)
    UpdateCheckFinished(Result<Option<(String, String)>, String>, bool), // (Result<(version, tag)>, is_manual_check)
    UpdateStarted,
    UpdateFailed(String),
    CheckUpdatesAtStartLoaded(bool),
}

#[derive(Clone, Debug, Default)]
pub struct StatisticsData {
    pub character_count: usize,
    pub total_tokens_avg: f32,
    pub total_chars_avg: f32,

    // Breakdown averages
    pub name_tokens_avg: f32,
    pub name_chars_avg: f32,

    pub title_tokens_avg: f32,
    pub title_chars_avg: f32,

    pub personality_tokens_avg: f32,
    pub personality_chars_avg: f32,

    pub scenario_tokens_avg: f32,
    pub scenario_chars_avg: f32,

    pub first_message_tokens_avg: f32,
    pub first_message_chars_avg: f32,

    pub example_dialogue_tokens_avg: f32,
    pub example_dialogue_chars_avg: f32,
}

#[derive(Clone, Debug)]
pub struct StatisticsState {
    pub source_name: String,
    pub is_calculating: bool,
    pub data: Option<StatisticsData>,
}
