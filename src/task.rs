use crate::error::AppError;
use crate::ui::types::UiEvent;
use eframe::egui::Context;
use std::future::Future;
use tokio::task::JoinHandle;

/// Spawns a background task and monitors it for panics.
/// If a panic occurs, it logs the error, sends a generic failure message to the channel,
/// and forces an egui repaint.
pub fn spawn_supervised<F, T>(
    ctx: Context,
    future: F,
    error_tx: tokio::sync::mpsc::Sender<UiEvent>,
) -> JoinHandle<()>
where
    F: Future<Output = Result<T, AppError>> + Send + 'static,
    T: Send + 'static,
{
    tokio::spawn(async move {
        let inner_handle = tokio::spawn(future);

        match inner_handle.await {
            Ok(Ok(_)) => {
                // Task succeeded (any necessary success events should be sent by the task itself)
            }
            Ok(Err(e)) => {
                // Task failed with an expected error
                tracing::error!("Background task failed: {:?}", e);
                let _ = error_tx.send(UiEvent::AppError(e)).await;
                ctx.request_repaint();
            }
            Err(join_error) => {
                // Task panicked or was cancelled
                if join_error.is_panic() {
                    tracing::error!("CRITICAL: Background task panicked!");
                    let _ = error_tx.send(UiEvent::AppError(AppError::TaskPanic)).await;
                    ctx.request_repaint();
                } else if join_error.is_cancelled() {
                    tracing::warn!("Background task was cancelled.");
                }
            }
        }
    })
}
