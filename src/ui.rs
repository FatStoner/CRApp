use eframe::egui;
use tokio::sync::mpsc;
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
}

impl CrapApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, db: Database) -> Self {
        let (tx, rx) = mpsc::channel(10);
        
        let app = Self { 
            db,
            characters: Vec::new(),
            selected_character: None,
            tx,
            rx,
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

    fn save_character(&self, mut character: Character) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        tokio::spawn(async move {
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
                        Ok(chars) => self.characters = chars,
                        Err(e) => eprintln!("Error loading characters: {}", e),
                    }
                }
                UiEvent::CharacterSaved(result) => {
                    match result {
                        Ok(updated_char) => {
                            // Update the selected character with the verified/ID-updated version
                            // only if the IDs match or it was a new creation (ID 0 turned into ID X)
                            // Ideally we want to keep the editing session valid.
                            self.selected_character = Some(updated_char);
                        }
                        Err(e) => eprintln!("Error saving character: {}", e),
                    }
                }
            }
        }

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Characters");
            ui.separator();
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical(|ui| {
                    for char in &self.characters {
                        if ui.button(&char.name).clicked() {
                            self.selected_character = Some(char.clone());
                        }
                    }
                });
            });
            
            ui.add_space(10.0);
            if ui.button("+ Add New").clicked() {
                self.selected_character = Some(Character::default());
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
                    ui.text_edit_multiline(&mut character.personality);
                    
                    ui.label("First Message");
                    ui.text_edit_multiline(&mut character.first_message);
                    
                    ui.label("Author Notes");
                    ui.text_edit_multiline(&mut character.author_notes);

                    ui.label("Avatar Path");
                    let mut path_str = character.avatar_path.clone().unwrap_or_default();
                    if ui.text_edit_singleline(&mut path_str).changed() {
                        character.avatar_path = if path_str.is_empty() { None } else { Some(path_str) };
                    }
                    
                    ui.add_space(10.0);
                    if ui.button("Save").clicked() {
                        return Some(character.clone());
                    }
                    None
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
