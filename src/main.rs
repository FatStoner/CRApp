mod db;
mod models;
mod ui;
mod card_v2;

use db::Database;
use ui::CrapApp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Database
    let db = Database::init().await?;

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };
    
    // We need to move db into the closure, but eframe::run_native expects closure to be synchronous usually if not using newest patterns but the closure itself returns the app.
    // However, eframe::run_native blocks.
    
    eframe::run_native(
        "Character Repository Application",
        options,
        Box::new(|cc| Ok(Box::new(CrapApp::new(cc, db)))),
    )?;

    Ok(())
}
