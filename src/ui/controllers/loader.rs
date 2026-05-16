use super::state::CrapApp;
use crate::models::Tag;
use crate::ui::types::UiEvent;
use std::collections::HashMap;

impl CrapApp {
    pub fn refresh_all(&self) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        crate::task::spawn_supervised(ctx.clone(), async move {
            // Load characters
            let mut chars = db.get_all_characters().await?;

            // Load Tags (Bulk)
            let app_tags_res = db.get_all_tags_flat(false).await;
            let ext_tags_res = db.get_all_tags_flat(true).await;

            // Load URLs (Bulk)
            let urls_res = db.get_all_character_urls_flat().await;

            if let (Ok(app_flat), Ok(ext_flat), Ok(urls_flat)) =
                (app_tags_res, ext_tags_res, urls_res)
            {
                let mut app_map: HashMap<i64, Vec<Tag>> = HashMap::new();
                for (cid, tag) in app_flat {
                    app_map.entry(cid).or_default().push(tag);
                }

                let mut ext_map: HashMap<i64, Vec<Tag>> = HashMap::new();
                for (cid, tag) in ext_flat {
                    ext_map.entry(cid).or_default().push(tag);
                }

                let mut url_map: HashMap<i64, Vec<crate::models::CharacterUrl>> = HashMap::new();
                for url in urls_flat {
                    url_map.entry(url.character_id).or_default().push(url);
                }

                // Merge into characters
                for c in &mut chars {
                    if let Some(tags) = app_map.remove(&c.id) {
                        c.app_tags = tags;
                    }
                    if let Some(tags) = ext_map.remove(&c.id) {
                        c.external_tags = tags;
                    }
                    if let Some(urls) = url_map.remove(&c.id) {
                        c.urls = urls;
                    }
                }
            } else {
                tracing::error!("Failed to load specific tags/urls bulk");
            }

            let _ = tx.send(UiEvent::CharactersLoaded(Ok(chars))).await;

            // Load Lore Links (Bulk) - Critical for Sidebar Search
            if let Ok(links_flat) = db.get_all_lore_links_flat().await {
                let mut cl_map: HashMap<i64, Vec<i64>> = HashMap::new();
                for (cid, lid) in links_flat {
                    cl_map.entry(cid).or_default().push(lid);
                }
                let _ = tx.send(UiEvent::LoreLinksBulkLoaded(cl_map)).await;
            }

            // Load collections
            let collections = db.get_all_collections().await?;
            let _ = tx.send(UiEvent::CollectionsLoaded(Ok(collections))).await;

            // Load Lorebooks
            let mut books = db.get_all_lorebooks().await?;
            let tags_res = db.get_all_lorebook_tags_flat().await;
            if let Ok(tags_flat) = tags_res {
                let mut tag_map: HashMap<i64, Vec<Tag>> = HashMap::new();
                for (lid, tag) in tags_flat {
                    tag_map.entry(lid).or_default().push(tag);
                }
                for b in &mut books {
                    if let Some(tags) = tag_map.remove(&b.id) {
                        b.tags = tags;
                    }
                }
            }
            let _ = tx.send(UiEvent::LorebooksLoaded(Ok(books))).await;

            // Load Templates
            let templates = db.get_all_templates().await?;
            let _ = tx.send(UiEvent::TemplatesLoaded(Ok(templates))).await;

            ctx.request_repaint();
            Ok(())
        }, self.tx.clone());
    }

    pub fn reload_characters(&self) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        crate::task::spawn_supervised(ctx.clone(), async move {
            let mut chars = db.get_all_characters().await?;

            // Load Tags (Bulk) - Same logic as refresh_all to ensure tags persist
            let app_tags_res = db.get_all_tags_flat(false).await;
            let ext_tags_res = db.get_all_tags_flat(true).await;
            let urls_res = db.get_all_character_urls_flat().await;

            if let (Ok(app_flat), Ok(ext_flat), Ok(urls_flat)) =
                (app_tags_res, ext_tags_res, urls_res)
            {
                let mut app_map: HashMap<i64, Vec<Tag>> = HashMap::new();
                for (cid, tag) in app_flat {
                    app_map.entry(cid).or_default().push(tag);
                }

                let mut ext_map: HashMap<i64, Vec<Tag>> = HashMap::new();
                for (cid, tag) in ext_flat {
                    ext_map.entry(cid).or_default().push(tag);
                }

                let mut url_map: HashMap<i64, Vec<crate::models::CharacterUrl>> = HashMap::new();
                for url in urls_flat {
                    url_map.entry(url.character_id).or_default().push(url);
                }

                // Merge into characters
                for c in &mut chars {
                    if let Some(tags) = app_map.remove(&c.id) {
                        c.app_tags = tags;
                    }
                    if let Some(tags) = ext_map.remove(&c.id) {
                        c.external_tags = tags;
                    }
                    if let Some(urls) = url_map.remove(&c.id) {
                        c.urls = urls;
                    }
                }
            }

            let _ = tx.send(UiEvent::CharactersLoaded(Ok(chars))).await;
            // Load Lore Links (Bulk)
            if let Ok(links_flat) = db.get_all_lore_links_flat().await {
                let mut cl_map: HashMap<i64, Vec<i64>> = HashMap::new();
                for (cid, lid) in links_flat {
                    cl_map.entry(cid).or_default().push(lid);
                }
                let _ = tx.send(UiEvent::LoreLinksBulkLoaded(cl_map)).await;
            }
            ctx.request_repaint();
            Ok(())
        }, self.tx.clone());
    }

    pub fn reload_lorebooks(&self) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        crate::task::spawn_supervised(ctx.clone(), async move {
            let mut books = db.get_all_lorebooks().await?;
            let tags_res = db.get_all_lorebook_tags_flat().await;
            if let Ok(tags_flat) = tags_res {
                let mut tag_map: HashMap<i64, Vec<Tag>> = HashMap::new();
                for (lid, tag) in tags_flat {
                    tag_map.entry(lid).or_default().push(tag);
                }
                for b in &mut books {
                    if let Some(tags) = tag_map.remove(&b.id) {
                        b.tags = tags;
                    }
                }
            }
            let _ = tx.send(UiEvent::LorebooksLoaded(Ok(books))).await;
            ctx.request_repaint();
            Ok(())
        }, self.tx.clone());
    }

    pub fn load_links(&self, char_id: i64) {
        if char_id == 0 {
            return;
        }
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        crate::task::spawn_supervised(ctx.clone(), async move {
            let result = db.get_lore_links(char_id).await.map_err(|e| e.to_string());
            let _ = tx.send(UiEvent::LoreLinksLoaded(result)).await;
            ctx.request_repaint();
            Ok(())
        }, self.tx.clone());
    }

    // Loads tags for a single character (used after selection or tag operations)
    pub fn load_tags(&self, char_id: i64) {
        if char_id == 0 {
            return;
        }
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        crate::task::spawn_supervised(ctx.clone(), async move {
            let app_tags = db.get_tags_for_character(char_id, false).await?;
            let ext_tags = db.get_tags_for_character(char_id, true).await?;
            let _ = tx.send(UiEvent::TagsLoaded(Ok((char_id, app_tags, ext_tags)))).await;
            ctx.request_repaint();
            Ok(())
        }, self.tx.clone());
    }

    pub fn reload_collections(&self) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        crate::task::spawn_supervised(ctx.clone(), async move {
            let collections = db.get_all_collections().await?;
            let _ = tx.send(UiEvent::CollectionsLoaded(Ok(collections))).await;
            ctx.request_repaint();
            Ok(())
        }, self.tx.clone());
    }

    pub fn load_lorebook_tags(&self, lorebook_id: i64) {
        if lorebook_id == 0 {
            return;
        }
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        crate::task::spawn_supervised(ctx.clone(), async move {
            let tags = db.get_tags_for_lorebook(lorebook_id).await?;
            let _ = tx.send(UiEvent::LorebookTagsLoaded(Ok((lorebook_id, tags)))).await;
            ctx.request_repaint();
            Ok(())
        }, self.tx.clone());
    }

    pub fn load_lorebook_entries(&self, lorebook_id: i64) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ctx = self.ctx.clone();
        crate::task::spawn_supervised(ctx.clone(), async move {
            let entries = db.get_entries_for_lorebook(lorebook_id).await?;
            let _ = tx.send(UiEvent::LorebookEntriesLoaded(Ok((lorebook_id, entries)))).await;
            ctx.request_repaint();
            Ok(())
        }, self.tx.clone());
    }
}
