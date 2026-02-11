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
    // Check for updates before starting the app
    #[cfg(not(debug_assertions))]
    {
        match updater::check_and_update() {
            Ok(true) => {
                // Update was successful, show notification and restart
                rfd::MessageDialog::new()
                    .set_title("Update Installed")
                    .set_description("CRApp has been updated to the latest version.\nThe application will now restart.")
                    .set_level(rfd::MessageLevel::Info)
                    .show();

                // Restart the application
                if let Err(e) = updater::restart_application() {
                    eprintln!("Failed to restart application: {}", e);
                    rfd::MessageDialog::new()
                        .set_title("Restart Failed")
                        .set_description(&format!(
                            "Please restart the application manually.\n\nError: {}",
                            e
                        ))
                        .set_level(rfd::MessageLevel::Warning)
                        .show();
                }
                return Ok(());
            }
            Ok(false) => {
                // No update available, continue normally
                println!("No updates available");
            }
            Err(e) => {
                // Update check failed, log but don't block app startup
                eprintln!("Update check failed: {}", e);
                // Don't show a dialog for update failures to avoid annoying users
            }
        }
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
