use crate::db::Database;
use eframe::egui;

use crate::models::{
    Character, Collection, DeepSearchResult, Lorebook, SearchResultKind, Tag, Template, ThemeMode,
};

use tokio::sync::mpsc;

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use super::spell_check;
use super::types::*;
use super::utils::cleanup_avatar;
use super::{
    CharacterSearchFieldFilters, LorebookSearchFieldFilters, ParsedCharacterData, PopupState,
};

pub struct CrapApp {
    pub db: Database,
    pub tx: mpsc::Sender<UiEvent>,
    pub rx: mpsc::Receiver<UiEvent>,
    pub ctx: egui::Context,

    // Data (Cached)
    pub characters: Vec<Character>,
    pub lorebooks: Vec<Lorebook>,
    pub templates: Vec<Template>,
    pub collections: Vec<Collection>,
    pub lore_links: HashSet<i64>,
    pub char_lore_map: HashMap<i64, Vec<i64>>,
    pub token_cache: HashMap<i64, (usize, usize)>,
    pub token_calc_in_progress: HashSet<i64>,

    // State
    pub mode: AppMode,
    pub selected_character: Option<Character>,
    pub selected_lorebook: Option<Lorebook>,
    pub selected_template: Option<Template>,
    pub selected_entry: Option<crate::models::LorebookEntry>,
    pub active_char_tab: CharacterTab,
    pub active_lorebook_tab: LorebookTab,
    pub active_template_tab: TemplateTab,
    pub central_view: CentralView,
    pub theme: ThemeMode,
    pub ui_scale: f32,
    pub sort_mode: SortMode,
    pub sort_direction: SortDirection,
    pub browser_sort_mode: SortMode,
    pub browser_sort_direction: SortDirection,
    pub browser_view_mode: BrowserViewMode,
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
    pub deep_search_sort: Option<SortDirection>,
    pub is_deep_searching: bool,
    pub editor_search_query: String, // In-editor search

    // Tag editor
    pub app_tag_input: String,
    pub ext_tag_input: String,

    // Spell Checker
    pub spell_checker: Option<std::sync::Arc<spell_check::SpellChecker>>,

    // Import Modal State
    pub show_import_modal: bool,
    pub show_options_window: bool,
    pub import_text: String,
    pub parsed_data: Option<ParsedCharacterData>,

    pub viewing_all_characters: bool,
    pub viewing_favorites: bool,
    pub pending_action: Option<AppAction>,

    // Preferences
    pub count_title_in_total: bool,

    // Navigation History
    pub navigation_history: Vec<NavigationState>,

    pub scale_last_updated: Option<Instant>,
    pub last_scroll_time: Instant,

    pub focus_search_field: bool,

    // Lightbox
    pub fullscreen_image: Option<String>,
    pub gallery_context: Option<Vec<String>>,
    pub use_custom_background: bool,
    pub show_watermark: bool,
    pub show_background: bool,
    pub enable_spell_check: bool,

    // Smart Tab Switching
    pub last_active_character_id: Option<i64>,
    pub last_active_lorebook_id: Option<i64>,
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
            templates: Vec::new(),
            collections: Vec::new(),
            lore_links: HashSet::new(),
            char_lore_map: HashMap::new(),
            token_cache: HashMap::new(),
            token_calc_in_progress: HashSet::new(),
            mode: AppMode::Characters,
            selected_character: None,
            selected_lorebook: None,
            selected_template: None,
            selected_entry: None,

            active_char_tab: CharacterTab::MainData,
            active_lorebook_tab: LorebookTab::Entries,
            active_template_tab: TemplateTab::Details,
            central_view: CentralView::Browser,
            sort_mode: SortMode::Alphabetical,
            sort_direction: SortDirection::Ascending,
            browser_sort_mode: SortMode::Alphabetical,
            browser_sort_direction: SortDirection::Ascending,
            browser_view_mode: BrowserViewMode::Grid,
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
            deep_search_sort: None,
            is_deep_searching: false,
            editor_search_query: String::new(),
            app_tag_input: String::new(),
            ext_tag_input: String::new(),

            spell_checker: spell_check::SpellChecker::new().map(std::sync::Arc::new),

            show_import_modal: false,
            show_options_window: false,
            import_text: String::new(),
            parsed_data: None,

            viewing_all_characters: false,
            viewing_favorites: false,
            pending_action: None,
            theme: ThemeMode::System,
            ui_scale: 1.0,

            count_title_in_total: false,

            navigation_history: Vec::new(),
            scale_last_updated: None,
            last_scroll_time: Instant::now(),
            focus_search_field: false,
            fullscreen_image: None,
            gallery_context: None,
            use_custom_background: false,
            show_watermark: true,
            show_background: true,
            enable_spell_check: true,

            last_active_character_id: None,
            last_active_lorebook_id: None,
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

        // Initial Background Setting Load
        let tx = app.tx.clone();
        let db = app.db.clone();
        let ctx = app.ctx.clone();
        tokio::spawn(async move {
            match db.get_setting("use_custom_background").await {
                Ok(Some(val)) => {
                    let enabled = val == "true";
                    let _ = tx.send(UiEvent::CustomBackgroundLoaded(enabled)).await;
                    ctx.request_repaint();
                }
                Ok(None) => {}
                Err(e) => eprintln!("Failed to load background setting: {}", e),
            }
        });

        // Initial Watermark Load
        let tx = app.tx.clone();
        let db = app.db.clone();
        let ctx = app.ctx.clone();
        tokio::spawn(async move {
            match db.get_setting("show_watermark").await {
                Ok(Some(val)) => {
                    let show = val != "false";
                    let _ = tx.send(UiEvent::WatermarkLoaded(show)).await;
                    ctx.request_repaint();
                }
                Ok(None) => {
                    let _ = tx.send(UiEvent::WatermarkLoaded(true)).await;
                }
                Err(e) => eprintln!("Failed to load watermark setting: {}", e),
            }
        });

        // Initial Background Visibility Load
        let tx = app.tx.clone();
        let db = app.db.clone();
        let ctx = app.ctx.clone();
        tokio::spawn(async move {
            match db.get_setting("show_background").await {
                Ok(Some(val)) => {
                    let show = val != "false";
                    let _ = tx.send(UiEvent::BackgroundLoaded(show)).await;
                    ctx.request_repaint();
                }
                Ok(None) => {
                    let _ = tx.send(UiEvent::BackgroundLoaded(true)).await;
                }
                Err(e) => eprintln!("Failed to load background visibility setting: {}", e),
            }
        });

        // Initial Spell Check Load
        let tx = app.tx.clone();
        let db = app.db.clone();
        let ctx = app.ctx.clone();
        tokio::spawn(async move {
            match db.get_setting("enable_spell_check").await {
                Ok(Some(val)) => {
                    let enabled = val != "false";
                    let _ = tx.send(UiEvent::SpellCheckSettingLoaded(enabled)).await;
                    ctx.request_repaint();
                }
                Ok(None) => {
                    let _ = tx.send(UiEvent::SpellCheckSettingLoaded(true)).await;
                }
                Err(e) => eprintln!("Failed to load spell check setting: {}", e),
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

            // Load Templates
            match db.get_all_templates().await {
                Ok(templates) => {
                    let _ = tx.send(UiEvent::TemplatesLoaded(Ok(templates))).await;
                    ctx.request_repaint();
                }
                Err(e) => {
                    let _ = tx.send(UiEvent::TemplatesLoaded(Err(e.to_string()))).await;
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
            self.selected_lorebook = None; // Clear other selection
            self.selected_entry = None;
            self.load_links(id);
            self.load_tags(id);
            self.mode = AppMode::Characters;
            self.central_view = CentralView::Editor;
            self.last_active_character_id = Some(id);
        }
    }

    pub fn load_lorebook(&mut self, id: i64) {
        self.push_history();
        if let Some(book) = self.lorebooks.iter().find(|l| l.id == id).cloned() {
            self.selected_lorebook = Some(book);
            self.selected_character = None; // Clear other selection
            self.load_lorebook_entries(id);
            self.load_lorebook_tags(id);
            self.mode = AppMode::Lorebooks;
            self.central_view = CentralView::Editor;
            self.last_active_lorebook_id = Some(id);
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
                // Sync Tags (For both New and Existing)
                // We wipe and re-insert to match the editor state exactly (Overwrite behavior)
                // This handles deletions logic implicitly.
                let cid = character.id;

                // 1. External Tags
                let _ = db.remove_all_tags_from_character(cid, true).await;
                for tag in &character.external_tags {
                    let _ = db.add_tag_to_character(cid, &tag.name, true).await;
                }

                // 2. App Tags
                let _ = db.remove_all_tags_from_character(cid, false).await;
                for tag in &character.app_tags {
                    let _ = db.add_tag_to_character(cid, &tag.name, false).await;
                }

                if !is_new {
                    // Cleanup old avatar if changed
                    if let Some(path) = old_avatar_to_delete {
                        cleanup_avatar(&path);
                    }
                }

                // Reload tags to ensure we have correct database IDs (otherwise dirty check fails)
                // UPSERT handles URLs, but we manually handled Tags above, so we must reload them to get IDs.
                if let Ok(saved_app_tags) = db.get_tags_for_character(cid, false).await {
                    character.app_tags = saved_app_tags;
                }
                if let Ok(saved_ext_tags) = db.get_tags_for_character(cid, true).await {
                    character.external_tags = saved_ext_tags;
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
        if self.has_unsaved_changes() {
            self.popup_state = PopupState::UnsavedChanges {
                target: AppAction::CreateNewLorebook,
            };
        } else {
            self.perform_create_new_lorebook();
        }
    }

    pub fn perform_create_new_lorebook(&mut self) {
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
                let lid = lorebook.id;

                // 1. Sync Tags
                // Simple wipe and replace strategy for correctness with editor state
                // This assumes `lorebook.tags` is the source of truth
                let get_tags_res = db.get_tags_for_lorebook(lid).await;
                if let Ok(existing_tags) = get_tags_res {
                    for t in existing_tags {
                        let _ = db.remove_tag_from_lorebook(lid, t.id).await;
                    }
                }
                for tag in &lorebook.tags {
                    let _ = db.add_tag_to_lorebook(lid, &tag.name).await;
                }

                // 2. Sync Entries
                // We need to handle:
                // - Updates (existing ID)
                // - Inserts (ID 0)
                // - Deletions (ID exists in DB but not in `lorebook.entries`)
                let get_entries_res = db.get_entries_for_lorebook(lid).await;
                if let Ok(existing_entries) = get_entries_res {
                    let current_ids: HashSet<i64> = lorebook
                        .entries
                        .iter()
                        .filter(|e| e.id != 0)
                        .map(|e| e.id)
                        .collect();

                    for old_entry in existing_entries {
                        if !current_ids.contains(&old_entry.id) {
                            let _ = db.delete_lorebook_entry(old_entry.id).await;
                        }
                    }
                }

                let mut saved_entries = Vec::new();
                for mut entry in lorebook.entries.clone() {
                    entry.lorebook_id = lid; // Ensure consistency
                    entry.updated_at = chrono::Utc::now();

                    if entry.id == 0 {
                        entry.created_at = chrono::Utc::now(); // Is this correct? models usually handle default, but to be sure
                        if let Ok(new_id) = db.add_entry_to_lorebook(&entry).await {
                            entry.id = new_id;
                            saved_entries.push(entry);
                        }
                    } else {
                        if let Ok(_) = db.update_lorebook_entry(&entry).await {
                            saved_entries.push(entry);
                        }
                    }
                }

                // Update the object with saved entries (IDs populated)
                lorebook.entries = saved_entries;

                // Reload tags to ensure we have correct database IDs (otherwise dirty check fails)
                if let Ok(saved_tags) = db.get_tags_for_lorebook(lid).await {
                    lorebook.tags = saved_tags;
                }

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
            selected_lorebook_entry_id: self.selected_entry.as_ref().map(|e| e.id),
            selected_lorebook_entry_name: self.selected_entry.as_ref().map(|e| e.name.clone()),
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

                    if let Some(entry_id) = state.selected_lorebook_entry_id {
                        // We need access to the book's entries which might not be loaded yet if we rely on async load.
                        // However, load_lorebook_entries trigger async.
                        // BUT, self.selected_lorebook has entries if it was cloned?
                        // Wait, 'book' comes from 'self.lorebooks' which usually only has metadata if it's the main list.
                        // If we are lucky and it has entries (it might not), we can set selected_entry.
                        // If not, we rely on the async load to eventually populate it? No, async load sends event.
                        // We must set selected_entry here if possible, or set a "pending entry selection".
                        // For now, let's try to find it in the book we just set.
                        // NOTE: In CrapApp, self.lorebooks usually contains fully loaded books?
                        // Or just summaries? 'Lorebook' struct has entries vec.
                        // If it's empty, we might fail to show it immediately.
                        // Let's attempt to set it.
                        if let Some(b) = &self.selected_lorebook {
                            if let Some(entry) =
                                b.entries.iter().find(|e| e.id == entry_id).cloned()
                            {
                                self.selected_entry = Some(entry);
                            } else {
                                // If not found (maybe book entries not loaded), we clear it.
                                // Or we could trigger a "load and select" sequence.
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
            // Truncate history to the target index + 1 (so target becomes the last item)
            // But wait, go_back() pops the last item to use it.
            // If we want to go TO a state in history, we want that state to be active.
            // The history stack represents previous states properly.
            // go_back pops the TOP state and applies it.
            // If we want to jumping to index 'i', we effectively want to discard everything after 'i+1',
            // AND then pop 'i' to apply it? No.
            // The history stores PAST states. The CURRENT state is in 'self'.
            // If I have history [A, B, C] and current is D.
            // If I want to go to B (index 1).
            // I should truncate to [A, B] and then call go_back() which pops B and applies it?
            // Yes, that makes B the current state, and history becomes [A].
            // Wait, if B is the current state, it shouldn't be in history unless we push D?
            // We only push current state to history when we navigate AWAY.
            // So if we are compliant with go_back:
            // go_back pops the top (C), applies it. Buffer is now C. History is [A, B].
            // If we want to go to B.
            // We should truncate navigation_history to contain A and B.
            // Then pop B.
            // So truncate length to index + 1.
            self.navigation_history.truncate(index + 1);
            self.go_back();
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

    pub fn create_new_character(&mut self, collection_id: Option<i64>) {
        if self.has_unsaved_changes() {
            self.popup_state = PopupState::UnsavedChanges {
                target: AppAction::CreateNewCharacter(collection_id),
            };
        } else {
            self.perform_create_new_character(collection_id);
        }
    }

    pub fn perform_create_new_character(&mut self, collection_id: Option<i64>) {
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

    pub fn request_lorebook_switch(&mut self, id: i64) {
        if self.has_unsaved_changes() {
            self.popup_state = PopupState::UnsavedChanges {
                target: AppAction::SwitchLorebook(id),
            };
        } else {
            self.load_lorebook(id);
        }
    }

    pub fn toggle_favorite(&mut self, char_id: i64) {
        if let Some(c) = self.characters.iter_mut().find(|c| c.id == char_id) {
            c.is_favorite = !c.is_favorite;
            // Persist
            let char_clone = c.clone();
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
        self.deep_search_sort = None; // Reset sort on new search

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
                        index: 0, // Will be set after collection to ensure global order
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
            for (_, lb) in lorebook_map {
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
                        index: 0, // Will be set later
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

            // 6. Sort and Finalize
            // We want stable initial order, so we assign index here based on push order
            for (i, res) in results.iter_mut().enumerate() {
                res.index = i;
            }

            let _ = tx.send(UiEvent::DeepSearchCompleted(Ok(results))).await;
            ctx.request_repaint();
        });
    }

    pub fn sort_deep_search_results(&mut self) {
        if self.deep_search_results.is_empty() {
            return;
        }

        match self.deep_search_sort {
            Some(SortDirection::Ascending) => {
                self.deep_search_results.sort_by(|a, b| {
                    a.display_name
                        .to_lowercase()
                        .cmp(&b.display_name.to_lowercase())
                });
            }
            Some(SortDirection::Descending) => {
                self.deep_search_results.sort_by(|a, b| {
                    b.display_name
                        .to_lowercase()
                        .cmp(&a.display_name.to_lowercase())
                });
            }
            None => {
                // Restore original order
                self.deep_search_results.sort_by_key(|r| r.index);
            }
        }
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

    pub fn set_custom_background_mode(&mut self, enabled: bool) {
        self.use_custom_background = enabled;
        let db = self.db.clone();
        let val = if enabled { "true" } else { "false" };
        tokio::spawn(async move {
            let _ = db.set_setting("use_custom_background", val).await;
        });
    }

    pub fn set_watermark_visibility(&mut self, visible: bool) {
        self.show_watermark = visible;
        let db = self.db.clone();
        let val = visible.to_string();
        tokio::spawn(async move {
            let _ = db.set_setting("show_watermark", &val).await;
        });
    }

    pub fn set_background_visibility(&mut self, visible: bool) {
        self.show_background = visible;
        let db = self.db.clone();
        let val = visible.to_string();
        tokio::spawn(async move {
            let _ = db.set_setting("show_background", &val).await;
        });
    }

    pub fn set_spell_check(&mut self, enabled: bool) {
        self.enable_spell_check = enabled;
        let db = self.db.clone();
        let val = enabled.to_string();
        tokio::spawn(async move {
            let _ = db.set_setting("enable_spell_check", &val).await;
        });
    }

    pub fn ensure_token_count(&mut self, character: &Character) {
        if self.token_cache.contains_key(&character.id) {
            return;
        }
        if self.token_calc_in_progress.contains(&character.id) {
            return;
        }

        self.token_calc_in_progress.insert(character.id);
        let tx = self.tx.clone();
        let char_clone = character.clone();
        let include_title = self.count_title_in_total;

        tokio::spawn(async move {
            let mut total_tokens = 0;
            let mut total_chars = 0;

            // Note: Editor excludes Name from token count, so we do too.
            // fields: personality, scenario, example_dialogue, first_message, AND title (optional)

            let t_pers = crate::models::count_tokens(&char_clone.personality);
            let t_scen = crate::models::count_tokens(&char_clone.scenario);
            let t_ex = crate::models::count_tokens(&char_clone.example_dialogue);
            let t_first = crate::models::count_tokens(&char_clone.first_message);

            total_tokens += t_pers + t_scen + t_ex + t_first;

            total_chars += char_clone.personality.len();
            total_chars += char_clone.scenario.len();
            total_chars += char_clone.example_dialogue.len();
            total_chars += char_clone.first_message.len();

            if include_title {
                let t_title = crate::models::count_tokens(&char_clone.char_title);
                total_tokens += t_title;
                total_chars += char_clone.char_title.len();
            }

            let _ = tx
                .send(UiEvent::TokenCountCalculated(
                    char_clone.id,
                    total_tokens,
                    total_chars,
                ))
                .await;
        });
    }
    pub fn trigger_db_export_file_only(&self) {
        let db = self.db.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Export Database Value")
                .set_file_name(
                    format!(
                        "crap_data_backup_{}.db",
                        chrono::Local::now().format("%Y%m%d_%H%M%S")
                    )
                    .as_str(),
                )
                .save_file()
            {
                let target_str = path.to_string_lossy().to_string();

                // Use safe vacuum into
                match db.create_checkpoint_and_vacuum(&target_str).await {
                    Ok(_) => {
                        let _ = tx.send(UiEvent::DbExportFinished(Ok(target_str))).await;
                    }
                    Err(e) => {
                        let _ = tx
                            .send(UiEvent::DbExportFinished(Err(format!(
                                "Export Failed: {}",
                                e
                            ))))
                            .await;
                    }
                }
            }
        });
    }

    pub fn perform_full_zip_export(&self) {
        let db = self.db.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            if let Some(zip_path) = rfd::FileDialog::new()
                .set_title("Export Full Backup")
                .set_file_name(
                    format!(
                        "crap_backup_{}.zip",
                        chrono::Local::now().format("%Y%m%d_%H%M%S")
                    )
                    .as_str(),
                )
                .save_file()
            {
                // 1. Create Temp DB Snapshot
                let temp_db_name = format!("temp_snapshot_{}.db", uuid::Uuid::new_v4());
                let temp_db_path = std::env::temp_dir().join(&temp_db_name);
                let temp_db_str = temp_db_path.to_string_lossy().to_string();

                if let Err(e) = db.create_checkpoint_and_vacuum(&temp_db_str).await {
                    let _ = tx
                        .send(UiEvent::DbExportFinished(Err(format!(
                            "Snapshot Failed: {}",
                            e
                        ))))
                        .await;
                    return;
                }

                // 2. Create Zip
                let file = match std::fs::File::create(&zip_path) {
                    Ok(f) => f,
                    Err(e) => {
                        let _ = tx
                            .send(UiEvent::DbExportFinished(Err(format!(
                                "Create Zip Failed: {}",
                                e
                            ))))
                            .await;
                        // Cleanup
                        let _ = std::fs::remove_file(&temp_db_path);
                        return;
                    }
                };

                let mut zip = zip::ZipWriter::new(file);
                let options = zip::write::FileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated)
                    .unix_permissions(0o755);

                // 3. Add DB
                if let Err(e) = zip.start_file("crap_data.db", options) {
                    let _ = tx
                        .send(UiEvent::DbExportFinished(Err(format!(
                            "Zip DB Add Failed: {}",
                            e
                        ))))
                        .await;
                    let _ = std::fs::remove_file(&temp_db_path);
                    return;
                }

                if let Ok(mut f) = std::fs::File::open(&temp_db_path) {
                    if let Err(e) = std::io::copy(&mut f, &mut zip) {
                        let _ = tx
                            .send(UiEvent::DbExportFinished(Err(format!(
                                "Zip DB Write Failed: {}",
                                e
                            ))))
                            .await;
                        let _ = std::fs::remove_file(&temp_db_path);
                        return;
                    }
                }

                // Cleanup snapshot
                let _ = std::fs::remove_file(&temp_db_path);

                // 4. Add 'data' folder
                let data_dir = std::path::Path::new("data");
                if data_dir.exists() {
                    let walk = walkdir::WalkDir::new(data_dir);
                    for entry in walk.into_iter().filter_map(|e| e.ok()) {
                        let path = entry.path();
                        let name = path.strip_prefix(std::path::Path::new(".")).unwrap_or(path);
                        let name_str = name.to_string_lossy().replace("\\", "/"); // Zip requires forward slashes

                        if path.is_file() {
                            if let Err(e) = zip.start_file(name_str, options) {
                                eprintln!("Failed to start zip file {}: {}", path.display(), e);
                                continue;
                            }
                            if let Ok(mut f) = std::fs::File::open(path) {
                                let _ = std::io::copy(&mut f, &mut zip);
                            }
                        } else if path.is_dir() && !name.as_os_str().is_empty() {
                            let _ = zip.add_directory(name_str, options);
                        }
                    }
                }

                if let Err(e) = zip.finish() {
                    let _ = tx
                        .send(UiEvent::DbExportFinished(Err(format!(
                            "Zip Finish Failed: {}",
                            e
                        ))))
                        .await;
                } else {
                    let _ = tx
                        .send(UiEvent::DbExportFinished(Ok(zip_path
                            .to_string_lossy()
                            .to_string())))
                        .await;
                }
            }
        });
    }

    pub fn trigger_db_import(&self) {
        let db = self.db.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Import Data")
                .add_filter("All Supported", &["db", "sqlite", "sqlite3", "zip"])
                .add_filter("Database Files", &["db", "sqlite", "sqlite3"])
                .add_filter("Zip Backups", &["zip"])
                .pick_file()
            {
                // 1. Checkpoint current DB to ensure consistent state on disk
                if let Err(e) = db.checkpoint().await {
                    let _ = tx
                        .send(UiEvent::DbExportFinished(Err(format!(
                            "Pre-import checkpoint failed: {}",
                            e
                        ))))
                        .await;
                    return;
                }

                // 2. Close DB Connections using the existing async close
                db.close().await;

                // 3. Create Safety Backup
                let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                let backup_name = format!("crap_data_backup_{}.db", timestamp);
                if let Err(e) = std::fs::copy("crap_data.db", &backup_name) {
                    let _ = tx
                        .send(UiEvent::DbExportFinished(Err(format!(
                            "Auto-Backup Failed! Aborting import. Error: {}",
                            e
                        ))))
                        .await;
                    // Try to re-init
                    match Database::init().await {
                        Ok(new_db) => {
                            let _ = tx.send(UiEvent::DbReloaded(Ok(new_db))).await;
                        }
                        Err(re_e) => {
                            let _ = tx.send(UiEvent::DbReloaded(Err(re_e.to_string()))).await;
                        }
                    }
                    return;
                }

                let import_path = path.as_path();
                let extension = import_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                let result = if extension == "zip" {
                    // ZIP IMPORT
                    match std::fs::File::open(path.clone()) {
                        Ok(file) => {
                            match zip::ZipArchive::new(file) {
                                Ok(mut archive) => {
                                    // Extract everything
                                    match archive.extract(".") {
                                        Ok(_) => Ok(()),
                                        Err(e) => Err(format!("Unzip failed: {}", e)),
                                    }
                                }
                                Err(e) => Err(format!("Invalid Zip: {}", e)),
                            }
                        }
                        Err(e) => Err(format!("Could not open zip: {}", e)),
                    }
                } else {
                    // DB IMPORT
                    match std::fs::copy(path.clone(), "crap_data.db") {
                        Ok(_) => Ok(()),
                        Err(e) => Err(format!("DB Copy Failed: {}", e)),
                    }
                };

                match result {
                    Ok(_) => {
                        // Re-init DB
                        match Database::init().await {
                            Ok(new_db) => {
                                let _ = tx.send(UiEvent::DbReloaded(Ok(new_db))).await;
                            }
                            Err(re_e) => {
                                let _ = tx.send(UiEvent::DbReloaded(Err(re_e.to_string()))).await;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(UiEvent::DbExportFinished(Err(e))).await;
                        // Attempt re-init to restore state
                        match Database::init().await {
                            Ok(new_db) => {
                                let _ = tx.send(UiEvent::DbReloaded(Ok(new_db))).await;
                            }
                            Err(re_e) => {
                                let _ = tx.send(UiEvent::DbReloaded(Err(re_e.to_string()))).await;
                            }
                        }
                    }
                }
            }
        });
    }
    pub fn create_new_template(&mut self) {
        if self.has_unsaved_changes() {
            self.popup_state = PopupState::UnsavedChanges {
                target: AppAction::CreateNewTemplate,
            };
        } else {
            self.perform_create_new_template();
        }
    }

    pub fn perform_create_new_template(&mut self) {
        self.push_history();
        let new_template = Template::default();
        self.selected_template = Some(new_template.clone());
        self.save_template(new_template);
        self.mode = AppMode::Templates;
        self.selected_character = None;
        self.selected_lorebook = None;
    }

    pub fn save_template(&mut self, mut template: Template) {
        self.is_saving = true;
        self.status_message = None;
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            if let Err(e) = db.upsert_template(&mut template).await {
                let _ = tx.send(UiEvent::TemplateSaved(Err(e.to_string()))).await;
                ctx.request_repaint();
            } else {
                let _ = tx.send(UiEvent::TemplateSaved(Ok(template))).await;
                ctx.request_repaint();
                // Reload
                match db.get_all_templates().await {
                    Ok(templates) => {
                        let _ = tx.send(UiEvent::TemplatesLoaded(Ok(templates))).await;
                    }
                    Err(e) => {
                        let _ = tx.send(UiEvent::TemplatesLoaded(Err(e.to_string()))).await;
                    }
                }
                ctx.request_repaint();
            }
        });
    }

    pub fn delete_template(&self, id: i64) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let res = db.delete_template(id).await;
            let _ = tx
                .send(UiEvent::TemplateDeleted(
                    res.map(|_| id).map_err(|e| e.to_string()),
                ))
                .await;
            ctx.request_repaint();
        });
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
}
