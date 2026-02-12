use super::state::CrapApp;
use crate::ui::UiEvent;

impl CrapApp {
    /// Delete lorebook entry asynchronously and reload entries for the lorebook
    pub fn delete_lorebook_entry_async(&self, entry_id: i64, lorebook_id: i64) {
        let tx = self.tx.clone();
        let db = self.db.clone();

        tokio::spawn(async move {
            match db.delete_lorebook_entry(entry_id).await {
                Ok(_) => {
                    let _ = tx.send(UiEvent::LorebookEntryDeleted(Ok(entry_id))).await;
                }
                Err(e) => {
                    let _ = tx
                        .send(UiEvent::LorebookEntryDeleted(Err(e.to_string())))
                        .await;
                }
            }
            // Trigger reload of entries for this lorebook
            if let Ok(entries) = db.get_entries_for_lorebook(lorebook_id).await {
                let _ = tx
                    .send(UiEvent::LorebookEntriesLoaded(Ok((lorebook_id, entries))))
                    .await;
            }
        });
    }
}
