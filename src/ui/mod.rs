use eframe::egui;
use crate::db::Database;
use crate::models::{Character, Lorebook, Collection, Tag, DeepSearchResult, count_tokens, SearchResultKind};

use tokio::sync::mpsc;
use std::collections::{HashSet, HashMap};
use std::time::{Duration, Instant};

pub mod side_panel;
pub mod central_panel;
pub mod global_search;
pub mod widgets;

// Re-export specific items if needed
pub use central_panel::ParsedCharacterData;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AppMode {
    Characters,
    Lorebooks,
    DeepSearch,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CharacterTab {
    MainData,
    AuthorNotes,
    AssociatedLore,
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

#[derive(Clone, Debug, PartialEq)]
pub enum AppAction {
    SwitchCharacter(i64),
    SwitchCollection(Option<i64>),
    SwitchToAll,
    Exit,
}

#[derive(Clone, Debug)]
pub enum PopupState {
    None,
    Renaming { id: i64, name: String },
    DeleteConfirmation { id: i64 },
    DeleteWarning { id: i64, count: usize },
    DeleteCharacterConfirmation { id: i64, name: String },
    UnsavedChanges { target: AppAction },
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
    TagOperationFinished(Result<(), String>),
    ImportFileLoaded(Result<String, String>),
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
    
    // State
    pub mode: AppMode,
    pub selected_character: Option<Character>,
    pub selected_lorebook: Option<Lorebook>,
    pub active_char_tab: CharacterTab,
    pub central_view: CentralView,
    pub sort_mode: SortMode,
    pub selected_collection_id: Option<i64>,
    
    pub popup_state: PopupState,
    pub is_saving: bool,
    pub status_message: Option<(String, egui::Color32)>,
    pub status_clear_time: Option<Instant>,
    pub loading_error: Option<String>,
    
    // Search
    pub search_query: String, // Side panel filter
    pub deep_search_query: String, // Global
    pub deep_search_results: Vec<DeepSearchResult>,
    pub is_deep_searching: bool,
    
    // Tag editor
    pub app_tag_input: String,
    pub ext_tag_input: String,
    
    // Import Modal State
    pub show_import_modal: bool,
    pub import_text: String,
    pub parsed_data: Option<ParsedCharacterData>,

    pub viewing_all_characters: bool,
    
    // Hidden internal for double-click/expand preservation if needed
    // We rely on egui id for collapsing headers.
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
            mode: AppMode::Characters,
            selected_character: None,
            selected_lorebook: None,
            active_char_tab: CharacterTab::MainData,
            central_view: CentralView::Editor,
            sort_mode: SortMode::Alphabetical,
            selected_collection_id: None,
            popup_state: PopupState::None,
            is_saving: false,
            status_message: None,
            status_clear_time: None,
            loading_error: None,
            search_query: String::new(),
            deep_search_query: String::new(),
            deep_search_results: Vec::new(),
            is_deep_searching: false,
            app_tag_input: String::new(),
            ext_tag_input: String::new(),
            
            show_import_modal: false,
            import_text: String::new(),
            parsed_data: None,
            viewing_all_characters: false,
        };
        
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
                    
                    if let (Ok(app_flat), Ok(ext_flat)) = (app_tags_res, ext_tags_res) {
                         let mut app_map: HashMap<i64, Vec<Tag>> = HashMap::new();
                         for (cid, tag) in app_flat {
                             app_map.entry(cid).or_default().push(tag);
                         }
                         
                         let mut ext_map: HashMap<i64, Vec<Tag>> = HashMap::new();
                         for (cid, tag) in ext_flat {
                             ext_map.entry(cid).or_default().push(tag);
                         }
                         
                         // Merge into characters
                         for c in &mut chars {
                             if let Some(tags) = app_map.remove(&c.id) {
                                 c.app_tags = tags;
                             }
                             if let Some(tags) = ext_map.remove(&c.id) {
                                 c.external_tags = tags;
                             }
                         }
                    } else {
                        eprintln!("Failed to load specific tags bulk");
                    }
                    
                    let _ = tx.send(UiEvent::CharactersLoaded(Ok(chars))).await;
                    ctx.request_repaint();
                },
                Err(e) => { 
                    let _ = tx.send(UiEvent::CharactersLoaded(Err(e.to_string()))).await;
                    ctx.request_repaint();
                }
            }
            
            // Load collections
            let collections_res = db.get_all_collections().await.map_err(|e| e.to_string());
            let _ = tx.send(UiEvent::CollectionsLoaded(collections_res)).await;
            ctx.request_repaint();
            
            // Load Lorebooks
            let books_res = db.get_all_lorebooks().await.map_err(|e| e.to_string());
            let _ = tx.send(UiEvent::LorebooksLoaded(books_res)).await;
            ctx.request_repaint();
        });
    }

    pub fn reload_characters(&self) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let result = db.get_all_characters().await.map_err(|e| e.to_string());
            let _ = tx.send(UiEvent::CharactersLoaded(result)).await;
            ctx.request_repaint();
        });
    }

    pub fn reload_lorebooks(&self) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let result = db.get_all_lorebooks().await.map_err(|e| e.to_string());
            let _ = tx.send(UiEvent::LorebooksLoaded(result)).await;
            ctx.request_repaint();
        });
    }

    pub fn load_links(&self, char_id: i64) {
        if char_id == 0 { return; } 
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
        if char_id == 0 { return; }
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
                },
                (Err(e), _) | (_, Err(e)) => {
                    let _ = tx.send(UiEvent::TagsLoaded(Err(e.to_string()))).await;
                    ctx.request_repaint();
                }
            }
        });
    }
    
    // Now just a simplified helper that spawns a load
    pub fn load_character(&mut self, id: i64) {
        // Find in logic, or reload if needed. Currently we just select from list.
        if let Some(c) = self.characters.iter().find(|c| c.id == id).cloned() {
            self.selected_character = Some(c);
            self.load_links(id);
            self.load_tags(id);
            self.mode = AppMode::Characters;
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
         let ctx = self.ctx.clone();
         let ctx = self.ctx.clone();
         tokio::spawn(async move {
              let res = db.delete_collection(id).await;
              let _ = tx.send(UiEvent::CollectionDeleted(res.map(|_| id).map_err(|e| e.to_string()))).await;
              ctx.request_repaint();
         });
    }

    pub fn save_character(&mut self, mut character: Character) {
        self.is_saving = true;
        self.status_message = None;
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
                        let _ = db.add_tag_to_character(character.id, &tag.name, false).await;
                    }
                }

                let _ = tx.send(UiEvent::CharacterSaved(Ok(character))).await;
                ctx.request_repaint();
                let list = db.get_all_characters().await.map_err(|e| e.to_string());
                let _ = tx.send(UiEvent::CharactersLoaded(list)).await;
                ctx.request_repaint();
            }
        });
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
                let list = db.get_all_lorebooks().await.map_err(|e| e.to_string());
                let _ = tx.send(UiEvent::LorebooksLoaded(list)).await;
                ctx.request_repaint();
            }
        });
    }

    pub fn delete_character(&self, id: i64) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
             let res = db.delete_character(id).await;
             let _ = tx.send(UiEvent::CharacterDeleted(res.map(|_| id).map_err(|e| e.to_string()))).await;
             ctx.request_repaint();
        });
   }

   pub fn move_character(&self, char_id: i64, target_coll_id: Option<i64>) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
             let res = db.move_character(char_id, target_coll_id).await;
             let _ = tx.send(UiEvent::CharacterMoved(res.map(|_| (char_id, target_coll_id)).map_err(|e| e.to_string()))).await;
             ctx.request_repaint();
        });
   }

    pub fn save_collection(&mut self, id: i64, name: String, parent_id: Option<i64>) {
        self.is_saving = true;
        let tx = self.tx.clone();
        let db = self.db.clone();
        let col = crate::models::Collection { id, name, parent_id };
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let result = db.upsert_collection(&col).await.map_err(|e| e.to_string());
             let _ = tx.send(UiEvent::CollectionSaved(result)).await;
             ctx.request_repaint();
        });
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
        if char_id == 0 { return; }
        
        // Optimistic UI update
        if link {
            self.lore_links.insert(lore_id);
        } else {
            self.lore_links.remove(&lore_id);
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
            let _ = tx.send(UiEvent::LinkUpdated(res.map_err(|e| e.to_string()))).await;
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

    pub fn request_character_switch(&mut self, id: i64) {
        if self.has_unsaved_changes() {
            self.popup_state = PopupState::UnsavedChanges { target: AppAction::SwitchCharacter(id) };
        } else {
            self.load_character(id);
        }
    }

    pub fn request_collection_switch(&mut self, id: Option<i64>) {
        if self.has_unsaved_changes() {
            self.popup_state = PopupState::UnsavedChanges { target: AppAction::SwitchCollection(id) };
        } else {
            self.viewing_all_characters = false;
            self.selected_collection_id = id;
            self.mode = AppMode::Characters;
            self.central_view = CentralView::Browser;
            self.reload_collections();
        }
    }

    pub fn request_view_all(&mut self) {
        if self.has_unsaved_changes() {
            self.popup_state = PopupState::UnsavedChanges { target: AppAction::SwitchToAll };
        } else {
            self.viewing_all_characters = true;
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
                self.reload_collections();
            },
            AppAction::SwitchToAll => {
                self.viewing_all_characters = true;
                self.selected_collection_id = None;
                self.mode = AppMode::Characters;
                self.central_view = CentralView::Browser;
                self.selected_character = None;
                self.reload_characters();
            },
            AppAction::Exit => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }

    
    pub fn set_status(&mut self, msg: String, color: egui::Color32) {
        self.set_status_with_duration(msg, color, Duration::from_secs(3));
    }
    
    pub fn set_status_with_duration(&mut self, msg: String, color: egui::Color32, duration: Duration) {
        self.status_message = Some((msg, color));
        self.status_clear_time = Some(Instant::now() + duration);
    }
    
    pub fn add_tag(&self, char_id: i64, name: String, is_external: bool) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let res = db.add_tag_to_character(char_id, &name, is_external).await.map_err(|e| e.to_string());
            let _ = tx.send(UiEvent::TagOperationFinished(res)).await;
            ctx.request_repaint();
        });
    }

    pub fn remove_tag(&self, char_id: i64, tag_id: i64, is_external: bool) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let res = db.remove_tag_from_character(char_id, tag_id, is_external).await.map_err(|e| e.to_string());
            let _ = tx.send(UiEvent::TagOperationFinished(res)).await;
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
        
        if self.deep_search_query.trim().is_empty() { return; }
        
        self.is_deep_searching = true;
        self.mode = AppMode::DeepSearch;
        self.deep_search_results.clear();
        
        let query = self.deep_search_query.clone();
        let tx = self.tx.clone();
        let db = self.db.clone();
        
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let mut results = Vec::new();
            
            // 1. Search Characters Text
            let mut char_map: std::collections::HashMap<i64, Character> = std::collections::HashMap::new();
            
            if let Ok(chars) = db.search_characters_text(&query).await {
                for c in chars {
                    char_map.insert(c.id, c);
                }
            }
            
            // 2. Search Tags
            let mut tag_matches: Vec<(i64, String, bool)> = Vec::new();
            if let Ok(tags) = db.search_tags_matching(&query).await {
                tag_matches = tags;
            }
            
            // 3. Fetch missing characters found by tags
            let found_ids: std::collections::HashSet<i64> = tag_matches.iter().map(|(id, _, _)| *id).collect();
            let missing_ids: Vec<i64> = found_ids.into_iter().filter(|id| !char_map.contains_key(id)).collect();
            
            if !missing_ids.is_empty() {
                if let Ok(fetched) = db.get_characters_by_ids(&missing_ids).await {
                    for c in fetched {
                        char_map.insert(c.id, c);
                    }
                }
            }
            
            // 4. Build Character Results
            for (_, c) in char_map {
                let mut matches = Vec::new();
                
                // Use widget helper
                use crate::ui::widgets::extract_snippets;
                
                for s in extract_snippets(&c.personality, &query) { matches.push(("Personality".to_string(), s)); }
                for s in extract_snippets(&c.scenario, &query) { matches.push(("Scenario".to_string(), s)); }
                for s in extract_snippets(&c.example_dialogue, &query) { matches.push(("Example Dialogue".to_string(), s)); }
                for s in extract_snippets(&c.first_message, &query) { matches.push(("First Message".to_string(), s)); }
                for s in extract_snippets(&c.author_notes, &query) { matches.push(("Notes".to_string(), s)); }
                
                for (tid, tname, is_ext) in &tag_matches {
                    if *tid == c.id {
                         let label = if *is_ext { "Ext. Tag" } else { "App Tag" };
                         matches.push((label.to_string(), tname.clone()));
                    }
                }
                
                if !matches.is_empty() {
                    results.push(DeepSearchResult {
                        id: c.id,
                        kind: SearchResultKind::Character,
                        display_name: c.name,
                        matches,
                    });
                }
            }
            
            // Search Lorebooks
             use crate::ui::widgets::extract_snippets;
            if let Ok(books) = db.search_lorebooks_text(&query).await {
                for b in books {
                     let mut matches = Vec::new();
                     for s in extract_snippets(&b.description, &query) { matches.push(("Description".to_string(), s)); }
                     
                     if !matches.is_empty() {
                         results.push(DeepSearchResult {
                             id: b.id,
                             kind: SearchResultKind::Lorebook,
                             display_name: b.title,
                             matches,
                         });
                     }
                }
            }
            
            let _ = tx.send(UiEvent::DeepSearchCompleted(Ok(results))).await;
            ctx.request_repaint();
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
                    Ok(list) => { self.characters = list; self.loading_error = None; }
                    Err(e) => { eprintln!("Load error: {}", e); self.loading_error = Some(e); }
                },
                UiEvent::LorebooksLoaded(res) => {
                    match res {
                        Ok(books) => self.lorebooks = books,
                        Err(e) => { self.loading_error = Some(e); }
                    }
                },
                UiEvent::CollectionsLoaded(res) => {
                     match res {
                        Ok(collections) => self.collections = collections,
                        Err(e) => { self.loading_error = Some(e); }
                     }
                },
                UiEvent::LoreLinksLoaded(res) => {
                     match res {
                        Ok(set) => self.lore_links = set,
                        Err(e) => eprintln!("Link load error: {}", e),
                     }
                },
                UiEvent::CharacterSaved(res) => {
                    self.is_saving = false;
                    match res {
                        Ok(c) => {
                            self.selected_character = Some(c);
                            self.set_status("Character Saved!".to_string(), egui::Color32::GREEN);
                        },
                        Err(e) => self.set_status(format!("Save Error: {}", e), egui::Color32::RED),
                    }
                },
                UiEvent::LorebookSaved(res) => {
                    self.is_saving = false;
                    match res {
                        Ok(l) => {
                            self.selected_lorebook = Some(l);
                            self.set_status("Lorebook Saved!".to_string(), egui::Color32::GREEN);
                        },
                        Err(e) => self.set_status(format!("Save Error: {}", e), egui::Color32::RED),
                    }
                },
                UiEvent::CollectionSaved(res) => {
                    self.is_saving = false;
                    match res {
                        Ok(_) => { 
                            self.set_status("Collection Saved!".to_string(), egui::Color32::GREEN);
                            self.reload_collections();
                            self.popup_state = PopupState::None;
                        },
                        Err(e) => self.set_status(format!("Save Error: {}", e), egui::Color32::RED),
                    }
                },
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
                        },
                        Err(e) => self.set_status(format!("Delete Error: {}", e), egui::Color32::RED),
                    }
                },
                UiEvent::LinkUpdated(res) => {
                     if let Err(e) = res {
                          self.set_status(format!("Link Error: {}", e), egui::Color32::RED);
                     }
                },
                UiEvent::TagsLoaded(res) => {
                    match res {
                        Ok((id, app, ext)) => {
                            if let Some(c) = &mut self.selected_character {
                                if c.id == id {
                                    c.app_tags = app;
                                    c.external_tags = ext;
                                }
                            }
                        },
                        Err(e) => self.set_status(format!("Tag Load Error: {}", e), egui::Color32::RED),
                    }
                },
                UiEvent::TagOperationFinished(res) => {
                    match res {
                        Ok(_) => {
                            if let Some(c) = &self.selected_character {
                                self.load_tags(c.id);
                            }
                            self.refresh_all();
                        },
                        Err(e) => self.set_status(format!("Tag Error: {}", e), egui::Color32::RED),
                    }
                },
                UiEvent::DeepSearchCompleted(res) => {
                    self.is_deep_searching = false;
                    match res {
                        Ok(results) => self.deep_search_results = results,
                        Err(e) => self.set_status(format!("Search failed: {}", e), egui::Color32::RED),
                    }
                },
                UiEvent::UiRepaint => {
                     // Just wakes the loop, nothing to do
                },
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
                          },
                          Err(e) => self.set_status(format!("Delete Error: {}", e), egui::Color32::RED),
                     }
                },
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
                          },
                          Err(e) => self.set_status(format!("Move Error: {}", e), egui::Color32::RED),
                     }
                },
                UiEvent::ImportFileLoaded(res) => {
                    match res {
                         Ok(json_content) => {
                             if let Ok(mut char_obj) = serde_json::from_str::<Character>(&json_content) {
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
                                      external_tags: char_obj.external_tags.iter().map(|t| t.name.clone()).collect(),
                                  };
                                  
                                  // Force "New Character" mode
                                  self.selected_character = Some(Character::default()); 
                                  self.mode = AppMode::Characters;
                                  
                                  self.parsed_data = Some(parsed);
                                  self.show_import_modal = true;
                                  self.import_text.clear(); // Clear clipboard text if any
                                  
                                  self.set_status_with_duration("File loaded for review.".to_string(), egui::Color32::GREEN, Duration::from_secs(10));
                             } else {
                                 self.set_status("Failed to parse file structure.".to_string(), egui::Color32::RED);
                             }
                         },
                         Err(e) => self.set_status(format!("Read Error: {}", e), egui::Color32::RED),
                    }
                }
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
                self.popup_state = PopupState::UnsavedChanges { target: AppAction::Exit };
            }
        }

        // Side Panel
        side_panel::render_side_panel(self, ctx);

        // Central Panel
        central_panel::render_central_panel(self, ctx);
        
        // Global Popups
        if let PopupState::UnsavedChanges { target } = self.popup_state.clone() {
            egui::Window::new("Unsaved Changes")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label("You have unsaved changes.");
                    ui.label("What would you like to do?");
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("Save & Continue").clicked() {
                            if let Some(c) = self.selected_character.clone() {
                                self.save_character(c); 
                                // We can't immediately perform action because save is async.
                                // We need to defer the action until save completes?
                                // OR: Just save (async) and let user manually continue? 
                                // Better: Perform clean switch if save started? 
                                // Actually, async save means we don't know when it finishes here.
                                // Complex. Simplified: Just save. The user stays on page but sees "Saving...".
                                // Then they can click again? No, that's annoying.
                                // Workaround: We just trigger save, and close popup. 
                                // If they click exit again, it might still represent old state if not fast enough?
                                // "dirty" check will be cleared when `CharacterSaved` event returns.
                                // So we should just trigger save and close popup. The user will see save status.
                                // They have to click the action again after save.
                                // OR: We implement a "pending action" queue.
                            }
                            self.popup_state = PopupState::None;
                        }
                        
                        if ui.button("Discard Changes").clicked() {
                            // Revert changes
                            if let Some(selected) = &self.selected_character {
                                if selected.id != 0 {
                                    if let Some(original) = self.characters.iter().find(|c| c.id == selected.id) {
                                        self.selected_character = Some(original.clone());
                                    }
                                }
                                // If new character (id 0), discard means what? 
                                // If switching away, we just don't save. 
                            }
                            self.perform_action(target.clone(), ctx);
                            self.popup_state = PopupState::None;
                        }
                        
                        if ui.button("Cancel").clicked() {
                            self.popup_state = PopupState::None;
                        }
                    });
                });
        }

        if let PopupState::DeleteWarning { id: _, count } = self.popup_state {
            let mut close = false;
            egui::Window::new("Cannot Delete Folder")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                     ui.colored_label(egui::Color32::RED, "Warning: Folder is not empty.");
                     ui.add_space(5.0);
                     ui.label(format!("This folder contains {} character(s) or subfolder(s).", count));
                     ui.label("You must move or delete all contents before deleting this folder.");
                     ui.add_space(10.0);
                     
                     ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                         if ui.button("OK").clicked() {
                             close = true;
                         }
                     });
                });
            if close {
                self.popup_state = PopupState::None;
            }
        }

        if let PopupState::DeleteCharacterConfirmation { id, name } = self.popup_state.clone() {
             let mut close = false;
             egui::Window::new("Delete Character?")
                 .collapsible(false)
                 .resizable(false)
                 .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                 .show(ctx, |ui| {
                      ui.label(format!("Are you sure you want to delete '{}'?", name));
                      ui.colored_label(egui::Color32::RED, "This action cannot be undone.");
                      ui.add_space(10.0);
                      
                      ui.horizontal(|ui| {
                          if ui.button("Yes, Delete").clicked() {
                              self.delete_character(id);
                              close = true;
                          }
                          if ui.button("Cancel").clicked() {
                              close = true;
                          }
                      });
                 });
             if close {
                 self.popup_state = PopupState::None;
             }
        }

        let mut rename_request = None;
        if let PopupState::Renaming { id, name } = &mut self.popup_state {
            let coll_id = *id;
            let mut close = false;
            egui::Window::new("Rename Folder")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label("Enter new folder name:");
                    ui.text_edit_singleline(name);
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            rename_request = Some((coll_id, name.clone()));
                            close = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });
            if close {
                self.popup_state = PopupState::None;
            }
        }

        if let Some((id, new_name)) = rename_request {
            let parent_id = self.collections.iter().find(|c| c.id == id).and_then(|c| c.parent_id);
            self.save_collection(id, new_name, parent_id);
        }
    }
}
