use eframe::egui;
use tokio::sync::mpsc;
use crate::db::Database;
use crate::models::Character;

pub struct CrapApp {
    db: Database,
    characters: Vec<Character>,
    // Receiver for async character loading
    // We use a specific error type or just Box<dyn Error> for simplicity here as we are in a GUI context
    rx: Option<mpsc::Receiver<Result<Vec<Character>, String>>>,
}

impl CrapApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, db: Database) -> Self {
        let (tx, rx) = mpsc::channel(1);
        let db_clone = db.clone();

        // Spawn async task to fetch characters
        tokio::spawn(async move {
            let result = db_clone.get_all_characters().await
                .map_err(|e| e.to_string());
            
            // Send result back to UI thread
            let _ = tx.send(result).await;
        });

        Self { 
            db,
            characters: Vec::new(),
            rx: Some(rx),
        }
    }
}

impl eframe::App for CrapApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for async updates
        if let Some(rx) = &mut self.rx {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(chars) => {
                        self.characters = chars;
                    }
                    Err(e) => {
                        eprintln!("Failed to load characters: {}", e);
                    }
                }
                // We only expect one message (initial load), but keeping rx open/Option is fine logic
                // If we wanted to keep reloading we'd need a different architecture possibly, 
                // but for "start of app" this is sufficient.
                // We could set self.rx = None here if we're sure it's one-shot.
                self.rx = None; 
            }
        }

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Postacie");
            ui.separator();
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                for char in &self.characters {
                    if ui.button(&char.name).clicked() {
                        // Handle selection TODO
                    }
                }
            });
            
            ui.add_space(10.0);
            if ui.button("+ Dodaj Nową").clicked() {
                // TODO: Add new character logic
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Main Dashboard");
            ui.label("Wybierz postać z listy po lewej.");
        });
    }
}
