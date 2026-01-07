use eframe::egui;
use tokio::sync::mpsc;
use std::time::{Duration, Instant};
use std::collections::HashSet;
use crate::db::Database;
use crate::models::{Character, Lorebook, Collection};

pub enum UiEvent {
    CharactersLoaded(Result<Vec<Character>, String>),
    LorebooksLoaded(Result<Vec<Lorebook>, String>),
    CollectionsLoaded(Result<Vec<Collection>, String>),
    LoreLinksLoaded(Result<HashSet<i64>, String>),
    CharacterSaved(Result<Character, String>),
    LorebookSaved(Result<Lorebook, String>),
    CollectionSaved(Result<i64, String>),
    CollectionDeleted(Result<i64, String>), // Returns ID of deleted collection
    LinkUpdated(Result<(), String>),
}

#[derive(PartialEq)]
enum AppMode {
    Characters,
    Lorebooks,
}

#[derive(PartialEq)]
enum CharacterTab {
    MainData,
    AuthorNotes,
    AssociatedLore,
}

#[derive(Clone, PartialEq)]
enum PopupState {
    None,
    Renaming { id: i64, name: String },
    DeleteConfirmation { id: i64 },
    DeleteWarning { id: i64, count: usize },
}

#[derive(PartialEq, Clone, Copy)]
pub enum SortMode {
    Alphabetical,
    NewestFirst,
    RecentlyUpdated,
}

pub struct CrapApp {
    db: Database,
    tx: mpsc::Sender<UiEvent>,
    rx: mpsc::Receiver<UiEvent>,
    
    // Data
    characters: Vec<Character>,
    lorebooks: Vec<Lorebook>,
    collections: Vec<Collection>,
    lore_links: HashSet<i64>, // IDs of lorebooks linked to selected_character

    // State
    mode: AppMode,
    selected_character: Option<Character>,
    selected_lorebook: Option<Lorebook>,
    active_char_tab: CharacterTab,
    sort_mode: SortMode,
    selected_collection_id: Option<i64>, // For "New Folder" context
    popup_state: PopupState,
    
    // Feedback
    is_saving: bool,
    status_message: Option<(String, egui::Color32)>,
    status_clear_time: Option<Instant>,
    loading_error: Option<String>,
    
    // Widgets
    search_query: String,
}

impl CrapApp {
    pub fn new(cc: &eframe::CreationContext<'_>, db: Database) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let (tx, rx) = mpsc::channel(20);
        
        let app = Self { 
            db,
            tx,
            rx,
            characters: Vec::new(),
            lorebooks: Vec::new(),
            collections: Vec::new(),
            lore_links: HashSet::new(),
            mode: AppMode::Characters,
            selected_character: None,
            selected_lorebook: None,
            active_char_tab: CharacterTab::MainData,
            sort_mode: SortMode::Alphabetical,
            selected_collection_id: None,
            popup_state: PopupState::None,
            is_saving: false,
            status_message: None,
            status_clear_time: None,
            loading_error: None,
            search_query: String::new(),
        };
        
        app.refresh_all();
        app
    }

    fn refresh_all(&self) {
        self.reload_characters();
        self.reload_lorebooks();
        self.reload_collections();
    }

    fn reload_characters(&self) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        tokio::spawn(async move {
            let result = db.get_all_characters().await.map_err(|e| e.to_string());
            let _ = tx.send(UiEvent::CharactersLoaded(result)).await;
        });
    }

    fn reload_lorebooks(&self) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        tokio::spawn(async move {
            let result = db.get_all_lorebooks().await.map_err(|e| e.to_string());
            let _ = tx.send(UiEvent::LorebooksLoaded(result)).await;
        });
    }

    fn load_links(&self, char_id: i64) {
        if char_id == 0 { return; } 
        let tx = self.tx.clone();
        let db = self.db.clone();
        tokio::spawn(async move {
            let result = db.get_lore_links(char_id).await.map_err(|e| e.to_string());
            let _ = tx.send(UiEvent::LoreLinksLoaded(result)).await;
        });
    }

    fn reload_collections(&self) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        tokio::spawn(async move {
            let result = db.get_all_collections().await.map_err(|e| e.to_string());
            let _ = tx.send(UiEvent::CollectionsLoaded(result)).await;
        });
    }

    fn save_character(&mut self, mut character: Character) {
        self.is_saving = true;
        self.status_message = None;
        let tx = self.tx.clone();
        let db = self.db.clone();
        tokio::spawn(async move {
            if let Err(e) = db.upsert_character(&mut character).await {
                let _ = tx.send(UiEvent::CharacterSaved(Err(e.to_string()))).await;
            } else {
                let _ = tx.send(UiEvent::CharacterSaved(Ok(character))).await;
                // Reload list to reflect name changes
                let list = db.get_all_characters().await.map_err(|e| e.to_string());
                let _ = tx.send(UiEvent::CharactersLoaded(list)).await;
            }
        });
    }

    fn save_lorebook(&mut self, mut lorebook: Lorebook) {
        self.is_saving = true;
        self.status_message = None;
        let tx = self.tx.clone();
        let db = self.db.clone();
        tokio::spawn(async move {
            if let Err(e) = db.upsert_lorebook(&mut lorebook).await {
                let _ = tx.send(UiEvent::LorebookSaved(Err(e.to_string()))).await;
            } else {
                let _ = tx.send(UiEvent::LorebookSaved(Ok(lorebook))).await;
                // Reload list
                let list = db.get_all_lorebooks().await.map_err(|e| e.to_string());
                let _ = tx.send(UiEvent::LorebooksLoaded(list)).await;
            }
        });
    }

    fn save_collection(&mut self, name: String, parent_id: Option<i64>) {
        self.is_saving = true;
        let tx = self.tx.clone();
        let db = self.db.clone();
        let col = crate::models::Collection { id: 0, name, parent_id }; // ID 0 means insert
        tokio::spawn(async move {
            let result = db.upsert_collection(&col).await.map_err(|e| e.to_string());
             let _ = tx.send(UiEvent::CollectionSaved(result)).await;
        });
    }
    
    fn get_collection_path(&self, mut col_id: i64) -> String {
        let mut path = Vec::new();
        // Simple loop to prevent infinite recursion if cycle exists (though DB shouldn't allow it easily without checking)
        // We limit depth to 10 for safety.
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

    fn toggle_lore_link(&mut self, char_id: i64, lore_id: i64, link: bool) {
        if link {
            self.lore_links.insert(lore_id);
        } else {
            self.lore_links.remove(&lore_id);
        }

        let tx = self.tx.clone();
        let db = self.db.clone();
        tokio::spawn(async move {
            let res = if link {
                db.link_lore(char_id, lore_id).await
            } else {
                db.unlink_lore(char_id, lore_id).await
            };
            let _ = tx.send(UiEvent::LinkUpdated(res.map_err(|e| e.to_string()))).await;
        });
    }
}

enum TreeAction {
    SelectChar(Character),
    SelectCollection(i64),
    DeselectCollection,
    RenameCollection(i64, String),
    RequestDeleteCollection(i64),
    CreateSubfolder(i64),
    CreateRootFolder,
}

fn render_tree(
    ui: &mut egui::Ui,
    collections: &[Collection],
    characters: &[Character],
    parent_id: Option<i64>,
    selected_char_id: Option<i64>,
    selected_coll_id: Option<i64>,
    actions: &mut Vec<TreeAction>,
    sort_mode: SortMode,
    search_query: &str,
) {
    let query_lower = search_query.to_lowercase();
    let is_search_active = !search_query.is_empty();

    // 1. Render Sub-collections
    let node_colls: Vec<&Collection> = collections.iter().filter(|c| c.parent_id == parent_id).collect();
    for col in node_colls {
        // Pre-calculate if this folder (or subfolders) has any matching chars
        // Start simple recursive check if searching? Or let valid items show?
        // User asked: "Jeśli folder jest pusty po przefiltrowaniu, lekko go wyszarz."
        // We can check if `has_matches` locally, but we need deep check.
        // Actually, render_tree calls itself.
        // Standard approach: Render it, but maybe disable/dim if no children visible?
        // Let's implement the dimming:
        // Helper to count visible descendants matching query
        let has_visible_descendants = if is_search_active {
             has_matches(col.id, collections, characters, &query_lower)
        } else {
             true
        };
        
        // Render
        let is_selected = Some(col.id) == selected_coll_id;
        let id_str = ui.make_persistent_id(col.id);
        
        let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id_str, false);
        let was_open = state.is_open();
        
        // If searching and has matches, force expand? User didn't ask, but good UX. 
        // User asked for "grey out if empty after filter".
        if is_search_active && has_visible_descendants {
             state.set_open(true);
        }

        let mut toggle = false;
        let header_res = state.show_header(ui, |ui| {
             let alpha = if has_visible_descendants { 255 } else { 100 };
             let _color = if is_selected { 
                 ui.visuals().selection.bg_fill
             } else {
                 egui::Color32::from_gray(200).linear_multiply(alpha as f32 / 255.0)
             };
             
             let text_color = if is_selected { egui::Color32::WHITE } else {
                  ui.visuals().text_color().linear_multiply(alpha as f32 / 255.0)
             };

             let label = egui::RichText::new(format!("📁 {}", col.name)).strong().color(text_color);
             
             let mut response = ui.selectable_label(is_selected, label);
             
             // Tooltip explaining empty
             if !has_visible_descendants {
                 response = response.on_hover_text("No matching characters in this folder");
             }
             
             if response.clicked() {
                 actions.push(TreeAction::SelectCollection(col.id));
                 toggle = true;
             }
             
             response.context_menu(|ui| {
                 if ui.button("Rename").clicked() {
                     actions.push(TreeAction::RenameCollection(col.id, col.name.clone()));
                     ui.close_menu();
                 }
                 if ui.button("New Subfolder").clicked() {
                     actions.push(TreeAction::CreateSubfolder(col.id));
                     ui.close_menu();
                 }
                 ui.separator();
                 if ui.button("Delete").clicked() {
                     actions.push(TreeAction::RequestDeleteCollection(col.id));
                     ui.close_menu();
                 }
             });
        });
        
        header_res.body(|ui| {
            render_tree(ui, collections, characters, Some(col.id), selected_char_id, selected_coll_id, actions, sort_mode, search_query);
        });

        if toggle {
             // Manual toggle triggered from inside header
             // Verify if auto-toggle already happened by checking if state changed
             if let Some(mut state) = egui::collapsing_header::CollapsingState::load(ui.ctx(), id_str) {
                 let is_open_now = state.is_open();
                 if was_open == is_open_now {
                      state.toggle(ui);
                      state.store(ui.ctx());
                      ui.ctx().request_repaint();
                 }
             }
        }
    }

    // 2. Render Characters
    let mut node_chars: Vec<&Character> = characters.iter().filter(|c| c.collection_id == parent_id).collect();
    
    // Filter
    if is_search_active {
         node_chars.retain(|c| {
             c.name.to_lowercase().contains(&query_lower) || c.char_title.to_lowercase().contains(&query_lower)
         });
    }

    // Sort
    match sort_mode {
        SortMode::Alphabetical => node_chars.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
        SortMode::NewestFirst => node_chars.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
        SortMode::RecentlyUpdated => node_chars.sort_by(|a, b| b.updated_at.cmp(&a.updated_at)),
    }

    for char in node_chars {
        let is_selected = Some(char.id) == selected_char_id;
        
        let item_height = 48.0; // 40px thumb + padding
        let (rect, response) = ui.allocate_exact_size(
             egui::vec2(ui.available_width(), item_height), 
             egui::Sense::click()
        );

        // Interaction
        if response.clicked() {
             actions.push(TreeAction::SelectChar(char.clone()));
        }
        
        // Hover/Select background
        if is_selected {
             ui.painter().rect_filled(rect, 4.0, ui.visuals().selection.bg_fill);
        } else if response.hovered() {
             ui.painter().rect_filled(rect, 4.0, ui.visuals().widgets.hovered.bg_fill);
        }

        // Content
        let thumb_size = 40.0;
        let thumb_rect = egui::Rect::from_min_size(rect.min + egui::vec2(4.0, 4.0), egui::vec2(thumb_size, thumb_size));
        
        // Thumbnail
        if let Some(path_str) = &char.avatar_path {
             // Simple check if it looks like a path.
             // We use file:// prefix for local files.
             let uri = if path_str.contains("://") { 
                 path_str.clone() 
             } else {
                 if let Ok(abs_path) = std::fs::canonicalize(path_str) {
                      format!("file://{}", abs_path.to_string_lossy())
                 } else {
                      path_str.clone() // Fallback
                 }
             };
             
             egui::Image::new(uri)
                 .rounding(4.0)
                 .paint_at(ui, thumb_rect);
        } else {
             ui.painter().rect_filled(thumb_rect, 4.0, egui::Color32::from_gray(70));
             let initial = char.name.chars().next().unwrap_or('?').to_uppercase().to_string();
             ui.painter().text(
                 thumb_rect.center(), 
                 egui::Align2::CENTER_CENTER, 
                 initial, 
                 egui::FontId::proportional(20.0), 
                 egui::Color32::WHITE
             );
        }
        
        // Text
        let text_left = thumb_rect.max.x + 8.0;
        let _text_width = rect.width() - (text_left - rect.min.x) - 4.0; // Available width for text
        
        // Name
        let name_font = egui::FontId::proportional(15.0); // Strong/Header-ish
        let name_color = if is_selected { egui::Color32::WHITE } else { ui.visuals().text_color() };
        
        // We use layout_job for potentially better control, or just layout_no_wrap
        // Let's use layout with wrapping disabled but strictly checking width if we wanted truncation.
        // For simplicity: layout_no_wrap. To clip, we rely on painter clip or just let it overflow slightly if huge?
        // Better: clip to rect.
        let name_galley = ui.painter().layout_no_wrap(char.name.clone(), name_font, name_color);
        let name_pos = egui::pos2(text_left, rect.min.y + 4.0);
        
        ui.painter().with_clip_rect(rect).galley(name_pos, name_galley, egui::Color32::WHITE);
        
        // Title
        if !char.char_title.is_empty() {
             let title_font = egui::FontId::proportional(12.0);
             let title_color = if is_selected { 
                 egui::Color32::from_white_alpha(200) 
             } else { 
                 ui.visuals().text_color().linear_multiply(0.7) 
             };
             
             let title_galley = ui.painter().layout_no_wrap(char.char_title.clone(), title_font, title_color);
             let title_pos = egui::pos2(text_left, rect.min.y + 24.0); // Below name
             
             ui.painter().with_clip_rect(rect).galley(title_pos, title_galley, egui::Color32::WHITE);
        }
    }
}

// Helper to check for matches recursively
fn has_matches(collection_id: i64, collections: &[Collection], characters: &[Character], query: &str) -> bool {
    // 1. Check characters in this collection
    if characters.iter().any(|c| c.collection_id == Some(collection_id) && (c.name.to_lowercase().contains(query) || c.char_title.to_lowercase().contains(query))) {
        return true;
    }
    
    // 2. Check sub-collections
    let sub_colls: Vec<&Collection> = collections.iter().filter(|c| c.parent_id == Some(collection_id)).collect();
    for sub in sub_colls {
        if has_matches(sub.id, collections, characters, query) {
            return true;
        }
    }
    
    false
}
impl CrapApp {
    fn set_status(&mut self, msg: String, color: egui::Color32) {
        self.status_message = Some((msg, color));
        self.status_clear_time = Some(Instant::now() + Duration::from_secs(3));
    }
}

impl eframe::App for CrapApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Event Loop
        while let Ok(event) = self.rx.try_recv() {
            match event {
                UiEvent::CharactersLoaded(res) => match res {
                    Ok(list) => { self.characters = list; self.loading_error = None; }
                    Err(e) => { eprintln!("Load error: {}", e); self.loading_error = Some(e); }
                },
                UiEvent::LorebooksLoaded(res) => {
                    match res {
                        Ok(books) => self.lorebooks = books,
                        Err(e) => {
                            self.loading_error = Some(e);
                        }
                    }
                },
                UiEvent::CollectionsLoaded(res) => {
                    match res {
                        Ok(collections) => self.collections = collections,
                        Err(e) => {
                             self.loading_error = Some(e);
                        }
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
                            self.reload_collections(); // Refresh tree
                            self.popup_state = PopupState::None; // Close rename popup if open
                        },
                        Err(e) => self.set_status(format!("Save Error: {}", e), egui::Color32::RED),
                    }
                },
                UiEvent::CollectionDeleted(res) => {
                    self.is_saving = false;
                     match res {
                        Ok(id) => { 
                            self.set_status("Collection Deleted".to_string(), egui::Color32::GREEN);
                            self.reload_collections(); // Refresh tree
                            self.reload_characters(); // Refresh chars (orphans)
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
            }
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

        egui::SidePanel::left("side_panel")
            .min_width(250.0) // Increased width
            .show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.mode, AppMode::Characters, "Characters");
                ui.selectable_value(&mut self.mode, AppMode::Lorebooks, "Lorebooks");
            });
            ui.separator();

             if let Some(err) = &self.loading_error {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
                if ui.button("Retry").clicked() { self.refresh_all(); }
            } else {
                 // Sorting Toolbar (only for Characters)
                 if self.mode == AppMode::Characters {
                      // Search Bar
                      ui.horizontal(|ui| {
                          ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                              ui.add(egui::TextEdit::singleline(&mut self.search_query).hint_text("Search characters..."));
                              if !self.search_query.is_empty() && ui.button("x").clicked() {
                                  self.search_query.clear();
                              }
                          });
                      });
                      ui.separator();
                      
                      ui.horizontal(|ui| {
                         ui.label("Sort:");
                         ui.selectable_value(&mut self.sort_mode, SortMode::Alphabetical, "A-Z");
                         ui.selectable_value(&mut self.sort_mode, SortMode::NewestFirst, "New");
                         ui.selectable_value(&mut self.sort_mode, SortMode::RecentlyUpdated, "Last");
                     });
                     ui.separator();
                     
                     // Toolbar for folders
                     ui.horizontal(|ui| {
                         let label = if self.selected_collection_id.is_some() { "New Subfolder" } else { "New Folder" };
                         if ui.button(label).clicked() {
                             // Context-aware creation
                             if let Some(pid) = self.selected_collection_id {
                                 self.save_collection("New Folder".to_string(), Some(pid));
                             } else {
                                 self.save_collection("New Folder".to_string(), None);
                             }
                         }
                     });
                     ui.separator();
                 }

                 egui::ScrollArea::vertical().show(ui, |ui| {
                     ui.vertical(|ui| {
                         match self.mode {
                             AppMode::Characters => {
                                 let mut actions = Vec::new();
                                 let sel_char = self.selected_character.as_ref().map(|c| c.id);
                                 let sel_col = self.selected_collection_id;
                                 
                                 // Render "Uncategorized" explicit section or just root characters?
                                 // Plan said: "Top: Uncategorized... Standard: Recursive..."
                                 // `render_tree` with parent_id: None handles root items (which are uncategorized/root).
                                 // If we want a specific "Uncategorized" folder visual, we can fake it, but passing None to render_tree mimics standard file system root.
                                 // Let's stick to calling render_tree with None.
                                 
                                 let root_label = if sel_col.is_none() {
                                     egui::RichText::new("📁 Root").strong()
                                 } else {
                                     egui::RichText::new("📁 Root")
                                 };

                                 let root_response = ui.selectable_label(sel_col.is_none(), root_label);
                                 if root_response.clicked() {
                                      actions.push(TreeAction::DeselectCollection);
                                 }
                                 
                                 root_response.context_menu(|ui| {
                                     if ui.button("New Folder").clicked() {
                                         actions.push(TreeAction::CreateRootFolder);
                                         ui.close_menu();
                                     }
                                 });
                                 
                                 render_tree(
                                     ui, 
                                     &self.collections, 
                                     &self.characters, 
                                     None, 
                                     sel_char, 
                                     sel_col, 
                                     &mut actions,
                                     self.sort_mode,
                                     &self.search_query,
                                 );
                                 
                                 // Process actions
                                 for action in actions {
                                     match action {
                                         TreeAction::SelectChar(c) => {
                                             self.selected_collection_id = c.collection_id; // Auto-select parent folder
                                             self.selected_character = Some(c.clone());
                                             self.active_char_tab = CharacterTab::MainData;
                                             self.status_message = None;
                                             self.load_links(c.id);
                                         },
                                         TreeAction::SelectCollection(id) => {
                                             self.selected_collection_id = Some(id);
                                         },
                                         TreeAction::DeselectCollection => {
                                             self.selected_collection_id = None;
                                         },
                                         TreeAction::RenameCollection(id, name) => {
                                             self.popup_state = PopupState::Renaming { id, name };
                                         },
                                         TreeAction::CreateSubfolder(parent_id) => {
                                             self.save_collection("New Folder".to_string(), Some(parent_id));
                                             // Auto-expand? We don't track expansion state explicitly here (egui does internally), 
                                             // but creating it will likely show up.
                                         },
                                         TreeAction::CreateRootFolder => {
                                             self.save_collection("New Folder".to_string(), None);
                                         },
                                         TreeAction::RequestDeleteCollection(id) => {
                                             self.popup_state = PopupState::DeleteConfirmation { id };
                                         }
                                     }
                                 }
                             },
                             AppMode::Lorebooks => {
                                 for lore in &self.lorebooks {
                                     if ui.button(&lore.title).clicked() {
                                         self.selected_lorebook = Some(lore.clone());
                                         self.status_message = None;
                                     }
                                 }
                             }
                         }
                     });
                 });
             }
             
             ui.add_space(10.0);
             if ui.button("+ Add New").clicked() {
                 match self.mode {
                     AppMode::Characters => {
                         self.selected_character = Some(Character::default());
                         self.lore_links.clear();
                     },
                     AppMode::Lorebooks => self.selected_lorebook = Some(Lorebook::default()),
                 }
                 self.status_message = None;
             }
        });

        // Prepare collection options to avoid borrow checker issues inside the closure where we mutate char.
        let collection_options: Vec<(i64, String)> = self.collections.iter().map(|c| {
            (c.id, self.get_collection_path(c.id))
        }).collect();

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.mode {
                AppMode::Characters => {
                    let mut save_req = None;
                    let mut toggle_requests = Vec::new();
                    
                    if let Some(character) = &mut self.selected_character {
                        ui.horizontal(|ui| {
                            ui.heading("Edit Character");
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Collection:");
                            let current_col_name = character.collection_id.and_then(|id| {
                                collection_options.iter().find(|(cid, _)| *cid == id).map(|(_, name)| name.clone())
                            }).unwrap_or_else(|| "Uncategorized".to_string());
                            
                            egui::ComboBox::from_id_source("collection_combo")
                                .selected_text(current_col_name)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut character.collection_id, None, "Uncategorized");
                                    for (id, name) in &collection_options {
                                        ui.selectable_value(&mut character.collection_id, Some(*id), name);
                                    }
                                });
                        });
                        
                        ui.separator();
                        
                        // Tabs
                        ui.horizontal(|ui| {
                            ui.selectable_value(&mut self.active_char_tab, CharacterTab::MainData, "Main Data");
                            ui.selectable_value(&mut self.active_char_tab, CharacterTab::AuthorNotes, "Author Notes");
                            ui.selectable_value(&mut self.active_char_tab, CharacterTab::AssociatedLore, "Associated Lore");
                        });
                        ui.separator();

                        let inner = egui::ScrollArea::vertical().show(ui, |ui| {
                            match self.active_char_tab {
                                CharacterTab::MainData => {
                                    ui.label("Name (File Name)");
                                    ui.text_edit_singleline(&mut character.name);
                                    ui.label("Character Name");
                                    ui.text_edit_singleline(&mut character.char_name);
                                    ui.label("Title");
                                    ui.text_edit_singleline(&mut character.char_title);
                                    
                                    ui.label("Personality");
                                    if ui.text_edit_multiline(&mut character.personality).changed() {}
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                        ui.label(format!("Tokens: {}", crate::models::count_tokens(&character.personality)));
                                    });

                                    ui.label("Scenario");
                                    if ui.text_edit_multiline(&mut character.scenario).changed() {}
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                        ui.label(format!("Tokens: {}", crate::models::count_tokens(&character.scenario)));
                                    });

                                    ui.label("Example Dialogue");
                                    if ui.text_edit_multiline(&mut character.example_dialogue).changed() {}
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                        ui.label(format!("Tokens: {}", crate::models::count_tokens(&character.example_dialogue)));
                                    });

                                    ui.label("First Message");
                                    if ui.text_edit_multiline(&mut character.first_message).changed() {}
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                        ui.label(format!("Tokens: {}", crate::models::count_tokens(&character.first_message)));
                                    });

                                    ui.label("Avatar");
                                    if let Some(path_str) = &character.avatar_path {
                                         if let Ok(abs_path) = std::fs::canonicalize(path_str) {
                                             let uri = format!("file://{}", abs_path.to_string_lossy());
                                             ui.add(egui::Image::new(uri).max_width(200.0).max_height(200.0));
                                         } else {
                                             ui.label("Image not found");
                                         }
                                    }
                                    if ui.button("Browse Avatar").clicked() {
                                        if let Some(path) = rfd::FileDialog::new().add_filter("image", &["png", "jpg", "jpeg"]).pick_file() {
                                            let dest_dir = std::path::Path::new("data/avatars");
                                            let _ = std::fs::create_dir_all(dest_dir);
                                            if let Some(name) = path.file_name() {
                                                let dest = dest_dir.join(name);
                                                let _ = std::fs::copy(&path, &dest);
                                                character.avatar_path = Some(dest.to_string_lossy().to_string());
                                            }
                                        }
                                    }
                                },
                                CharacterTab::AuthorNotes => {
                                    ui.label("Author Notes");
                                    ui.text_edit_multiline(&mut character.author_notes);
                                },
                                CharacterTab::AssociatedLore => {
                                    ui.label("Select relevant lorebooks:");
                                    // Iterate lorebooks (immutably borrowed via self)
                                    // toggle_requests will store actions to perform later
                                    for lore in &self.lorebooks {
                                        let mut is_linked = self.lore_links.contains(&lore.id);
                                        if ui.checkbox(&mut is_linked, &lore.title).clicked() {
                                            if character.id != 0 {
                                                toggle_requests.push((character.id, lore.id, is_linked));
                                            }
                                        }
                                    }
                                    if character.id == 0 {
                                        ui.colored_label(egui::Color32::YELLOW, "Save character to enable linking.");
                                    }
                                }
                            }
                            
                            ui.add_space(20.0);
                            ui.horizontal(|ui| {
                                if self.is_saving {
                                    ui.spinner();
                                    ui.label("Saving...");
                                } else {
                                    if ui.button("Save Character").clicked() {
                                        return Some(character.clone());
                                    }
                                    if let Some((msg, color)) = &self.status_message {
                                        ui.colored_label(*color, msg);
                                    }
                                }
                                None
                            }).inner
                        }).inner;
                        save_req = inner;

                    } else {
                        ui.label("Select a character");
                    }

                    // Process save request
                    if let Some(c) = save_req {
                        self.save_character(c);
                    }
                    
                    // Process toggle requests (mutable self borrow here is fine now)
                    for (char_id, lore_id, link) in toggle_requests {
                        self.toggle_lore_link(char_id, lore_id, link);
                    }
                    

                },
                AppMode::Lorebooks => {
                    let mut save_req = None;
                    if let Some(lore) = &mut self.selected_lorebook {
                        ui.heading("Edit Lorebook");
                        ui.separator();
                        let inner = egui::ScrollArea::vertical().show(ui, |ui| {
                             ui.label("Title");
                             ui.text_edit_singleline(&mut lore.title);
                             
                             ui.label("Description");
                             ui.text_edit_multiline(&mut lore.description);
                             
                             ui.label("Cover Image");
                             if let Some(path_str) = &lore.cover_path {
                                  if let Ok(abs_path) = std::fs::canonicalize(path_str) {
                                      let uri = format!("file://{}", abs_path.to_string_lossy());
                                      ui.add(egui::Image::new(uri).max_width(200.0).max_height(200.0));
                                  } else {
                                      ui.label(format!("Image not found at: {}", path_str));
                                  }
                             }

                             if ui.button("Browse Cover").clicked() {
                                 if let Some(path) = rfd::FileDialog::new().add_filter("image", &["png", "jpg", "jpeg"]).pick_file() {
                                     let dest_dir = std::path::Path::new("data/covers");
                                     if let Err(e) = std::fs::create_dir_all(dest_dir) {
                                         eprintln!("Failed to create covers directory: {}", e);
                                     } else {
                                         if let Some(file_name) = path.file_name() {
                                             let dest_path = dest_dir.join(file_name);
                                             match std::fs::copy(&path, &dest_path) {
                                                 Ok(_) => {
                                                     lore.cover_path = Some(dest_path.to_string_lossy().to_string());
                                                 }
                                                 Err(e) => eprintln!("Failed to copy cover: {}", e),
                                             }
                                         }
                                     }
                                 }
                             }


                             
                             ui.add_space(20.0);
                             ui.horizontal(|ui| {
                                if self.is_saving {
                                    ui.spinner();
                                    ui.label("Saving...");
                                } else {
                                    if ui.button("Save Lorebook").clicked() {
                                        return Some(lore.clone());
                                    }
                                    if let Some((msg, color)) = &self.status_message {
                                        ui.colored_label(*color, msg);
                                    }
                                }
                                None
                            }).inner
                        }).inner;
                        save_req = inner;
                    } else {
                        ui.label("Select a lorebook");
                    }
                    if let Some(l) = save_req {
                        self.save_lorebook(l);
                    }
                }
            }
        });
        
        // Modals
        let mut close_popup = false;
        let mut save_rename = None;
        let mut confirm_delete = None;
        let mut check_delete_contents = None;

        match &mut self.popup_state {
            PopupState::None => {},
            PopupState::Renaming { id, name } => {
                egui::Window::new("Rename Collection")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .show(ctx, |ui| {
                        ui.label("Enter new name:");
                        ui.text_edit_singleline(name);
                        ui.horizontal(|ui| {
                            if ui.button("Cancel").clicked() {
                                close_popup = true;
                            }
                            if ui.button("Rename").clicked() {
                                save_rename = Some((*id, name.clone()));
                                close_popup = true;
                            }
                        });
                    });
            },
            PopupState::DeleteConfirmation { id } => {
                 egui::Window::new("Delete Collection")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .show(ctx, |ui| {
                        ui.label("Are you sure you want to delete this folder?");
                        ui.horizontal(|ui| {
                            if ui.button("Cancel").clicked() {
                                close_popup = true;
                            }
                            if ui.button("Delete").clicked() {
                                check_delete_contents = Some(*id);
                                close_popup = true; // We might transition, not close to None, logic below
                            }
                        });
                    });
            },
            PopupState::DeleteWarning { id, count } => {
                 egui::Window::new("Delete Non-Empty Folder")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .show(ctx, |ui| {
                        ui.colored_label(egui::Color32::RED, format!("Warning: This folder contains {} items (subfolders/characters).", count));
                        ui.label("Deleting it will ORPHAN these items (they will move to Root).");
                        ui.label("Are you REALLY sure?");
                        ui.horizontal(|ui| {
                            if ui.button("Cancel").clicked() {
                                close_popup = true;
                            }
                            if ui.button("Yes, Delete Everything").clicked() {
                                confirm_delete = Some(*id);
                                close_popup = true;
                            }
                        });
                    });
            }
        }

        if close_popup {
            // Only reset if we are NOT transitioning to another state
            if check_delete_contents.is_none() {
                 self.popup_state = PopupState::None;
            }
        }

        if let Some((id, name)) = save_rename {
            // Reuse upsert logic, need parent_id. Find existing.
            let parent_id = self.collections.iter().find(|c| c.id == id).and_then(|c| c.parent_id);
            let col = Collection { id, name, parent_id };
            let tx = self.tx.clone();
            let db = self.db.clone();
            self.is_saving = true;
            tokio::spawn(async move {
                 let result = db.upsert_collection(&col).await.map_err(|e| e.to_string());
                 let _ = tx.send(UiEvent::CollectionSaved(result)).await;
            });
        }
        
        if let Some(id) = check_delete_contents {
            // Count children locally
            let child_colls = self.collections.iter().filter(|c| c.parent_id == Some(id)).count();
            let child_chars = self.characters.iter().filter(|c| c.collection_id == Some(id)).count();
            let total = child_colls + child_chars;
            
            if total > 0 {
                self.popup_state = PopupState::DeleteWarning { id, count: total };
            } else {
                confirm_delete = Some(id);
                self.popup_state = PopupState::None;
            }
        }
        
        if let Some(id) = confirm_delete {
             let tx = self.tx.clone();
             let db = self.db.clone();
             self.is_saving = true;
             tokio::spawn(async move {
                 let result = db.delete_collection(id).await.map(|_| id).map_err(|e| e.to_string());
                 let _ = tx.send(UiEvent::CollectionDeleted(result)).await;
             });
        }
    }
}
