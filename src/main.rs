#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod card_v2;
mod cleaner;
mod db;
mod image_utils;
mod models;
mod ui;
mod updater;

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
    // Spawn update check in background (non-blocking)
    #[cfg(not(debug_assertions))]
    {
        // We only check if the setting allows it
        let db_check = Database::init().await.ok();
        // Note: we can't easily access the DB here properly without init, but we init below.
        // Actually, we should probably do this AFTER db init or read the file manually?
        // Simpler: Just spawn it, and inside the thread check the DB?
        // Or better: Let CrapApp handle it in its init!
        // Moving update check to CrapApp::new or CrapApp::update is better for access to settings.
        // BUT, CrapApp::new is synchronous.

        // Let's defer this to CrapApp initialization where we load settings.
    }

    // Initialize Database
    let db = Database::init()
        .await
        .map_err(|e| e as Box<dyn std::error::Error>)?;

    // Clean up unused media
    // We swallow errors here (logging them) to not crash the app on cleanup failure
    if let Err(e) = cleaner::cleanup_unused_media(&db.pool).await {
        eprintln!("Warning: Failed to cleanup unused media: {}", e);
    }

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
