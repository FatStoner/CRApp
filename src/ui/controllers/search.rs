use super::state::CrapApp;
use crate::models::{Character, DeepSearchResult, Lorebook, SearchResultKind};
use crate::ui::types::{AppMode, SortDirection, UiEvent};

impl CrapApp {
    pub fn perform_deep_search(&mut self) {
        if self.deep_search_query.trim().is_empty() {
            return;
        }

        self.is_deep_searching = true;
        self.mode = AppMode::DeepSearch;
        self.deep_search_results.clear();
        self.deep_search_sort = None; // Reset sort on new search

        let query = self.deep_search_query.clone();
        let filter_collection = self.deep_search_filter_collection;
        let char_filters = self.deep_search_char_field_filters.clone();
        let all_collections = self.collections.clone();
        let tx = self.tx.clone();
        let db = self.db.clone();
        let lore_filters = self.deep_search_lore_field_filters.clone();

        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let mut results = Vec::new();

            // 1. Search Characters Text
            let mut char_map: std::collections::HashMap<i64, Character> =
                std::collections::HashMap::new();

            if let Ok(chars) = db.search_characters_text(&query).await {
                for c in chars {
                    char_map.insert(c.id, c);
                }
            }

            // 2. Search Tags
            let mut tag_matches: Vec<(i64, String, bool)> = Vec::new();
            if char_filters.tags {
                if let Ok(tags) = db.search_tags_matching(&query).await {
                    tag_matches = tags;
                }
            }

            // 3. Fetch missing characters found by tags
            let found_ids: std::collections::HashSet<i64> =
                tag_matches.iter().map(|(id, _, _)| *id).collect();
            let missing_ids: Vec<i64> = found_ids
                .into_iter()
                .filter(|id| !char_map.contains_key(id))
                .collect();

            if !missing_ids.is_empty() {
                if let Ok(fetched) = db.get_characters_by_ids(&missing_ids).await {
                    for c in fetched {
                        char_map.insert(c.id, c);
                    }
                }
            }

            // 3.5. Fetch URLs for result candidates
            if char_filters.urls && !char_map.is_empty() {
                if let Ok(urls) = db.get_all_character_urls_flat().await {
                    for u in urls {
                        if let Some(c) = char_map.get_mut(&u.character_id) {
                            c.urls.push(u);
                        }
                    }
                }
            }

            // 4. Build Character Results

            for (_, c) in char_map {
                let mut matches = Vec::new();

                // Use widget helper
                use crate::ui::widgets::extract_snippets;

                if char_filters.name {
                    for s in extract_snippets(&c.name, &query) {
                        matches.push(("Name".to_string(), s));
                    }
                }
                if char_filters.char_title {
                    for s in extract_snippets(&c.char_title, &query) {
                        matches.push(("Title".to_string(), s));
                    }
                }
                if char_filters.personality {
                    for s in extract_snippets(&c.personality, &query) {
                        matches.push(("Personality".to_string(), s));
                    }
                }
                if char_filters.scenario {
                    for s in extract_snippets(&c.scenario, &query) {
                        matches.push(("Scenario".to_string(), s));
                    }
                }
                if char_filters.example_dialogue {
                    for s in extract_snippets(&c.example_dialogue, &query) {
                        matches.push(("Example Dialogue".to_string(), s));
                    }
                }
                if char_filters.first_message {
                    for s in extract_snippets(&c.first_message, &query) {
                        matches.push(("First Message".to_string(), s));
                    }
                }
                if char_filters.author_notes {
                    for s in extract_snippets(&c.author_notes, &query) {
                        matches.push(("Notes".to_string(), s));
                    }
                }

                if char_filters.urls {
                    for url in &c.urls {
                        for s in extract_snippets(&url.url, &query) {
                            matches.push(("URL".to_string(), s));
                        }
                        if let Some(label) = &url.label {
                            for s in extract_snippets(label, &query) {
                                matches.push(("URL Label".to_string(), s));
                            }
                        }
                    }
                }

                if char_filters.tags {
                    for (tid, tname, is_ext) in &tag_matches {
                        if *tid == c.id {
                            let label = if *is_ext { "Ext. Tag" } else { "App Tag" };
                            matches.push((label.to_string(), tname.clone()));
                        }
                    }
                }

                if !matches.is_empty() {
                    results.push(DeepSearchResult {
                        id: c.id,
                        kind: SearchResultKind::Character,
                        display_name: c.name,
                        collection_id: c.collection_id,
                        matches,
                        index: 0, // Will be set after collection to ensure global order
                    });
                }
            }

            // 5. Build Lorebook Results
            use crate::ui::widgets::extract_snippets;
            let mut lorebook_map: std::collections::HashMap<i64, Lorebook> =
                std::collections::HashMap::new();

            // 5.1 Text Search
            if let Ok(books) = db.search_lorebooks_text(&query).await {
                for b in books {
                    lorebook_map.insert(b.id, b);
                }
            }

            // 5.2 Tags Search
            let mut lb_tag_matches: Vec<(i64, String)> = Vec::new();
            if let Ok(tags) = db.search_lorebook_tags_matching(&query).await {
                lb_tag_matches = tags;
            }

            // 5.3 Entries Search
            let mut entry_matches: Vec<crate::models::LorebookEntry> = Vec::new();
            if let Ok(entries) = db.search_lorebook_entries_text(&query).await {
                entry_matches = entries;
            }

            // 5.4 Fetch Missing Lorebooks
            let mut missing_lb_ids: std::collections::HashSet<i64> =
                std::collections::HashSet::new();
            for (lid, _) in &lb_tag_matches {
                if !lorebook_map.contains_key(lid) {
                    missing_lb_ids.insert(*lid);
                }
            }
            for entry in &entry_matches {
                if !lorebook_map.contains_key(&entry.lorebook_id) {
                    missing_lb_ids.insert(entry.lorebook_id);
                }
            }

            if !missing_lb_ids.is_empty() {
                let ids: Vec<i64> = missing_lb_ids.into_iter().collect();
                if let Ok(fetched) = db.get_lorebooks_by_ids(&ids).await {
                    for b in fetched {
                        lorebook_map.insert(b.id, b);
                    }
                }
            }

            // 5.5 Aggregate Matches
            for (_, lb) in lorebook_map {
                let mut matches = Vec::new();

                // 5.5.1 Lorebook Text Matches
                if lore_filters.title {
                    for s in extract_snippets(&lb.title, &query) {
                        matches.push(("Title".to_string(), s));
                    }
                }
                if lore_filters.description {
                    for s in extract_snippets(&lb.description, &query) {
                        matches.push(("Description".to_string(), s));
                    }
                    for s in extract_snippets(&lb.content, &query) {
                        matches.push(("Content".to_string(), s));
                    }
                }

                // 5.5.2 Tag Matches
                if lore_filters.tags {
                    for (lid, tname) in &lb_tag_matches {
                        if *lid == lb.id {
                            matches.push(("Tag".to_string(), tname.clone()));
                        }
                    }
                }

                // 5.5.3 Entry Matches
                for entry in &entry_matches {
                    if entry.lorebook_id == lb.id {
                        if lore_filters.entry_name {
                            for s in extract_snippets(&entry.name, &query) {
                                matches.push((format!("Entry: {}", entry.name), s));
                            }
                        }
                        if lore_filters.entry_keywords {
                            for s in extract_snippets(&entry.keywords, &query) {
                                matches.push((format!("Entry Keywords: {}", entry.name), s));
                            }
                        }
                        if lore_filters.entry_content {
                            for s in extract_snippets(&entry.content, &query) {
                                matches.push((format!("Entry Content: {}", entry.name), s));
                            }
                        }
                    }
                }

                if !matches.is_empty() {
                    results.push(DeepSearchResult {
                        id: lb.id,
                        kind: SearchResultKind::Lorebook,
                        display_name: lb.title,
                        collection_id: None,
                        matches,
                        index: 0, // Will be set later
                    });
                }
            }

            // Filter by collection if specified
            if let Some(filter_coll_id) = filter_collection {
                // Get all allowed collection IDs (parent + all descendants)
                let allowed_collections = {
                    let mut allowed = vec![filter_coll_id];

                    // Recursively find all children
                    let mut to_process = vec![filter_coll_id];
                    while let Some(parent_id) = to_process.pop() {
                        let children: Vec<i64> = all_collections
                            .iter()
                            .filter(|c| c.parent_id == Some(parent_id))
                            .map(|c| c.id)
                            .collect();

                        allowed.extend(&children);
                        to_process.extend(children);
                    }

                    allowed
                };

                // Filter results to only include characters from allowed collections
                results.retain(|res| {
                    if res.kind == SearchResultKind::Character {
                        if let Some(cid) = res.collection_id {
                            allowed_collections.contains(&cid)
                        } else {
                            false // Exclude uncategorized characters when filtering
                        }
                    } else {
                        true // Keep lorebooks regardless of filter
                    }
                });
            }

            // 6. Sort and Finalize
            // We want stable initial order, so we assign index here based on push order
            for (i, res) in results.iter_mut().enumerate() {
                res.index = i;
            }

            let _ = tx.send(UiEvent::DeepSearchCompleted(Ok(results))).await;
            ctx.request_repaint();
        });
    }

    pub fn sort_deep_search_results(&mut self) {
        if self.deep_search_results.is_empty() {
            return;
        }

        match self.deep_search_sort {
            Some(SortDirection::Ascending) => {
                self.deep_search_results.sort_by(|a, b| {
                    a.display_name
                        .to_lowercase()
                        .cmp(&b.display_name.to_lowercase())
                });
            }
            Some(SortDirection::Descending) => {
                self.deep_search_results.sort_by(|a, b| {
                    b.display_name
                        .to_lowercase()
                        .cmp(&a.display_name.to_lowercase())
                });
            }
            None => {
                // Restore original order
                self.deep_search_results.sort_by_key(|r| r.index);
            }
        }
    }

    pub fn ensure_token_count(&mut self, character: &Character) {
        if self.token_cache.contains_key(&character.id) {
            return;
        }
        if self.token_calc_in_progress.contains(&character.id) {
            return;
        }

        self.token_calc_in_progress.insert(character.id);
        let tx = self.tx.clone();
        let char_clone = character.clone();

        // Capture all flags
        let include_name = self.count_name_in_total;
        let include_title = self.count_title_in_total;
        let include_first = self.count_first_message_in_total;
        let include_pers = self.count_personality_in_total;
        let include_scen = self.count_scenario_in_total;
        let include_ex = self.count_example_in_total;

        tokio::spawn(async move {
            let mut total_tokens = 0;
            let mut total_chars = 0;

            if include_pers {
                let t = crate::models::count_tokens(&char_clone.personality);
                total_tokens += t;
                total_chars += char_clone.personality.len();
            }

            if include_scen {
                let t = crate::models::count_tokens(&char_clone.scenario);
                total_tokens += t;
                total_chars += char_clone.scenario.len();
            }

            if include_ex {
                let t = crate::models::count_tokens(&char_clone.example_dialogue);
                total_tokens += t;
                total_chars += char_clone.example_dialogue.len();
            }

            if include_first {
                let t = crate::models::count_tokens(&char_clone.first_message);
                total_tokens += t;
                total_chars += char_clone.first_message.len();
            }

            if include_name {
                let t = crate::models::count_tokens(&char_clone.name);
                total_tokens += t;
                total_chars += char_clone.name.len();
            }

            if include_title {
                let t = crate::models::count_tokens(&char_clone.char_title);
                total_tokens += t;
                total_chars += char_clone.char_title.len();
            }

            let _ = tx
                .send(UiEvent::TokenCountCalculated(
                    char_clone.id,
                    total_tokens,
                    total_chars,
                ))
                .await;
        });
    }
}
