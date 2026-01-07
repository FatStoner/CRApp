use eframe::egui;
use tokio::sync::mpsc;
use std::time::{Duration, Instant};
use crate::db::Database;
use crate::models::Character;

pub enum UiEvent {
    CharactersLoaded(Result<Vec<Character>, String>),
    CharacterSaved(Result<Character, String>),
}

pub struct CrapApp {
    db: Database,
    characters: Vec<Character>,
    selected_character: Option<Character>,
    tx: mpsc::Sender<UiEvent>,
    rx: mpsc::Receiver<UiEvent>,
    
    // UI State
    is_saving: bool,
    status_message: Option<(String, egui::Color32)>,
    status_clear_time: Option<Instant>,
    loading_error: Option<String>,
}

impl CrapApp {
    pub fn new(cc: &eframe::CreationContext<'_>, db: Database) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let (tx, rx) = mpsc::channel(10);
        
        let app = Self { 
            db,
            characters: Vec::new(),
            selected_character: None,
            tx,
            rx,
            is_saving: false,
            status_message: None,
            status_clear_time: None,
            loading_error: None,
        };
        
        app.reload_characters();
        app
    }

    fn reload_characters(&self) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        tokio::spawn(async move {
            let result = db.get_all_characters().await
                .map_err(|e| e.to_string());
            let _ = tx.send(UiEvent::CharactersLoaded(result)).await;
        });
    }

    fn save_character(&mut self, mut character: Character) {
        self.is_saving = true;
        self.status_message = None; // Clear previous status
        
        let tx = self.tx.clone();
        let db = self.db.clone();
        
        tokio::spawn(async move {
            // Artificial delay for better UX if it's too fast (optional, but good for "Saving..." visibility check)
            // tokio::time::sleep(Duration::from_millis(500)).await; 

            if let Err(e) = db.upsert_character(&mut character).await {
                let _ = tx.send(UiEvent::CharacterSaved(Err(e.to_string()))).await;
            } else {
                let _ = tx.send(UiEvent::CharacterSaved(Ok(character))).await;
                // Also trigger a reload to update the list
                let list = db.get_all_characters().await.map_err(|e| e.to_string());
                let _ = tx.send(UiEvent::CharactersLoaded(list)).await;
            }
        });
    }
}

impl eframe::App for CrapApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle async events
        while let Ok(event) = self.rx.try_recv() {
            match event {
                UiEvent::CharactersLoaded(result) => {
                    match result {
                        Ok(chars) => {
                            self.characters = chars;
                            self.loading_error = None;
                        }
                        Err(e) => {
                            eprintln!("Error loading characters: {}", e);
                            self.loading_error = Some(e);
                        }
                    }
                }
                UiEvent::CharacterSaved(result) => {
                    self.is_saving = false;
                    match result {
                        Ok(updated_char) => {
                            self.selected_character = Some(updated_char);
                            self.status_message = Some(("Saved successfully!".to_string(), egui::Color32::GREEN));
                        }
                        Err(e) => {
                            self.status_message = Some((format!("Error: {}", e), egui::Color32::RED));
                        }
                    }
                    // Set timeout for status message
                    self.status_clear_time = Some(Instant::now() + Duration::from_secs(3));
                }
            }
        }

        // Clean up status message
        if let Some(deadline) = self.status_clear_time {
            if Instant::now() > deadline {
                self.status_message = None;
                self.status_clear_time = None;
            } else {
                ctx.request_repaint(); // Animation frame for timer
            }
        }

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.heading("Characters");
            ui.separator();
            
            if let Some(err) = &self.loading_error {
                ui.colored_label(egui::Color32::RED, format!("Failed to load: {}", err));
                if ui.button("Retry").clicked() {
                    self.reload_characters();
                }
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical(|ui| {
                        for char in &self.characters {
                            if ui.button(&char.name).clicked() {
                                self.selected_character = Some(char.clone());
                                // Also clear status when switching
                                self.status_message = None;
                            }
                        }
                    });
                });
            }
            
            ui.add_space(10.0);
            if ui.button("+ Add New").clicked() {
                self.selected_character = Some(Character::default());
                self.status_message = None;
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut save_request = None;

            if let Some(character) = &mut self.selected_character {
                ui.heading("Edit Character");
                ui.separator();
                
                // We use .inner to get the return value of the closure
                save_request = egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label("Name (File Name)");
                    ui.text_edit_singleline(&mut character.name);
                    
                    ui.label("Character Name (In-Game)");
                    ui.text_edit_singleline(&mut character.char_name);
                    
                    ui.label("Title / Role");
                    ui.text_edit_singleline(&mut character.char_title);
                    
                    ui.label("Personality");
                    if ui.text_edit_multiline(&mut character.personality).changed() {
                        // calculate on change if needed, but we calculate for display below anyway
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        ui.label(format!("Tokens: {}", crate::models::count_tokens(&character.personality)));
                    });
                    
                    ui.label("First Message");
                    ui.text_edit_multiline(&mut character.first_message);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        ui.label(format!("Tokens: {}", crate::models::count_tokens(&character.first_message)));
                    });
                    
                    ui.label("Author Notes");
                    ui.text_edit_multiline(&mut character.author_notes);

                    ui.label("Avatar");
                    if let Some(path_str) = &character.avatar_path {
                         // Convert stored path to absolute URI for egui
                         if let Ok(abs_path) = std::fs::canonicalize(path_str) {
                             let uri = format!("file://{}", abs_path.to_string_lossy());
                             ui.add(egui::Image::new(uri).max_width(200.0).max_height(200.0));
                         } else {
                             ui.label(format!("Image not found at: {}", path_str));
                         }
                    }

                    if ui.button("Browse Avatar").clicked() {
                        if let Some(path) = rfd::FileDialog::new().add_filter("image", &["png", "jpg", "jpeg", "webp"]).pick_file() {
                            let dest_dir = std::path::Path::new("data/avatars");
                            if let Err(e) = std::fs::create_dir_all(dest_dir) {
                                eprintln!("Failed to create avatars directory: {}", e);
                            } else {
                                if let Some(file_name) = path.file_name() {
                                    let dest_path = dest_dir.join(file_name);
                                    match std::fs::copy(&path, &dest_path) {
                                        Ok(_) => {
                                            character.avatar_path = Some(dest_path.to_string_lossy().to_string());
                                        }
                                        Err(e) => eprintln!("Failed to copy avatar: {}", e),
                                    }
                                }
                            }
                        }
                    }
                    
                    ui.add_space(20.0);
                    
                    // Save bar
                    ui.horizontal(|ui| {
                        if self.is_saving {
                            ui.add(egui::Spinner::new());
                            ui.label("Saving...");
                            None
                        } else {
                            if ui.button("Save").clicked() {
                                return Some(character.clone());
                            }
                            
                            if let Some((msg, color)) = &self.status_message {
                                ui.colored_label(*color, msg);
                            }
                            None
                        }
                    }).inner

                }).inner;
            } else {
                ui.heading("Main Dashboard");
                ui.label("Select a character or create a new one");
            }

            if let Some(char_to_save) = save_request {
                self.save_character(char_to_save);
            }
        });
    }
}
