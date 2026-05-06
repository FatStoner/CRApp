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

fn cleanup_old_logs(log_dir: &str, keep_max: usize) {
    if let Ok(entries) = std::fs::read_dir(log_dir) {
        let mut logs: Vec<_> = entries
            .filter_map(Result::ok)
            .filter(|e| {
                e.path().is_file() && e.file_name().to_string_lossy().starts_with("crapp.log")
            })
            .collect();

        if logs.len() <= keep_max {
            return;
        }

        // Sort by modification date (newest first)
        logs.sort_by_key(|e| {
            std::cmp::Reverse(
                e.metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
            )
        });

        for entry in logs.into_iter().skip(keep_max) {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

fn init_tracing() -> tracing_appender::non_blocking::WorkerGuard {
    std::fs::create_dir_all("data/logs").unwrap_or_default();
    
    cleanup_old_logs("data/logs", 5);

    let file_appender = tracing_appender::rolling::daily("data/logs", "crapp.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false);

    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout);

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

    if let Err(e) = run_app(&rt) {
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
    let db = rt.block_on(async { Database::init().await })
        .map_err(|e| e as Box<dyn std::error::Error>)?;

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
