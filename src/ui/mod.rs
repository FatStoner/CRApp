use crate::db::Database;
use eframe::egui;

use crate::models::{
    count_tokens, Character, Collection, DeepSearchResult, Lorebook, SearchResultKind, Tag,
    ThemeMode,
};

use tokio::sync::mpsc;

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

pub mod browser;
pub mod central_panel;
pub mod editor;
pub mod global_search;
pub mod popups;
pub mod side_panel;
pub mod text_highlight;
pub mod widgets;

pub use global_search::{CharacterSearchFieldFilters, LorebookSearchFieldFilters};
pub use popups::PopupState;

pub mod parsing;

// Re-export specific items if needed
pub use parsing::ParsedCharacterData;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AppMode {
    Characters,
    Lorebooks,
    DeepSearch,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CharacterTab {
    MainData,
    Notes,
    Lorebooks,
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

#[derive(Clone, Debug, PartialEq)]
pub enum AppAction {
    SwitchCharacter(i64),
    SwitchCollection(Option<i64>),
    SwitchToAll,
    Exit,
    GoBack,
}

// PopupState moved to popups.rs

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

    ImportFileLoaded(Result<String, String>),
    ThemeLoaded(Result<ThemeMode, String>),
    ScaleLoaded(Result<f32, String>),
    DbExportFinished(Result<String, String>),
    DbReloaded(Result<Database, String>),
    LoreLinksBulkLoaded(HashMap<i64, Vec<i64>>),
    CollectionIconUpdated(Result<i64, String>),
}

pub struct CrapApp {
    db: Database,
    tx: mpsc::Sender<UiEvent>,
    rx: mpsc::Receiver<UiEvent>,
    pub ctx: egui::Context,

    // Data (Cached)
    pub characters: Vec<Character>,
    pub lorebooks: Vec<Lorebook>,
    pub collections: Vec<Collection>,
    pub lore_links: HashSet<i64>,
    pub char_lore_map: HashMap<i64, Vec<i64>>,

    // State
    pub mode: AppMode,
    pub selected_character: Option<Character>,
    pub selected_lorebook: Option<Lorebook>,
    pub selected_entry: Option<crate::models::LorebookEntry>,
    pub active_char_tab: CharacterTab,
    pub active_lorebook_tab: LorebookTab,
    pub central_view: CentralView,
    pub theme: ThemeMode,
    pub ui_scale: f32,
    pub sort_mode: SortMode,
    pub sort_direction: SortDirection,
    pub browser_sort_mode: SortMode,
    pub browser_sort_direction: SortDirection,
    pub browser_show_urls: bool,
    pub selected_collection_id: Option<i64>,

    pub popup_state: PopupState,
    pub is_saving: bool,
    pub status_message: Option<(String, egui::Color32)>,
    pub status_clear_time: Option<Instant>,
    pub loading_error: Option<String>,

    // Search
    pub search_query: String,                       // Side panel filter
    pub deep_search_query: String,                  // Global
    pub deep_search_filter_collection: Option<i64>, // None = All Folders
    pub deep_search_char_field_filters: CharacterSearchFieldFilters, // Character field selection
    pub deep_search_lore_field_filters: LorebookSearchFieldFilters, // Lorebook field selection
    pub deep_search_results: Vec<DeepSearchResult>,
    pub is_deep_searching: bool,
    pub editor_search_query: String, // In-editor search

    // Tag editor
    pub app_tag_input: String,
    pub ext_tag_input: String,

    // Import Modal State
    pub show_import_modal: bool,
    pub import_text: String,
    pub parsed_data: Option<ParsedCharacterData>,

    pub viewing_all_characters: bool,
    pub viewing_favorites: bool,
    pub pending_action: Option<AppAction>,

    // Preferences
    pub count_title_in_total: bool,

    // Navigation History
    pub navigation_history: Vec<NavigationState>,
}

impl CrapApp {
    pub fn new(cc: &eframe::CreationContext<'_>, db: Database) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let (tx, rx) = mpsc::channel(20);

        let app = Self {
            db,
            tx,
            rx,
            ctx: cc.egui_ctx.clone(),
            characters: Vec::new(),
            lorebooks: Vec::new(),
            collections: Vec::new(),
            lore_links: HashSet::new(),
            char_lore_map: HashMap::new(),
            mode: AppMode::Characters,
            selected_character: None,
            selected_lorebook: None,
            selected_entry: None,

            active_char_tab: CharacterTab::MainData,
            active_lorebook_tab: LorebookTab::Entries,
            central_view: CentralView::Browser,
            sort_mode: SortMode::Alphabetical,
            sort_direction: SortDirection::Ascending,
            browser_sort_mode: SortMode::Alphabetical,
            browser_sort_direction: SortDirection::Ascending,
            browser_show_urls: false,
            selected_collection_id: None,
            popup_state: PopupState::None,
            is_saving: false,
            status_message: None,
            status_clear_time: None,
            loading_error: None,
            search_query: String::new(),
            deep_search_query: String::new(),
            deep_search_filter_collection: None,
            deep_search_char_field_filters: CharacterSearchFieldFilters::default(),
            deep_search_lore_field_filters: LorebookSearchFieldFilters::default(),
            deep_search_results: Vec::new(),
            is_deep_searching: false,
            editor_search_query: String::new(),
            app_tag_input: String::new(),
            ext_tag_input: String::new(),

            show_import_modal: false,
            import_text: String::new(),
            parsed_data: None,

            viewing_all_characters: false,
            viewing_favorites: false,
            pending_action: None,
            theme: ThemeMode::System,
            ui_scale: 1.0,

            count_title_in_total: false,

            navigation_history: Vec::new(),
        };

        // Initial Scale Load
        let tx = app.tx.clone();
        let db = app.db.clone();
        let ctx = app.ctx.clone();
        tokio::spawn(async move {
            match db.get_setting("ui_scale").await {
                Ok(Some(val)) => {
                    if let Ok(scale) = val.parse::<f32>() {
                        let _ = tx.send(UiEvent::ScaleLoaded(Ok(scale))).await;
                        ctx.request_repaint();
                    }
                }
                Ok(None) => {} // Default 1.0
                Err(e) => eprintln!("Failed to load scale: {}", e),
            }
        });

        // Initial Theme Load
        let tx = app.tx.clone();
        let db = app.db.clone();
        let ctx = app.ctx.clone();
        tokio::spawn(async move {
            match db.get_setting("theme").await {
                Ok(Some(val)) => {
                    if let Ok(mode) = val.parse::<ThemeMode>() {
                        let _ = tx.send(UiEvent::ThemeLoaded(Ok(mode))).await;
                        ctx.request_repaint();
                    }
                }
                Ok(None) => {}
                Err(e) => eprintln!("Failed to load theme: {}", e),
            }
        });

        app.refresh_all();
        app
    }

    pub fn refresh_all(&self) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            // Load characters
            match db.get_all_characters().await {
                Ok(mut chars) => {
                    // Load Tags (Bulk)
                    let app_tags_res = db.get_all_tags_flat(false).await;
                    let ext_tags_res = db.get_all_tags_flat(true).await;

                    // Load URLs (Bulk)
                    let urls_res = db.get_all_character_urls_flat().await;

                    if let (Ok(app_flat), Ok(ext_flat), Ok(urls_flat)) =
                        (app_tags_res, ext_tags_res, urls_res)
                    {
                        let mut app_map: HashMap<i64, Vec<Tag>> = HashMap::new();
                        for (cid, tag) in app_flat {
                            app_map.entry(cid).or_default().push(tag);
                        }

                        let mut ext_map: HashMap<i64, Vec<Tag>> = HashMap::new();
                        for (cid, tag) in ext_flat {
                            ext_map.entry(cid).or_default().push(tag);
                        }

                        let mut url_map: HashMap<i64, Vec<crate::models::CharacterUrl>> =
                            HashMap::new();
                        for url in urls_flat {
                            url_map.entry(url.character_id).or_default().push(url);
                        }

                        // Merge into characters
                        for c in &mut chars {
                            if let Some(tags) = app_map.remove(&c.id) {
                                c.app_tags = tags;
                            }
                            if let Some(tags) = ext_map.remove(&c.id) {
                                c.external_tags = tags;
                            }
                            if let Some(urls) = url_map.remove(&c.id) {
                                c.urls = urls;
                            }
                        }
                    } else {
                        eprintln!("Failed to load specific tags/urls bulk");
                    }

                    let _ = tx.send(UiEvent::CharactersLoaded(Ok(chars))).await;
                    ctx.request_repaint();
                }
                Err(e) => {
                    let _ = tx.send(UiEvent::CharactersLoaded(Err(e.to_string()))).await;
                    ctx.request_repaint();
                }
            }

            // Load Lore Links (Bulk) - Critical for Sidebar Search
            if let Ok(links_flat) = db.get_all_lore_links_flat().await {
                let mut cl_map: HashMap<i64, Vec<i64>> = HashMap::new();
                for (cid, lid) in links_flat {
                    cl_map.entry(cid).or_default().push(lid);
                }
                let _ = tx.send(UiEvent::LoreLinksBulkLoaded(cl_map)).await;
            }

            // Load collections
            let collections_res = db.get_all_collections().await.map_err(|e| e.to_string());
            let _ = tx.send(UiEvent::CollectionsLoaded(collections_res)).await;
            ctx.request_repaint();

            // Load Lorebooks
            match db.get_all_lorebooks().await {
                Ok(mut books) => {
                    let tags_res = db.get_all_lorebook_tags_flat().await;
                    if let Ok(tags_flat) = tags_res {
                        let mut tag_map: HashMap<i64, Vec<Tag>> = HashMap::new();
                        for (lid, tag) in tags_flat {
                            tag_map.entry(lid).or_default().push(tag);
                        }
                        for b in &mut books {
                            if let Some(tags) = tag_map.remove(&b.id) {
                                b.tags = tags;
                            }
                        }
                    }
                    let _ = tx.send(UiEvent::LorebooksLoaded(Ok(books))).await;
                    ctx.request_repaint();
                }
                Err(e) => {
                    let _ = tx.send(UiEvent::LorebooksLoaded(Err(e.to_string()))).await;
                    ctx.request_repaint();
                }
            }
        });
    }

    pub fn reload_characters(&self) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            match db.get_all_characters().await {
                Ok(mut chars) => {
                    // Load Tags (Bulk) - Same logic as refresh_all to ensure tags persist
                    let app_tags_res = db.get_all_tags_flat(false).await;
                    let ext_tags_res = db.get_all_tags_flat(true).await;
                    let urls_res = db.get_all_character_urls_flat().await;

                    if let (Ok(app_flat), Ok(ext_flat), Ok(urls_flat)) =
                        (app_tags_res, ext_tags_res, urls_res)
                    {
                        let mut app_map: HashMap<i64, Vec<Tag>> = HashMap::new();
                        for (cid, tag) in app_flat {
                            app_map.entry(cid).or_default().push(tag);
                        }

                        let mut ext_map: HashMap<i64, Vec<Tag>> = HashMap::new();
                        for (cid, tag) in ext_flat {
                            ext_map.entry(cid).or_default().push(tag);
                        }

                        let mut url_map: HashMap<i64, Vec<crate::models::CharacterUrl>> =
                            HashMap::new();
                        for url in urls_flat {
                            url_map.entry(url.character_id).or_default().push(url);
                        }

                        // Merge into characters
                        for c in &mut chars {
                            if let Some(tags) = app_map.remove(&c.id) {
                                c.app_tags = tags;
                            }
                            if let Some(tags) = ext_map.remove(&c.id) {
                                c.external_tags = tags;
                            }
                            if let Some(urls) = url_map.remove(&c.id) {
                                c.urls = urls;
                            }
                        }
                    }

                    let _ = tx.send(UiEvent::CharactersLoaded(Ok(chars))).await;
                    // Load Lore Links (Bulk)
                    if let Ok(links_flat) = db.get_all_lore_links_flat().await {
                        let mut cl_map: HashMap<i64, Vec<i64>> = HashMap::new();
                        for (cid, lid) in links_flat {
                            cl_map.entry(cid).or_default().push(lid);
                        }
                        let _ = tx.send(UiEvent::LoreLinksBulkLoaded(cl_map)).await;
                    }
                    ctx.request_repaint();
                }
                Err(e) => {
                    let _ = tx.send(UiEvent::CharactersLoaded(Err(e.to_string()))).await;
                    ctx.request_repaint();
                }
            }
        });
    }

    pub fn reload_lorebooks(&self) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            match db.get_all_lorebooks().await {
                Ok(mut books) => {
                    let tags_res = db.get_all_lorebook_tags_flat().await;
                    if let Ok(tags_flat) = tags_res {
                        let mut tag_map: HashMap<i64, Vec<Tag>> = HashMap::new();
                        for (lid, tag) in tags_flat {
                            tag_map.entry(lid).or_default().push(tag);
                        }
                        for b in &mut books {
                            if let Some(tags) = tag_map.remove(&b.id) {
                                b.tags = tags;
                            }
                        }
                    }
                    let _ = tx.send(UiEvent::LorebooksLoaded(Ok(books))).await;
                    ctx.request_repaint();
                }
                Err(e) => {
                    let _ = tx.send(UiEvent::LorebooksLoaded(Err(e.to_string()))).await;
                    ctx.request_repaint();
                }
            }
        });
    }

    pub fn load_links(&self, char_id: i64) {
        if char_id == 0 {
            return;
        }
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let result = db.get_lore_links(char_id).await.map_err(|e| e.to_string());
            let _ = tx.send(UiEvent::LoreLinksLoaded(result)).await;
            ctx.request_repaint();
        });
    }

    // Loads tags for a single character (used after selection or tag operations)
    pub fn load_tags(&self, char_id: i64) {
        if char_id == 0 {
            return;
        }
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let app_tags = db.get_tags_for_character(char_id, false).await;
            let ext_tags = db.get_tags_for_character(char_id, true).await;

            match (app_tags, ext_tags) {
                (Ok(app), Ok(ext)) => {
                    let _ = tx.send(UiEvent::TagsLoaded(Ok((char_id, app, ext)))).await;
                    ctx.request_repaint();
                }
                (Err(e), _) | (_, Err(e)) => {
                    let _ = tx.send(UiEvent::TagsLoaded(Err(e.to_string()))).await;
                    ctx.request_repaint();
                }
            }
        });
    }

    // Now just a simplified helper that spawns a load
    pub fn load_character(&mut self, id: i64) {
        self.push_history();
        // Find in logic, or reload if needed. Currently we just select from list.
        if let Some(c) = self.characters.iter().find(|c| c.id == id).cloned() {
            self.selected_character = Some(c);
            self.load_links(id);
            self.load_tags(id);
            self.mode = AppMode::Characters;
            self.central_view = CentralView::Editor;
        }
    }

    pub fn load_lorebook(&mut self, id: i64) {
        self.push_history();
        if let Some(book) = self.lorebooks.iter().find(|l| l.id == id).cloned() {
            self.selected_lorebook = Some(book);
            self.load_lorebook_entries(id);
            self.load_lorebook_tags(id);
            self.mode = AppMode::Lorebooks;
            self.central_view = CentralView::Editor;
        }
    }

    pub fn reload_collections(&self) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let result = db.get_all_collections().await.map_err(|e| e.to_string());
            let _ = tx.send(UiEvent::CollectionsLoaded(result)).await;
            ctx.request_repaint();
        });
    }

    pub fn delete_collection(&self, id: i64) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let _ctx = self.ctx.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let res = db.delete_collection(id).await;
            let _ = tx
                .send(UiEvent::CollectionDeleted(
                    res.map(|_| id).map_err(|e| e.to_string()),
                ))
                .await;
            ctx.request_repaint();
        });
    }

    pub fn save_character(&mut self, mut character: Character) {
        self.is_saving = true;
        self.status_message = None;

        // Check for avatar change to cleanup old file
        let mut old_avatar_to_delete = None;
        if character.id != 0 {
            if let Some(old) = self.characters.iter().find(|c| c.id == character.id) {
                if old.avatar_path != character.avatar_path {
                    old_avatar_to_delete = old.avatar_path.clone();
                }
            }
        }

        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let is_new = character.id == 0;
            if let Err(e) = db.upsert_character(&mut character).await {
                let _ = tx.send(UiEvent::CharacterSaved(Err(e.to_string()))).await;
                ctx.request_repaint();
            } else {
                if is_new {
                    for tag in &character.external_tags {
                        let _ = db.add_tag_to_character(character.id, &tag.name, true).await;
                    }
                    for tag in &character.app_tags {
                        let _ = db
                            .add_tag_to_character(character.id, &tag.name, false)
                            .await;
                    }
                } else {
                    // Cleanup old avatar if changed
                    if let Some(path) = old_avatar_to_delete {
                        cleanup_avatar(&path);
                    }
                }

                let _ = tx.send(UiEvent::CharacterSaved(Ok(character))).await;
                ctx.request_repaint();
                let mut chars = db.get_all_characters().await.map_err(|e| e.to_string());
                if let Ok(ref mut characters) = chars {
                    let app_tags_res = db.get_all_tags_flat(false).await;
                    let ext_tags_res = db.get_all_tags_flat(true).await;
                    let urls_res = db.get_all_character_urls_flat().await;

                    if let (Ok(app_flat), Ok(ext_flat), Ok(urls_flat)) =
                        (app_tags_res, ext_tags_res, urls_res)
                    {
                        let mut app_map: HashMap<i64, Vec<Tag>> = HashMap::new();
                        for (cid, tag) in app_flat {
                            app_map.entry(cid).or_default().push(tag);
                        }

                        let mut ext_map: HashMap<i64, Vec<Tag>> = HashMap::new();
                        for (cid, tag) in ext_flat {
                            ext_map.entry(cid).or_default().push(tag);
                        }

                        let mut url_map: HashMap<i64, Vec<crate::models::CharacterUrl>> =
                            HashMap::new();
                        for url in urls_flat {
                            url_map.entry(url.character_id).or_default().push(url);
                        }

                        for c in characters {
                            if let Some(tags) = app_map.remove(&c.id) {
                                c.app_tags = tags;
                            }
                            if let Some(tags) = ext_map.remove(&c.id) {
                                c.external_tags = tags;
                            }
                            if let Some(urls) = url_map.remove(&c.id) {
                                c.urls = urls;
                            }
                        }
                    }
                }

                let _ = tx.send(UiEvent::CharactersLoaded(chars)).await;
                ctx.request_repaint();
            }
        });
    }

    pub fn update_collection_icon(&self, id: i64, path: Option<String>) {
        if let Some(col) = self.collections.iter().find(|c| c.id == id).cloned() {
            let mut new_col = col.clone();
            new_col.image_path = path;

            let tx = self.tx.clone();
            let db = self.db.clone();
            tokio::spawn(async move {
                let _ = db.upsert_collection(&new_col).await;
                // We reuse CollectionSaved event to trigger reload
                let _ = tx.send(UiEvent::CollectionSaved(Ok(id))).await;
            });
        }
    }

    pub fn create_new_lorebook(&mut self) {
        self.push_history();
        let new_book = Lorebook::default();
        // Optimistic update so UI shows it immediately
        self.selected_lorebook = Some(new_book.clone());
        self.save_lorebook(new_book);
        self.mode = AppMode::Lorebooks;
        self.selected_character = None;
    }

    pub fn save_lorebook(&mut self, mut lorebook: Lorebook) {
        self.is_saving = true;
        self.status_message = None;
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            if let Err(e) = db.upsert_lorebook(&mut lorebook).await {
                let _ = tx.send(UiEvent::LorebookSaved(Err(e.to_string()))).await;
                ctx.request_repaint();
            } else {
                let _ = tx.send(UiEvent::LorebookSaved(Ok(lorebook))).await;
                ctx.request_repaint();

                // Correctly reload lorebooks WITH tags
                match db.get_all_lorebooks().await {
                    Ok(mut books) => {
                        let tags_res = db.get_all_lorebook_tags_flat().await;
                        if let Ok(tags_flat) = tags_res {
                            let mut tag_map: HashMap<i64, Vec<Tag>> = HashMap::new();
                            for (lid, tag) in tags_flat {
                                tag_map.entry(lid).or_default().push(tag);
                            }
                            for b in &mut books {
                                if let Some(tags) = tag_map.remove(&b.id) {
                                    b.tags = tags;
                                }
                            }
                        }
                        let _ = tx.send(UiEvent::LorebooksLoaded(Ok(books))).await;
                    }
                    Err(e) => {
                        let _ = tx.send(UiEvent::LorebooksLoaded(Err(e.to_string()))).await;
                    }
                }
                ctx.request_repaint();
            }
        });
    }

    pub fn delete_lorebook(&self, id: i64) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let res = db.delete_lorebook(id).await;
            let _ = tx
                .send(UiEvent::LorebookDeleted(
                    res.map(|_| id).map_err(|e| e.to_string()),
                ))
                .await;
            ctx.request_repaint();
        });
    }

    pub fn delete_character(&self, id: i64) {
        // Capture avatar path for cleanup
        let avatar_to_delete = self
            .characters
            .iter()
            .find(|c| c.id == id)
            .and_then(|c| c.avatar_path.clone());

        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let res = db.delete_character(id).await;

            if res.is_ok() {
                if let Some(path) = avatar_to_delete {
                    cleanup_avatar(&path);
                }
            }

            let _ = tx
                .send(UiEvent::CharacterDeleted(
                    res.map(|_| id).map_err(|e| e.to_string()),
                ))
                .await;
            ctx.request_repaint();
        });
    }

    pub fn move_character(&self, char_id: i64, target_coll_id: Option<i64>) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let res = db.move_character(char_id, target_coll_id).await;
            let _ = tx
                .send(UiEvent::CharacterMoved(
                    res.map(|_| (char_id, target_coll_id))
                        .map_err(|e| e.to_string()),
                ))
                .await;
            ctx.request_repaint();
        });
    }

    pub fn save_collection(&mut self, id: i64, name: String, parent_id: Option<i64>) {
        self.is_saving = true;
        let mut image_path = None;
        if id != 0 {
            if let Some(c) = self.collections.iter().find(|c| c.id == id) {
                image_path = c.image_path.clone();
            }
        }

        let tx = self.tx.clone();
        let db = self.db.clone();
        let col = crate::models::Collection {
            id,
            name,
            parent_id,
            display_order: 0,
            image_path,
        };
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let result = db.upsert_collection(&col).await.map_err(|e| e.to_string());
            let _ = tx.send(UiEvent::CollectionSaved(result)).await;
            ctx.request_repaint();
        });
    }

    pub fn reorder_collection(&self, id: i64, move_up: bool) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            if let Err(e) = db.reorder_collection(id, move_up).await {
                let _ = tx.send(UiEvent::CollectionSaved(Err(e.to_string()))).await;
            } else {
                let _ = tx
                    .send(UiEvent::CollectionSaved(Ok(id))) // Reuse Saved event to trigger reload
                    .await;
            }
            ctx.request_repaint();
        });
    }

    pub fn push_history(&mut self) {
        let state = NavigationState {
            mode: self.mode,
            central_view: self.central_view,
            selected_character_id: self.selected_character.as_ref().map(|c| c.id),
            selected_lorebook_id: self.selected_lorebook.as_ref().map(|l| l.id),
            selected_collection_id: self.selected_collection_id,
            active_char_tab: self.active_char_tab,
            active_lorebook_tab: self.active_lorebook_tab,
        };
        // Avoid pushing duplicates if nothing changed
        if let Some(last) = self.navigation_history.last() {
            // Simplified check: if IDs and mode/view match, don't push.
            // This prevents spamming history when clicking same thing.
            if last.mode == state.mode
                && last.central_view == state.central_view
                && last.selected_character_id == state.selected_character_id
                && last.selected_lorebook_id == state.selected_lorebook_id
                && last.selected_collection_id == state.selected_collection_id
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
                // Manually set selection instead of load_character to avoid side effects or re-pushing history
                if let Some(c) = self.characters.iter().find(|c| c.id == char_id).cloned() {
                    self.selected_character = Some(c);
                    // We might need to reload tabs/links if they aren't cached or if we want to ensure freshness
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
                }
            } else {
                self.selected_lorebook = None;
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

    pub fn get_descendant_collections(&self, parent_id: Option<i64>) -> Vec<i64> {
        let mut result = Vec::new();

        if let Some(pid) = parent_id {
            result.push(pid);

            // Find all direct children
            let children: Vec<i64> = self
                .collections
                .iter()
                .filter(|c| c.parent_id == Some(pid))
                .map(|c| c.id)
                .collect();

            // Recursively get descendants
            for child_id in children {
                result.extend(self.get_descendant_collections(Some(child_id)));
            }
        }

        result
    }

    pub fn toggle_lore_link(&mut self, char_id: i64, lore_id: i64, link: bool) {
        if char_id == 0 {
            return;
        }

        // Optimistic UI update
        if link {
            self.lore_links.insert(lore_id);
            self.char_lore_map.entry(char_id).or_default().push(lore_id);
        } else {
            self.lore_links.remove(&lore_id);
            if let Some(links) = self.char_lore_map.get_mut(&char_id) {
                links.retain(|&id| id != lore_id);
            }
        }

        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let res = if link {
                db.link_lore(char_id, lore_id).await
            } else {
                db.unlink_lore(char_id, lore_id).await
            };
            let _ = tx
                .send(UiEvent::LinkUpdated(res.map_err(|e| e.to_string())))
                .await;
            ctx.request_repaint();
        });
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
        } else {
            false
        }
    }

    pub fn create_new_character(&mut self, collection_id: Option<i64>) {
        self.push_history();
        let mut character = Character::default();
        character.collection_id = collection_id;

        // Immediate save
        self.save_character(character);

        // UI Navigation handled by event loop when CharacterSaved(Ok(c)) returns,
        // but we can set mode here for immediate visual switch if desired.
        // Actually, let's let the event loop handle selection, but switch mode now.
        self.mode = AppMode::Characters;
        self.central_view = CentralView::Editor;
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

    pub fn toggle_favorite(&mut self, char_id: i64) {
        if let Some(c) = self.characters.iter_mut().find(|c| c.id == char_id) {
            c.is_favorite = !c.is_favorite;
            // Persist
            let mut char_clone = c.clone();
            // We use save_character which handles upsert.
            // But save_character might be too heavy if it reloads everything?
            // Actually it spawns a task and eventually reloads chars.
            // That's fine for now.
            self.save_character(char_clone);
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

    pub fn perform_action(&mut self, action: AppAction, ctx: &egui::Context) {
        match action {
            AppAction::SwitchCharacter(id) => self.load_character(id),
            AppAction::SwitchCollection(id) => {
                self.viewing_all_characters = false;
                self.selected_collection_id = id;
                self.mode = AppMode::Characters;
                self.central_view = CentralView::Browser;
                self.selected_character = None;
                self.reload_collections();
            }
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
        }
    }

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

    pub fn add_tag(&self, char_id: i64, name: String, is_external: bool) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let res = db
                .add_tag_to_character(char_id, &name, is_external)
                .await
                .map_err(|e| e.to_string());
            let _ = tx.send(UiEvent::TagOperationFinished(res)).await;
            ctx.request_repaint();
        });
    }

    pub fn remove_tag(&self, char_id: i64, tag_id: i64, is_external: bool) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let res = db
                .remove_tag_from_character(char_id, tag_id, is_external)
                .await
                .map_err(|e| e.to_string());
            let _ = tx.send(UiEvent::TagOperationFinished(res)).await;
            ctx.request_repaint();
        });
    }

    pub fn load_lorebook_tags(&self, lorebook_id: i64) {
        if lorebook_id == 0 {
            return;
        }
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            match db.get_tags_for_lorebook(lorebook_id).await {
                Ok(tags) => {
                    let _ = tx
                        .send(UiEvent::LorebookTagsLoaded(Ok((lorebook_id, tags))))
                        .await;
                }
                Err(e) => {
                    let _ = tx
                        .send(UiEvent::LorebookTagsLoaded(Err(e.to_string())))
                        .await;
                }
            }
            ctx.request_repaint();
        });
    }

    pub fn add_tag_to_lorebook(&self, lorebook_id: i64, name: String) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let res = db.add_tag_to_lorebook(lorebook_id, &name).await;

            match res {
                Ok(_) => {
                    let tags = db.get_tags_for_lorebook(lorebook_id).await;
                    if let Ok(t) = tags {
                        let _ = tx
                            .send(UiEvent::LorebookTagsLoaded(Ok((lorebook_id, t))))
                            .await;
                    }
                    let _ = tx.send(UiEvent::LorebookTagOperationFinished(Ok(()))).await;
                }
                Err(e) => {
                    let _ = tx
                        .send(UiEvent::LorebookTagOperationFinished(Err(e.to_string())))
                        .await;
                }
            }
            ctx.request_repaint();
        });
    }

    pub fn remove_tag_from_lorebook(&self, lorebook_id: i64, tag_id: i64) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            match db.remove_tag_from_lorebook(lorebook_id, tag_id).await {
                Ok(_) => {
                    let tags = db.get_tags_for_lorebook(lorebook_id).await;
                    if let Ok(t) = tags {
                        let _ = tx
                            .send(UiEvent::LorebookTagsLoaded(Ok((lorebook_id, t))))
                            .await;
                    }
                    let _ = tx.send(UiEvent::LorebookTagOperationFinished(Ok(()))).await;
                }
                Err(e) => {
                    let _ = tx
                        .send(UiEvent::LorebookTagOperationFinished(Err(e.to_string())))
                        .await;
                }
            }
            ctx.request_repaint();
        });
    }

    pub fn load_lorebook_entries(&self, lorebook_id: i64) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            match db.get_entries_for_lorebook(lorebook_id).await {
                Ok(entries) => {
                    let _ = tx
                        .send(UiEvent::LorebookEntriesLoaded(Ok((lorebook_id, entries))))
                        .await;
                }
                Err(e) => {
                    let _ = tx
                        .send(UiEvent::LorebookEntriesLoaded(Err(e.to_string())))
                        .await;
                }
            }
            ctx.request_repaint();
        });
    }

    pub fn add_entry_to_lorebook(&self, lorebook_id: i64) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let mut entry = crate::models::LorebookEntry::default();
            entry.lorebook_id = lorebook_id;

            match db.add_entry_to_lorebook(&entry).await {
                Ok(id) => {
                    let _ = tx.send(UiEvent::LorebookEntryAdded(Ok(id))).await;
                    // Auto-reload
                    match db.get_entries_for_lorebook(lorebook_id).await {
                        Ok(entries) => {
                            let _ = tx
                                .send(UiEvent::LorebookEntriesLoaded(Ok((lorebook_id, entries))))
                                .await;
                        }
                        Err(_) => {}
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(UiEvent::LorebookEntryAdded(Err(e.to_string())))
                        .await;
                }
            }
            ctx.request_repaint();
        });
    }

    pub fn save_lorebook_entry(&self, entry: crate::models::LorebookEntry) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            match db.update_lorebook_entry(&entry).await {
                Ok(_) => {
                    let _ = tx.send(UiEvent::LorebookEntrySaved(Ok(()))).await;
                    if let Ok(entries) = db.get_entries_for_lorebook(entry.lorebook_id).await {
                        let _ = tx
                            .send(UiEvent::LorebookEntriesLoaded(Ok((
                                entry.lorebook_id,
                                entries,
                            ))))
                            .await;
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(UiEvent::LorebookEntrySaved(Err(e.to_string())))
                        .await;
                }
            }
            ctx.request_repaint();
        });
    }

    pub fn perform_deep_search(&mut self) {
        // ... (This logically could be here or in global_search module if it was purely logic,
        // but it modifies App state heavily.
        // Let's call the helper in global_search? No, global_search was ui rendering.
        // Actually, logic IS in CrapApp usually.
        // I will keep the Logic here, but the Render in global_search.rs.
        // Wait, I didn't move the perform_deep_search LOGIC to global_search.rs, I just made render call it.
        // So I must implement it here.

        if self.deep_search_query.trim().is_empty() {
            return;
        }

        self.is_deep_searching = true;
        self.mode = AppMode::DeepSearch;
        self.deep_search_results.clear();

        let query = self.deep_search_query.clone();
        let filter_collection = self.deep_search_filter_collection;
        let char_filters = self.deep_search_char_field_filters.clone();
        let all_collections = self.collections.clone();
        let tx = self.tx.clone();
        let db = self.db.clone();
        let lore_filters = self.deep_search_lore_field_filters.clone();

        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let mut results = Vec::new();

            // 1. Search Characters Text
            let mut char_map: std::collections::HashMap<i64, Character> =
                std::collections::HashMap::new();

            if let Ok(chars) = db.search_characters_text(&query).await {
                for c in chars {
                    char_map.insert(c.id, c);
                }
            }

            // 2. Search Tags
            let mut tag_matches: Vec<(i64, String, bool)> = Vec::new();
            if char_filters.tags {
                if let Ok(tags) = db.search_tags_matching(&query).await {
                    tag_matches = tags;
                }
            }

            // 3. Fetch missing characters found by tags
            let found_ids: std::collections::HashSet<i64> =
                tag_matches.iter().map(|(id, _, _)| *id).collect();
            let missing_ids: Vec<i64> = found_ids
                .into_iter()
                .filter(|id| !char_map.contains_key(id))
                .collect();

            if !missing_ids.is_empty() {
                if let Ok(fetched) = db.get_characters_by_ids(&missing_ids).await {
                    for c in fetched {
                        char_map.insert(c.id, c);
                    }
                }
            }

            // 3.5. Fetch URLs for result candidates
            if char_filters.urls && !char_map.is_empty() {
                if let Ok(urls) = db.get_all_character_urls_flat().await {
                    for u in urls {
                        if let Some(c) = char_map.get_mut(&u.character_id) {
                            c.urls.push(u);
                        }
                    }
                }
            }

            // 4. Build Character Results

            for (_, c) in char_map {
                let mut matches = Vec::new();

                // Use widget helper
                use crate::ui::widgets::extract_snippets;

                if char_filters.name {
                    for s in extract_snippets(&c.name, &query) {
                        matches.push(("Name".to_string(), s));
                    }
                }
                if char_filters.char_title {
                    for s in extract_snippets(&c.char_title, &query) {
                        matches.push(("Title".to_string(), s));
                    }
                }
                if char_filters.personality {
                    for s in extract_snippets(&c.personality, &query) {
                        matches.push(("Personality".to_string(), s));
                    }
                }
                if char_filters.scenario {
                    for s in extract_snippets(&c.scenario, &query) {
                        matches.push(("Scenario".to_string(), s));
                    }
                }
                if char_filters.example_dialogue {
                    for s in extract_snippets(&c.example_dialogue, &query) {
                        matches.push(("Example Dialogue".to_string(), s));
                    }
                }
                if char_filters.first_message {
                    for s in extract_snippets(&c.first_message, &query) {
                        matches.push(("First Message".to_string(), s));
                    }
                }
                if char_filters.author_notes {
                    for s in extract_snippets(&c.author_notes, &query) {
                        matches.push(("Notes".to_string(), s));
                    }
                }

                if char_filters.urls {
                    for url in &c.urls {
                        for s in extract_snippets(&url.url, &query) {
                            matches.push(("URL".to_string(), s));
                        }
                        if let Some(label) = &url.label {
                            for s in extract_snippets(label, &query) {
                                matches.push(("URL Label".to_string(), s));
                            }
                        }
                    }
                }

                if char_filters.tags {
                    for (tid, tname, is_ext) in &tag_matches {
                        if *tid == c.id {
                            let label = if *is_ext { "Ext. Tag" } else { "App Tag" };
                            matches.push((label.to_string(), tname.clone()));
                        }
                    }
                }

                if !matches.is_empty() {
                    results.push(DeepSearchResult {
                        id: c.id,
                        kind: SearchResultKind::Character,
                        display_name: c.name,
                        collection_id: c.collection_id,
                        matches,
                    });
                }
            }

            // 5. Build Lorebook Results
            use crate::ui::widgets::extract_snippets;
            let mut lorebook_map: std::collections::HashMap<i64, Lorebook> =
                std::collections::HashMap::new();

            // 5.1 Text Search
            if let Ok(books) = db.search_lorebooks_text(&query).await {
                for b in books {
                    lorebook_map.insert(b.id, b);
                }
            }

            // 5.2 Tags Search
            let mut lb_tag_matches: Vec<(i64, String)> = Vec::new();
            if let Ok(tags) = db.search_lorebook_tags_matching(&query).await {
                lb_tag_matches = tags;
            }

            // 5.3 Entries Search
            let mut entry_matches: Vec<crate::models::LorebookEntry> = Vec::new();
            if let Ok(entries) = db.search_lorebook_entries_text(&query).await {
                entry_matches = entries;
            }

            // 5.4 Fetch Missing Lorebooks
            let mut missing_lb_ids: std::collections::HashSet<i64> =
                std::collections::HashSet::new();
            for (lid, _) in &lb_tag_matches {
                if !lorebook_map.contains_key(lid) {
                    missing_lb_ids.insert(*lid);
                }
            }
            for entry in &entry_matches {
                if !lorebook_map.contains_key(&entry.lorebook_id) {
                    missing_lb_ids.insert(entry.lorebook_id);
                }
            }

            if !missing_lb_ids.is_empty() {
                let ids: Vec<i64> = missing_lb_ids.into_iter().collect();
                if let Ok(fetched) = db.get_lorebooks_by_ids(&ids).await {
                    for b in fetched {
                        lorebook_map.insert(b.id, b);
                    }
                }
            }

            // 5.5 Aggregate Matches
            for (_, mut lb) in lorebook_map {
                let mut matches = Vec::new();

                // 5.5.1 Lorebook Text Matches
                if lore_filters.title {
                    for s in extract_snippets(&lb.title, &query) {
                        matches.push(("Title".to_string(), s));
                    }
                }
                if lore_filters.description {
                    for s in extract_snippets(&lb.description, &query) {
                        matches.push(("Description".to_string(), s));
                    }
                    for s in extract_snippets(&lb.content, &query) {
                        matches.push(("Content".to_string(), s));
                    }
                }

                // 5.5.2 Tag Matches
                if lore_filters.tags {
                    for (lid, tname) in &lb_tag_matches {
                        if *lid == lb.id {
                            matches.push(("Tag".to_string(), tname.clone()));
                        }
                    }
                }

                // 5.5.3 Entry Matches
                for entry in &entry_matches {
                    if entry.lorebook_id == lb.id {
                        if lore_filters.entry_name {
                            for s in extract_snippets(&entry.name, &query) {
                                matches.push((format!("Entry: {}", entry.name), s));
                            }
                        }
                        if lore_filters.entry_keywords {
                            for s in extract_snippets(&entry.keywords, &query) {
                                matches.push((format!("Entry Keywords: {}", entry.name), s));
                            }
                        }
                        if lore_filters.entry_content {
                            for s in extract_snippets(&entry.content, &query) {
                                matches.push((format!("Entry Content: {}", entry.name), s));
                            }
                        }
                    }
                }

                if !matches.is_empty() {
                    results.push(DeepSearchResult {
                        id: lb.id,
                        kind: SearchResultKind::Lorebook,
                        display_name: lb.title,
                        collection_id: None,
                        matches,
                    });
                }
            }

            // Filter by collection if specified
            if let Some(filter_coll_id) = filter_collection {
                // Get all allowed collection IDs (parent + all descendants)
                let allowed_collections = {
                    let mut allowed = vec![filter_coll_id];

                    // Recursively find all children
                    let mut to_process = vec![filter_coll_id];
                    while let Some(parent_id) = to_process.pop() {
                        let children: Vec<i64> = all_collections
                            .iter()
                            .filter(|c| c.parent_id == Some(parent_id))
                            .map(|c| c.id)
                            .collect();

                        allowed.extend(&children);
                        to_process.extend(children);
                    }

                    allowed
                };

                // Filter results to only include characters from allowed collections
                results.retain(|res| {
                    if res.kind == SearchResultKind::Character {
                        if let Some(cid) = res.collection_id {
                            allowed_collections.contains(&cid)
                        } else {
                            false // Exclude uncategorized characters when filtering
                        }
                    } else {
                        true // Keep lorebooks regardless of filter
                    }
                });
            }

            let _ = tx.send(UiEvent::DeepSearchCompleted(Ok(results))).await;
            ctx.request_repaint();
        });
    }

    pub fn set_theme(&mut self, theme: ThemeMode) {
        self.theme = theme;
        self.apply_theme();

        let db = self.db.clone();
        let val = theme.to_string();
        tokio::spawn(async move {
            let _ = db.set_setting("theme", &val).await;
        });
    }

    pub fn apply_theme(&self) {
        match self.theme {
            ThemeMode::System => {
                self.ctx.set_style(egui::Style::default());
            }
            ThemeMode::Light => {
                self.ctx.set_visuals(egui::Visuals::light());
            }
            ThemeMode::Dark => {
                self.ctx.set_visuals(egui::Visuals::dark());
            }
        }
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.ui_scale = scale;
        self.ctx.set_pixels_per_point(scale);

        let db = self.db.clone();
        let val = scale.to_string();
        tokio::spawn(async move {
            let _ = db.set_setting("ui_scale", &val).await;
        });
    }
}

impl eframe::App for CrapApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Event Loop
        let mut received_event = false;
        while let Ok(event) = self.rx.try_recv() {
            received_event = true;
            match event {
                UiEvent::CharactersLoaded(res) => match res {
                    Ok(list) => {
                        self.characters = list;
                        self.loading_error = None;
                    }
                    Err(e) => {
                        eprintln!("Load error: {}", e);
                        self.loading_error = Some(e);
                    }
                },
                UiEvent::LorebooksLoaded(res) => match res {
                    Ok(books) => self.lorebooks = books,
                    Err(e) => {
                        self.loading_error = Some(e);
                    }
                },
                UiEvent::CollectionsLoaded(res) => match res {
                    Ok(collections) => self.collections = collections,
                    Err(e) => {
                        self.loading_error = Some(e);
                    }
                },
                UiEvent::ThemeLoaded(res) => {
                    if let Ok(mode) = res {
                        self.theme = mode;
                        self.apply_theme();
                    }
                }
                UiEvent::ScaleLoaded(res) => {
                    if let Ok(scale) = res {
                        self.ui_scale = scale;
                        self.ctx.set_pixels_per_point(scale);
                    }
                }
                UiEvent::LoreLinksLoaded(res) => match res {
                    Ok(set) => self.lore_links = set,
                    Err(e) => eprintln!("Link load error: {}", e),
                },
                UiEvent::LoreLinksBulkLoaded(map) => {
                    self.char_lore_map = map;
                }
                UiEvent::CharacterSaved(res) => {
                    self.is_saving = false;
                    match res {
                        Ok(c) => {
                            // Ensure links and tags are loaded (critical for new characters)
                            self.load_links(c.id);
                            self.load_tags(c.id);

                            self.selected_character = Some(c);
                            self.set_status("Character Saved!".to_string(), egui::Color32::GREEN);

                            // Handle pending action if any
                            if let Some(action) = self.pending_action.take() {
                                self.perform_action(action, &ctx);
                            }
                        }
                        Err(e) => {
                            self.set_status(format!("Save Error: {}", e), egui::Color32::RED);
                            self.pending_action = None;
                        }
                    }
                }
                UiEvent::LorebookSaved(res) => {
                    self.is_saving = false;
                    match res {
                        Ok(l) => {
                            self.selected_lorebook = Some(l);
                            self.set_status("Lorebook Saved!".to_string(), egui::Color32::GREEN);
                        }
                        Err(e) => self.set_status(format!("Save Error: {}", e), egui::Color32::RED),
                    }
                }
                UiEvent::CollectionSaved(res) => {
                    self.is_saving = false;
                    match res {
                        Ok(_) => {
                            self.set_status("Collection Saved!".to_string(), egui::Color32::GREEN);
                            self.reload_collections();
                            self.popup_state = PopupState::None;
                        }
                        Err(e) => self.set_status(format!("Save Error: {}", e), egui::Color32::RED),
                    }
                }
                UiEvent::CollectionDeleted(res) => {
                    self.is_saving = false;
                    match res {
                        Ok(id) => {
                            self.set_status("Collection Deleted".to_string(), egui::Color32::GREEN);
                            // Optimistic update
                            self.collections.retain(|c| c.id != id);
                            self.reload_collections();
                            self.reload_characters();
                            if self.selected_collection_id == Some(id) {
                                self.selected_collection_id = None;
                            }
                        }
                        Err(e) => {
                            self.set_status(format!("Delete Error: {}", e), egui::Color32::RED)
                        }
                    }
                }
                UiEvent::LinkUpdated(res) => {
                    if let Err(e) = res {
                        self.set_status(format!("Link Error: {}", e), egui::Color32::RED);
                    }
                }
                UiEvent::TagsLoaded(res) => match res {
                    Ok((id, app, ext)) => {
                        if let Some(c) = &mut self.selected_character {
                            if c.id == id {
                                c.app_tags = app;
                                c.external_tags = ext;
                            }
                        }
                    }
                    Err(e) => self.set_status(format!("Tag Load Error: {}", e), egui::Color32::RED),
                },
                UiEvent::TagOperationFinished(res) => match res {
                    Ok(_) => {
                        if let Some(c) = &self.selected_character {
                            self.load_tags(c.id);
                        }
                        self.refresh_all();
                    }
                    Err(e) => self.set_status(format!("Tag Error: {}", e), egui::Color32::RED),
                },
                UiEvent::LorebookTagsLoaded(res) => match res {
                    Ok((id, tags)) => {
                        // Update selected if matches
                        if let Some(l) = &mut self.selected_lorebook {
                            if l.id == id {
                                l.tags = tags.clone();
                            }
                        }
                        // Update cache
                        if let Some(cached) = self.lorebooks.iter_mut().find(|b| b.id == id) {
                            cached.tags = tags;
                        }
                    }
                    Err(e) => eprintln!("Lorebook tags load error: {}", e),
                },
                UiEvent::LorebookTagOperationFinished(res) => {
                    if let Err(e) = res {
                        self.set_status(format!("Tag Error: {}", e), egui::Color32::RED);
                    }
                }
                UiEvent::LorebookEntriesLoaded(res) => match res {
                    Ok((lid, entries)) => {
                        // Update cache
                        if let Some(l) = self.lorebooks.iter_mut().find(|l| l.id == lid) {
                            l.entries = entries.clone();
                        }
                        // Update selected
                        if let Some(l) = &mut self.selected_lorebook {
                            if l.id == lid {
                                l.entries = entries;
                            }
                        }
                    }
                    Err(e) => self
                        .set_status(format!("Failed to load entries: {}", e), egui::Color32::RED),
                },
                UiEvent::LorebookEntryAdded(res) => match res {
                    Ok(_) => self.set_status("Entry added".to_string(), egui::Color32::GREEN),
                    Err(e) => {
                        self.set_status(format!("Failed to add entry: {}", e), egui::Color32::RED)
                    }
                },
                UiEvent::LorebookEntrySaved(res) => match res {
                    Ok(_) => {} // Silent save
                    Err(e) => {
                        self.set_status(format!("Failed to save entry: {}", e), egui::Color32::RED)
                    }
                },
                UiEvent::LorebookEntryDeleted(res) => match res {
                    Ok(_) => self.set_status("Entry deleted".to_string(), egui::Color32::GREEN),
                    Err(e) => self
                        .set_status(format!("Failed to delete entry: {}", e), egui::Color32::RED),
                },
                UiEvent::LorebookDeleted(res) => match res {
                    Ok(id) => {
                        self.set_status("Lorebook Deleted".to_string(), egui::Color32::GREEN);
                        self.lorebooks.retain(|b| b.id != id);
                        if let Some(selected) = &self.selected_lorebook {
                            if selected.id == id {
                                self.selected_lorebook = None;
                            }
                        }
                    }
                    Err(e) => self.set_status(format!("Delete Error: {}", e), egui::Color32::RED),
                },
                UiEvent::DeepSearchCompleted(res) => {
                    self.is_deep_searching = false;
                    match res {
                        Ok(results) => self.deep_search_results = results,
                        Err(e) => {
                            self.set_status(format!("Search failed: {}", e), egui::Color32::RED)
                        }
                    }
                }
                UiEvent::UiRepaint => {
                    // Just wakes the loop, nothing to do
                }
                UiEvent::CharacterDeleted(res) => {
                    match res {
                        Ok(id) => {
                            // Optimistic update
                            self.characters.retain(|c| c.id != id);
                            if let Some(selected) = &self.selected_character {
                                if selected.id == id {
                                    self.selected_character = None;
                                    self.central_view = CentralView::Browser;
                                }
                            }
                            self.set_status("Character Deleted".to_string(), egui::Color32::GREEN);
                        }
                        Err(e) => {
                            self.set_status(format!("Delete Error: {}", e), egui::Color32::RED)
                        }
                    }
                }
                UiEvent::CharacterMoved(res) => {
                    match res {
                        Ok((char_id, new_coll_id)) => {
                            self.set_status("Character Moved".to_string(), egui::Color32::GREEN);

                            // 1. Sync Selected Character (Fix for editor desync)
                            if let Some(selected) = &mut self.selected_character {
                                if selected.id == char_id {
                                    selected.collection_id = new_coll_id;
                                }
                            }

                            // 2. Optimistic List Update
                            if let Some(c) = self.characters.iter_mut().find(|c| c.id == char_id) {
                                c.collection_id = new_coll_id;
                            }

                            self.reload_characters();
                        }
                        Err(e) => self.set_status(format!("Move Error: {}", e), egui::Color32::RED),
                    }
                }
                UiEvent::ImportFileLoaded(res) => {
                    match res {
                        Ok(json_content) => {
                            if let Ok(mut char_obj) =
                                serde_json::from_str::<Character>(&json_content)
                            {
                                // Clean ID for new import
                                char_obj.id = 0;

                                // Map to ParsedCharacterData for review
                                let parsed = ParsedCharacterData {
                                    name: char_obj.name.clone(),
                                    title: char_obj.char_title.clone(),
                                    personality: char_obj.personality.clone(),
                                    scenario: char_obj.scenario.clone(),
                                    first_message: char_obj.first_message.clone(),
                                    example_dialogue: char_obj.example_dialogue.clone(),
                                    external_tags: char_obj
                                        .external_tags
                                        .iter()
                                        .map(|t| t.name.clone())
                                        .collect(),
                                    app_tags: char_obj
                                        .app_tags
                                        .iter()
                                        .map(|t| t.name.clone())
                                        .collect(),
                                    urls: char_obj.urls.clone(),
                                };

                                // Force "New Character" mode
                                self.selected_character = Some(Character::default());
                                self.mode = AppMode::Characters;

                                self.parsed_data = Some(parsed);
                                self.show_import_modal = true;
                                self.import_text.clear(); // Clear clipboard text if any

                                self.set_status_with_duration(
                                    "File loaded for review.".to_string(),
                                    egui::Color32::GREEN,
                                    Duration::from_secs(10),
                                );
                            } else {
                                self.set_status(
                                    "Failed to parse file structure.".to_string(),
                                    egui::Color32::RED,
                                );
                            }
                        }
                        Err(e) => self.set_status(format!("Read Error: {}", e), egui::Color32::RED),
                    }
                }
                UiEvent::DbExportFinished(res) => match res {
                    Ok(path) => self.set_status(
                        format!("Database exported to: {}", path),
                        egui::Color32::GREEN,
                    ),
                    Err(e) => self.set_status(format!("Export Failed: {}", e), egui::Color32::RED),
                },
                UiEvent::DbReloaded(res) => match res {
                    Ok(new_db) => {
                        self.db = new_db;
                        self.set_status(
                            "Database imported successfully. Reloading view...".to_string(),
                            egui::Color32::GREEN,
                        );
                        self.refresh_all();
                    }
                    Err(e) => {
                        self.set_status(
                            format!("CRITICAL: Database Swap Failed: {}", e),
                            egui::Color32::RED,
                        );
                    }
                },
                UiEvent::CollectionIconUpdated(res) => match res {
                    Ok(_) => {
                        self.set_status(
                            "Collection Icon Updated".to_string(),
                            egui::Color32::GREEN,
                        );
                        self.reload_collections();
                    }
                    Err(e) => {
                        self.set_status(format!("Icon Update Error: {}", e), egui::Color32::RED)
                    }
                },
            }
        }

        if received_event {
            ctx.request_repaint();
        }

        // Timer
        if let Some(deadline) = self.status_clear_time {
            if Instant::now() > deadline {
                self.status_message = None;
                self.status_clear_time = None;
            } else {
                ctx.request_repaint();
            }
        }

        // Handle Close Request
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.has_unsaved_changes() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.popup_state = PopupState::UnsavedChanges {
                    target: AppAction::Exit,
                };
            }
        }

        // Side Panel
        side_panel::render_side_panel(self, ctx);

        // Central Panel
        central_panel::render_central_panel(self, ctx);

        // Global Popups
        popups::render_popups(ctx, self);

        // Watermark: The Library of Snailexandria
        if ctx.screen_rect().width() > 300.0 {
            egui::Area::new("watermark_area".into())
                .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-10.0, -5.0))
                .order(egui::Order::Foreground)
                .interactable(false)
                .show(ctx, |ui| {
                    ui.style_mut().interaction.selectable_labels = false;
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("The Library of Snailexandria")
                                .size(11.0)
                                .color(egui::Color32::from_white_alpha(100))
                                .italics(),
                        );
                    });
                });
        }
    }
}

impl CrapApp {
    pub fn trigger_db_export(&self) {
        let db = self.db.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            // Checkpoint first
            if let Err(e) = db.checkpoint().await {
                let _ = tx
                    .send(UiEvent::DbExportFinished(Err(format!(
                        "Checkpoint failed: {}",
                        e
                    ))))
                    .await;
                return;
            }

            if let Some(path) = rfd::FileDialog::new()
                .set_title("Export Database Backup")
                .set_file_name("crap_data_backup.db")
                .save_file()
            {
                match std::fs::copy("crap_data.db", &path) {
                    Ok(_) => {
                        let _ = tx
                            .send(UiEvent::DbExportFinished(Ok(path
                                .to_string_lossy()
                                .to_string())))
                            .await;
                    }
                    Err(e) => {
                        let _ = tx.send(UiEvent::DbExportFinished(Err(e.to_string()))).await;
                    }
                }
            }
        });
    }

    pub fn trigger_db_import(&self) {
        let db = self.db.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Import Database (Restores Backup)")
                .add_filter("SQLite Database", &["db", "sqlite", "sqlite3"])
                .pick_file()
            {
                // 1. Validate
                if let Err(e) = Database::validate_candidate(&path).await {
                    let _ = tx
                        .send(UiEvent::DbReloaded(Err(format!(
                            "Validation Failed: {}",
                            e
                        ))))
                        .await;
                    return;
                }

                // 2. Hot Swap Logic
                // Close current
                db.close().await;

                // Backup current
                let _ = std::fs::copy("crap_data.db", "crap_data.db.old");

                // Replace
                if let Err(e) = std::fs::copy(&path, "crap_data.db") {
                    // Try to restore old?
                    let _ = std::fs::copy("crap_data.db.old", "crap_data.db");
                    let _ = tx
                        .send(UiEvent::DbReloaded(Err(format!(
                            "Copy Failed (Restored): {}",
                            e
                        ))))
                        .await;
                    // We still need to re-init DB because we closed pool.
                    // But we fail properly.
                }

                // 3. Re-init
                match Database::init().await {
                    Ok(new_db) => {
                        let _ = tx.send(UiEvent::DbReloaded(Ok(new_db))).await;
                    }
                    Err(e) => {
                        let _ = tx
                            .send(UiEvent::DbReloaded(Err(format!("Re-init Failed: {}", e))))
                            .await;
                    }
                }
            }
        });
    }
}

fn cleanup_avatar(path_str: &str) {
    let path = std::path::Path::new(path_str);
    // Security check: Only delete if inside "data/avatars"
    // Normalize logic loosely by checking components or starts_with
    if path_str.replace("\\", "/").contains("data/avatars/") {
        if path.exists() {
            if let Err(e) = std::fs::remove_file(path) {
                eprintln!("Failed to delete old avatar {}: {}", path_str, e);
            } else {
                println!("Deleted old avatar: {}", path_str);
            }
        }
    }
}
