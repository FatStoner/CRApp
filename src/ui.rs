use eframe::egui;
use tokio::sync::mpsc;
use std::time::{Duration, Instant};
use std::collections::HashSet;
use crate::db::Database;
use crate::models::{Character, Lorebook};

pub enum UiEvent {
    CharactersLoaded(Result<Vec<Character>, String>),
    LorebooksLoaded(Result<Vec<Lorebook>, String>),
    LoreLinksLoaded(Result<HashSet<i64>, String>),
    CharacterSaved(Result<Character, String>),
    LorebookSaved(Result<Lorebook, String>),
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
    lore_links: HashSet<i64>, // IDs of lorebooks linked to selected_character

    // State
    mode: AppMode,
    selected_character: Option<Character>,
    selected_lorebook: Option<Lorebook>,
    active_char_tab: CharacterTab,
    sort_mode: SortMode,
    
    // Feedback
    is_saving: bool,
    status_message: Option<(String, egui::Color32)>,
    status_clear_time: Option<Instant>,
    loading_error: Option<String>,
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
            lore_links: HashSet::new(),
            mode: AppMode::Characters,
            selected_character: None,
            selected_lorebook: None,
            active_char_tab: CharacterTab::MainData,
            sort_mode: SortMode::Alphabetical,
            is_saving: false,
            status_message: None,
            status_clear_time: None,
            loading_error: None,
        };
        
        app.refresh_all();
        app
    }

    fn refresh_all(&self) {
        self.reload_characters();
        self.reload_lorebooks();
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
                UiEvent::LorebooksLoaded(res) => match res {
                    Ok(list) => self.lorebooks = list,
                    Err(e) => eprintln!("Lore load error: {}", e),
                },
                UiEvent::LoreLinksLoaded(res) => match res {
                    Ok(set) => self.lore_links = set,
                    Err(e) => eprintln!("Link load error: {}", e),
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

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
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
                     ui.horizontal(|ui| {
                         ui.label("Sort:");
                         ui.selectable_value(&mut self.sort_mode, SortMode::Alphabetical, "A-Z");
                         ui.selectable_value(&mut self.sort_mode, SortMode::NewestFirst, "New");
                         ui.selectable_value(&mut self.sort_mode, SortMode::RecentlyUpdated, "Last");
                     });
                     ui.separator();
                 }

                 egui::ScrollArea::vertical().show(ui, |ui| {
                     ui.vertical(|ui| {
                         match self.mode {
                             AppMode::Characters => {
                                 // Sort characters based on mode
                                 // We create a viewing list of indices to avoid cloning the whole vec repeatedly if possible,
                                 // but for simplicity and since we want to iterate:
                                 let mut indices: Vec<usize> = (0..self.characters.len()).collect();
                                 match self.sort_mode {
                                     SortMode::Alphabetical => {
                                         indices.sort_by(|&a, &b| self.characters[a].name.to_lowercase().cmp(&self.characters[b].name.to_lowercase()));
                                     },
                                     SortMode::NewestFirst => {
                                         indices.sort_by(|&a, &b| self.characters[b].created_at.cmp(&self.characters[a].created_at));
                                     },
                                     SortMode::RecentlyUpdated => {
                                         indices.sort_by(|&a, &b| self.characters[b].updated_at.cmp(&self.characters[a].updated_at));
                                     },
                                 }

                                 for i in indices {
                                     let char = &self.characters[i];
                                     if ui.button(&char.name).clicked() {
                                         self.selected_character = Some(char.clone());
                                         self.active_char_tab = CharacterTab::MainData;
                                         self.status_message = None;
                                         // Load links
                                         self.load_links(char.id);
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

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.mode {
                AppMode::Characters => {
                    let mut save_req = None;
                    let mut toggle_requests = Vec::new();
                    
                    if let Some(character) = &mut self.selected_character {
                        ui.heading("Edit Character");
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
    }
}
