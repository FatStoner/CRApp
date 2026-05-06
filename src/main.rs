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
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_tracing() -> tracing_appender::non_blocking::WorkerGuard {
    std::fs::create_dir_all("data/logs").unwrap_or_default();
    let file_appender = tracing_appender::rolling::never("data/logs", "crapp.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false);

    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout);

    // Domyślny poziom logowania to INFO
    let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(stdout_layer)
        .with(file_layer)
        .init();

    guard
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _log_guard = init_tracing();
    tracing::info!("Starting CRApp...");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let _enter = rt.enter();

    let result = run_app(&rt);
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

fn run_app(rt: &tokio::runtime::Runtime) -> Result<(), Box<dyn std::error::Error>> {
    // Spawn update check in background (non-blocking)
    #[cfg(not(debug_assertions))]
    {
        // We only check if the setting allows it
        let db_check = rt.block_on(async { Database::init().await.ok() });
        // Note: we can't easily access the DB here properly without init, but we init below.
        // Actually, we should probably do this AFTER db init or read the file manually?
        // Simpler: Just spawn it, and inside the thread check the DB?
        // Or better: Let CrapApp handle it in its init!
        // Moving update check to CrapApp::new or CrapApp::update is better for access to settings.
        // BUT, CrapApp::new is synchronous.

        // Let's defer this to CrapApp initialization where we load settings.
    }

    // Initialize Database
    let db = rt.block_on(async { Database::init().await })
        .map_err(|e| e as Box<dyn std::error::Error>)?;

    // Clean up unused media
    // We swallow errors here (logging them) to not crash the app on cleanup failure
    if let Err(e) = rt.block_on(async { cleaner::cleanup_unused_media(&db.pool).await }) {
        tracing::warn!("Failed to cleanup unused media: {}", e);
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
