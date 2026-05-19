use super::state::CrapApp;
use crate::ui::types::UiEvent;

impl CrapApp {
    /// Adds a tag to a character (either app tag or external tag)
    pub fn add_tag(&self, char_id: i64, name: String, is_external: bool) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        crate::task::spawn_supervised(ctx.clone(), async move {
            db.add_tag_to_character(char_id, &name, is_external).await?;
            let _ = tx.send(UiEvent::TagOperationFinished(Ok(()))).await;
            ctx.request_repaint();
            Ok(())
        }, self.tx.clone());
    }

    /// Removes a tag from a character
    pub fn remove_tag(&self, char_id: i64, tag_id: i64, is_external: bool) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        crate::task::spawn_supervised(ctx.clone(), async move {
            db.remove_tag_from_character(char_id, tag_id, is_external).await?;
            let _ = tx.send(UiEvent::TagOperationFinished(Ok(()))).await;
            ctx.request_repaint();
            Ok(())
        }, self.tx.clone());
    }

    /// Adds a tag to a lorebook
    pub fn add_tag_to_lorebook(&self, lorebook_id: i64, name: String) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        crate::task::spawn_supervised(ctx.clone(), async move {
            db.add_tag_to_lorebook(lorebook_id, &name).await?;
            tracing::info!("Added tag '{}' to lorebook ID: {}", name, lorebook_id);
            let tags = db.get_tags_for_lorebook(lorebook_id).await?;
            let _ = tx.send(UiEvent::LorebookTagsLoaded(Ok((lorebook_id, tags)))).await;
            let _ = tx.send(UiEvent::LorebookTagOperationFinished(Ok(()))).await;
            ctx.request_repaint();
            Ok(())
        }, self.tx.clone());
    }

    /// Removes a tag from a lorebook
    pub fn remove_tag_from_lorebook(&self, lorebook_id: i64, tag_id: i64) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        crate::task::spawn_supervised(ctx.clone(), async move {
            db.remove_tag_from_lorebook(lorebook_id, tag_id).await?;
            tracing::info!("Removed tag ID {} from lorebook ID: {}", tag_id, lorebook_id);
            let tags = db.get_tags_for_lorebook(lorebook_id).await?;
            let _ = tx.send(UiEvent::LorebookTagsLoaded(Ok((lorebook_id, tags)))).await;
            let _ = tx.send(UiEvent::LorebookTagOperationFinished(Ok(()))).await;
            ctx.request_repaint();
            Ok(())
        }, self.tx.clone());
    }

    /// Adds a new entry to a lorebook
    pub fn add_entry_to_lorebook(&self, lorebook_id: i64) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        crate::task::spawn_supervised(ctx.clone(), async move {
            let mut entry = crate::models::LorebookEntry::default();
            entry.lorebook_id = lorebook_id;

            let id = db.add_entry_to_lorebook(&entry).await?;
            tracing::info!("Added new entry to lorebook ID: {} (Entry ID: {})", lorebook_id, id);
            let _ = tx.send(UiEvent::LorebookEntryAdded(Ok(id))).await;
            let entries = db.get_entries_for_lorebook(lorebook_id).await?;
            let _ = tx.send(UiEvent::LorebookEntriesLoaded(Ok((lorebook_id, entries)))).await;
            ctx.request_repaint();
            Ok(())
        }, self.tx.clone());
    }

    /// Adds a SPECIFIC entry object to a lorebook (used for Paste / Import)
    pub fn add_specific_entry_to_lorebook(&self, mut entry: crate::models::LorebookEntry) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();

        // Ensure ID is 0 for insert
        entry.id = 0;

        crate::task::spawn_supervised(ctx.clone(), async move {
            let lid = entry.lorebook_id;
            let id = db.add_entry_to_lorebook(&entry).await?;
            tracing::info!("Added specific entry to lorebook ID: {} (Entry ID: {})", lid, id);
            let _ = tx.send(UiEvent::LorebookEntryAdded(Ok(id))).await;
            let entries = db.get_entries_for_lorebook(lid).await?;
            let _ = tx.send(UiEvent::LorebookEntriesLoaded(Ok((lid, entries)))).await;
            ctx.request_repaint();
            Ok(())
        }, self.tx.clone());
    }

    /// Saves (updates) a lorebook entry
    pub fn save_lorebook_entry(&self, entry: crate::models::LorebookEntry) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        crate::task::spawn_supervised(ctx.clone(), async move {
            db.update_lorebook_entry(&entry).await?;
            tracing::info!("Updated lorebook entry ID: {} (Lorebook ID: {})", entry.id, entry.lorebook_id);
            let _ = tx.send(UiEvent::LorebookEntrySaved(Ok(()))).await;
            let entries = db.get_entries_for_lorebook(entry.lorebook_id).await?;
            let _ = tx.send(UiEvent::LorebookEntriesLoaded(Ok((entry.lorebook_id, entries)))).await;
            ctx.request_repaint();
            Ok(())
        }, self.tx.clone());
    }
}
