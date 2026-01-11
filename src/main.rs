mod card_v2;
mod db;
mod models;
mod ui;

use db::Database;
use ui::CrapApp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_app().await;
    if let Err(e) = result {
        rfd::MessageDialog::new()
            .set_title("Startup Error")
            .set_description(&format!("The application failed to start:\n\n{}", e))
            .set_level(rfd::MessageLevel::Error)
            .show();
        return Err(e);
    }
    Ok(())
}

async fn run_app() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Database
    let db = Database::init()
        .await
        .map_err(|e| e as Box<dyn std::error::Error>)?;

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Character Repository Application",
        options,
        Box::new(|cc| Ok(Box::new(CrapApp::new(cc, db)))),
    )?;

    Ok(())
}
