#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod card_v2;
mod cleaner;
mod db;
mod error;
mod image_utils;
mod models;
mod task;
mod ui;
mod updater;

use std::panic;
use std::fs::File;
use std::io::Write;
use anyhow::{Context, Result};
use db::Database;
use ui::CrapApp;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn setup_crash_handler() {
    panic::set_hook(Box::new(|panic_info| {
        let _ = std::fs::create_dir_all("data/logs");
        let mut crash_log = File::create("data/logs/crash.log").unwrap_or_else(|_| {
            std::io::stderr().write_all(b"Failed to create data/logs/crash.log\n").unwrap();
            std::process::exit(1);
        });

        let payload = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            *s
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.as_str()
        } else {
            "Unknown panic payload"
        };

        let location = panic_info.location().map(|l| format!("{}:{}", l.file(), l.line())).unwrap_or_else(|| "unknown location".to_string());

        let _ = writeln!(crash_log, "CRITICAL PANIC at {}:\n{}", location, payload);
        let _ = writeln!(crash_log, "\nBacktrace is handled via OS or RUST_BACKTRACE if enabled.");
        
        std::process::exit(1);
    }));
}

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

fn init_tracing() -> Result<tracing_appender::non_blocking::WorkerGuard> {
    std::fs::create_dir_all("data/logs").context("Failed to create 'data/logs' directory")?;
    
    cleanup_old_logs("data/logs", 5);

    let file_appender = tracing_appender::rolling::daily("data/logs", "crapp.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = tracing_subscriber::fmt::layer()
        .json()
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

    Ok(guard)
}

fn main() {
    setup_crash_handler();

    match run_gatekeeper() {
        Ok((rt, db, _log_guard)) => {
            let _enter_guard = rt.enter();

            let options = eframe::NativeOptions {
                viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
                ..Default::default()
            };

            if let Err(e) = eframe::run_native(
                "Character Repository Application",
                options,
                Box::new(|cc| Ok(Box::new(CrapApp::new(cc, db)))),
            ) {
                tracing::error!("GUI Error: {}", e);
                rfd::MessageDialog::new()
                    .set_title("GUI Error")
                    .set_description(&format!("The graphical interface failed to launch:\n\n{}", e))
                    .set_level(rfd::MessageLevel::Error)
                    .show();
                std::process::exit(1);
            }
        }
        Err(e) => {
            if tracing::dispatcher::has_been_set() {
                tracing::error!("Initialization Gatekeeper failed: {:#}", e);
            }
            rfd::MessageDialog::new()
                .set_title("Initialization Error")
                .set_description(&format!("The application failed to initialize properly:\n\n{:#}", e))
                .set_level(rfd::MessageLevel::Error)
                .show();
            std::process::exit(1);
        }
    }
}

fn run_gatekeeper() -> Result<(tokio::runtime::Runtime, Database, tracing_appender::non_blocking::WorkerGuard)> {
    // 1. Storage & IO Setup
    let log_guard = init_tracing().context("Logging subsystem initialization failed")?;
    tracing::info!("Starting CRApp Initialization...");

    // 2. Async Runtime Bootstrapping
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("Failed to initialize Tokio async runtime")?;
    
    let _enter = rt.enter();

    // 3. Database Connection
    let db = rt.block_on(async { Database::init().await })
        .map_err(|e| anyhow::anyhow!("Failed to initialize database: {}", e))?;

    if let Err(e) = rt.block_on(async { cleaner::cleanup_unused_media(&db.pool).await }) {
        tracing::warn!("Failed to cleanup unused media: {}", e);
    }

    Ok((rt, db, log_guard))
}
