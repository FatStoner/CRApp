use super::state::CrapApp;
use crate::ui::types::UiEvent;

impl CrapApp {
    /// Adds a tag to a character (either app tag or external tag)
    pub fn add_tag(&self, char_id: i64, name: String, is_external: bool) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let res = db
                .add_tag_to_character(char_id, &name, is_external)
                .await
                .map_err(|e| e.to_string());
            let _ = tx.send(UiEvent::TagOperationFinished(res)).await;
            ctx.request_repaint();
        });
    }

    /// Removes a tag from a character
    pub fn remove_tag(&self, char_id: i64, tag_id: i64, is_external: bool) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let res = db
                .remove_tag_from_character(char_id, tag_id, is_external)
                .await
                .map_err(|e| e.to_string());
            let _ = tx.send(UiEvent::TagOperationFinished(res)).await;
            ctx.request_repaint();
        });
    }

    /// Adds a tag to a lorebook
    pub fn add_tag_to_lorebook(&self, lorebook_id: i64, name: String) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let res = db.add_tag_to_lorebook(lorebook_id, &name).await;

            match res {
                Ok(_) => {
                    let tags = db.get_tags_for_lorebook(lorebook_id).await;
                    if let Ok(t) = tags {
                        let _ = tx
                            .send(UiEvent::LorebookTagsLoaded(Ok((lorebook_id, t))))
                            .await;
                    }
                    let _ = tx.send(UiEvent::LorebookTagOperationFinished(Ok(()))).await;
                }
                Err(e) => {
                    let _ = tx
                        .send(UiEvent::LorebookTagOperationFinished(Err(e.to_string())))
                        .await;
                }
            }
            ctx.request_repaint();
        });
    }

    /// Removes a tag from a lorebook
    pub fn remove_tag_from_lorebook(&self, lorebook_id: i64, tag_id: i64) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            match db.remove_tag_from_lorebook(lorebook_id, tag_id).await {
                Ok(_) => {
                    let tags = db.get_tags_for_lorebook(lorebook_id).await;
                    if let Ok(t) = tags {
                        let _ = tx
                            .send(UiEvent::LorebookTagsLoaded(Ok((lorebook_id, t))))
                            .await;
                    }
                    let _ = tx.send(UiEvent::LorebookTagOperationFinished(Ok(()))).await;
                }
                Err(e) => {
                    let _ = tx
                        .send(UiEvent::LorebookTagOperationFinished(Err(e.to_string())))
                        .await;
                }
            }
            ctx.request_repaint();
        });
    }

    /// Adds a new entry to a lorebook
    pub fn add_entry_to_lorebook(&self, lorebook_id: i64) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let mut entry = crate::models::LorebookEntry::default();
            entry.lorebook_id = lorebook_id;

            match db.add_entry_to_lorebook(&entry).await {
                Ok(id) => {
                    let _ = tx.send(UiEvent::LorebookEntryAdded(Ok(id))).await;
                    // Auto-reload
                    match db.get_entries_for_lorebook(lorebook_id).await {
                        Ok(entries) => {
                            let _ = tx
                                .send(UiEvent::LorebookEntriesLoaded(Ok((lorebook_id, entries))))
                                .await;
                        }
                        Err(_) => {}
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(UiEvent::LorebookEntryAdded(Err(e.to_string())))
                        .await;
                }
            }
            ctx.request_repaint();
        });
    }

    /// Saves (updates) a lorebook entry
    pub fn save_lorebook_entry(&self, entry: crate::models::LorebookEntry) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            match db.update_lorebook_entry(&entry).await {
                Ok(_) => {
                    let _ = tx.send(UiEvent::LorebookEntrySaved(Ok(()))).await;
                    if let Ok(entries) = db.get_entries_for_lorebook(entry.lorebook_id).await {
                        let _ = tx
                            .send(UiEvent::LorebookEntriesLoaded(Ok((
                                entry.lorebook_id,
                                entries,
                            ))))
                            .await;
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(UiEvent::LorebookEntrySaved(Err(e.to_string())))
                        .await;
                }
            }
            ctx.request_repaint();
        });
    }
}
