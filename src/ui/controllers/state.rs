use crate::db::Database;
use eframe::egui;

use crate::models::{Character, Collection, DeepSearchResult, Lorebook, Template, ThemeMode};

use tokio::sync::mpsc;

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::ui::spell_check;
use crate::ui::types::*;
use crate::ui::{
    views::search::{CharacterSearchFieldFilters, LorebookSearchFieldFilters},
    ParsedCharacterData, PopupState,
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
    pub active_settings_tab: SettingsTab,
    pub active_lorebook_tab: LorebookTab,
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

    pub show_statistics_window: bool,
    pub statistics_state: Option<StatisticsState>,

    pub viewing_all_characters: bool,
    pub viewing_favorites: bool,
    pub pending_action: Option<AppAction>,

    // Preferences
    pub count_name_in_total: bool,
    pub count_title_in_total: bool,
    pub count_first_message_in_total: bool,
    pub count_personality_in_total: bool,
    pub count_scenario_in_total: bool,
    pub count_example_in_total: bool,

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
    pub background_scale: f32,
    pub enable_spell_check: bool,
    pub editor_font: EditorFontFamily,
    pub editor_large_font: bool,

    pub editor_bright_mode: bool,
    pub blur_all_images: bool,
    pub blur_all_nsfw: bool,
    pub blur_overrides: std::collections::HashMap<i64, bool>,

    // Gallery Zoom
    pub gallery_zoom: f32,
    pub gallery_pan: egui::Vec2,

    // Smart Tab Switching
    pub last_active_character_id: Option<i64>,
    pub last_active_lorebook_id: Option<i64>,
    pub check_updates_at_start: bool,
    pub is_checking_for_updates: bool,

    // Cosmic Text Integration
    pub cosmic_font_system: egui_cosmic_text::cosmic_text::FontSystem,
    pub cosmic_swash_cache: egui_cosmic_text::cosmic_text::SwashCache,
    pub cosmic_atlas: egui_cosmic_text::atlas::TextureAtlas,
    pub cosmic_editors: std::collections::HashMap<
        String,
        egui_cosmic_text::widget::CosmicEdit<egui_cosmic_text::widget::FillWidth>,
    >,
    pub cosmic_clipboard: arboard::Clipboard,
    pub gallery_cache: HashMap<i64, std::sync::Arc<Vec<GalleryImage>>>,
    pub gallery_loading: HashSet<i64>,
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
            active_settings_tab: SettingsTab::General,
            active_lorebook_tab: LorebookTab::Entries,
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

            show_statistics_window: false,
            statistics_state: None,

            viewing_all_characters: false,
            viewing_favorites: false,
            pending_action: None,
            theme: ThemeMode::System,
            ui_scale: 1.0,

            count_name_in_total: false,
            count_title_in_total: true,
            count_first_message_in_total: true, // Default to true as it's a major section
            count_personality_in_total: true,
            count_scenario_in_total: true,
            count_example_in_total: true,

            navigation_history: Vec::new(),
            scale_last_updated: None,
            last_scroll_time: Instant::now(),
            focus_search_field: false,
            fullscreen_image: None,
            gallery_context: None,
            use_custom_background: false,
            show_watermark: true,
            show_background: true,
            background_scale: 0.9,
            enable_spell_check: true,
            editor_font: EditorFontFamily::SansSerif,
            editor_large_font: false,

            editor_bright_mode: true,
            blur_all_images: false,
            blur_all_nsfw: false,
            blur_overrides: std::collections::HashMap::new(),

            gallery_zoom: 1.0,
            gallery_pan: egui::vec2(0.0, 0.0),

            last_active_character_id: None,
            last_active_lorebook_id: None,
            check_updates_at_start: true,
            is_checking_for_updates: false,

            cosmic_font_system: egui_cosmic_text::cosmic_text::FontSystem::new(),
            cosmic_swash_cache: egui_cosmic_text::cosmic_text::SwashCache::new(),
            cosmic_atlas: egui_cosmic_text::atlas::TextureAtlas::new(
                cc.egui_ctx.clone(),
                egui::Color32::WHITE,
            ),
            cosmic_editors: std::collections::HashMap::new(),
            cosmic_clipboard: arboard::Clipboard::new().expect("Failed to initialize clipboard"),
            gallery_cache: HashMap::new(),
            gallery_loading: HashSet::new(),
        };

        // Initialize Settings
        app.initialize_settings();
        app
    }

    fn initialize_settings(&self) {
        // Initial Scale Load
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
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
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
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
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
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
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
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
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
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

        // Initial Background Scale Load
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            match db.get_setting("background_scale").await {
                Ok(Some(val)) => {
                    if let Ok(scale) = val.parse::<f32>() {
                        let _ = tx.send(UiEvent::BackgroundScaleLoaded(scale)).await;
                        ctx.request_repaint();
                    }
                }
                Ok(None) => {} // Default 0.9
                Err(e) => eprintln!("Failed to load background scale: {}", e),
            }
        });

        // Initial Spell Check Load
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
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

        // Initial Check Updates at Start Load
        let tx = self.tx.clone();
        let db = self.db.clone();
        tokio::spawn(async move {
            match db.get_setting("check_updates_at_start").await {
                Ok(Some(val)) => {
                    let enabled = val != "false"; // Default true if present but somehow empty? "false" is explicit false.
                    let _ = tx.send(UiEvent::CheckUpdatesAtStartLoaded(enabled)).await;
                }
                Ok(None) => {
                    // Default to true
                    let _ = tx.send(UiEvent::CheckUpdatesAtStartLoaded(true)).await;
                }
                Err(e) => eprintln!("Failed to load check updates setting: {}", e),
            }
        });

        // Initial Editor Font Load
        let tx = self.tx.clone();
        let db = self.db.clone();
        tokio::spawn(async move {
            match db.get_setting("editor_font").await {
                Ok(Some(val)) => {
                    if let Ok(font) = val.parse::<EditorFontFamily>() {
                        let _ = tx.send(UiEvent::EditorFontLoaded(font)).await;
                    }
                }
                Ok(None) => {}
                Err(e) => eprintln!("Failed to load editor font setting: {}", e),
            }
        });

        // Initial Large Font Load
        let tx = self.tx.clone();
        let db = self.db.clone();
        tokio::spawn(async move {
            match db.get_setting("editor_large_font").await {
                Ok(Some(val)) => {
                    let enabled = val == "true";
                    let _ = tx.send(UiEvent::EditorLargeFontLoaded(enabled)).await;
                }
                Ok(None) => {}
                Err(e) => eprintln!("Failed to load large font setting: {}", e),
            }
        });

        // Initial Bright Mode Load
        let tx = self.tx.clone();
        let db = self.db.clone();
        tokio::spawn(async move {
            match db.get_setting("editor_bright_mode").await {
                Ok(Some(val)) => {
                    let enabled = val != "false"; // Default true
                    let _ = tx.send(UiEvent::EditorBrightModeLoaded(enabled)).await;
                }
                Ok(None) => {
                    let _ = tx.send(UiEvent::EditorBrightModeLoaded(true)).await;
                }
                Err(e) => eprintln!("Failed to load bright mode setting: {}", e),
            }
        });

        // Initial Blur All Images Load
        let tx = self.tx.clone();
        let db = self.db.clone();
        tokio::spawn(async move {
            match db.get_setting("blur_all_images").await {
                Ok(Some(val)) => {
                    let enabled = val == "true";
                    let _ = tx.send(UiEvent::BlurAllImagesLoaded(enabled)).await;
                }
                Ok(None) => {
                    let _ = tx.send(UiEvent::BlurAllImagesLoaded(false)).await;
                }
                Err(e) => eprintln!("Failed to load blur all images setting: {}", e),
            }
        });

        // Initial Blur All NSFW Load
        let tx = self.tx.clone();
        let db = self.db.clone();
        tokio::spawn(async move {
            match db.get_setting("blur_all_nsfw").await {
                Ok(Some(val)) => {
                    let enabled = val == "true";
                    let _ = tx.send(UiEvent::BlurAllNsfwLoaded(enabled)).await;
                }
                Ok(None) => {
                    let _ = tx.send(UiEvent::BlurAllNsfwLoaded(false)).await;
                }
                Err(e) => eprintln!("Failed to load blur all nsfw setting: {}", e),
            }
        });

        self.refresh_all();
    }
}
