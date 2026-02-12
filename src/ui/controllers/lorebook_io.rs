use super::state::CrapApp;
use crate::models::Lorebook;
use eframe::egui;

impl CrapApp {
    /// Export lorebook to .crappbook file (JSON format)
    pub fn export_lorebook(&self, lorebook: &Lorebook) -> Option<(String, egui::Color32)> {
        let export_data = crate::ui::parsing::ParsedLorebookData {
            title: lorebook.title.clone(),
            description: lorebook.description.clone(),
            tags: lorebook.tags.iter().map(|t| t.name.clone()).collect(),
            entries: lorebook
                .entries
                .iter()
                .map(|e| crate::ui::parsing::ParsedLorebookEntry {
                    name: e.name.clone(),
                    keywords: e
                        .keywords
                        .split(',')
                        .map(|k| k.trim().to_string())
                        .filter(|k| !k.is_empty())
                        .collect(),
                    content: e.content.clone(),
                })
                .collect(),
        };

        if let Ok(json) = serde_json::to_string_pretty(&export_data) {
            let safe_title: String = lorebook
                .title
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
                .collect();
            let filename = format!("{}.crappbook", safe_title.replace(" ", "_"));

            if let Some(path) = rfd::FileDialog::new()
                .set_file_name(&filename)
                .add_filter("Crappbook", &["crappbook", "json"])
                .save_file()
            {
                if let Err(e) = std::fs::write(&path, json) {
                    return Some((format!("Export failed: {}", e), egui::Color32::RED));
                } else {
                    return Some(("Export successful!".to_string(), egui::Color32::GREEN));
                }
            }
        } else {
            return Some(("Serialization failed!".to_string(), egui::Color32::RED));
        }
        None
    }

    /// Update lorebook cover image from file (copies file to data/covers/)
    pub fn update_lorebook_cover(
        &self,
        _lorebook_id: i64,
        source_path: std::path::PathBuf,
    ) -> Option<String> {
        let dest_dir = std::path::Path::new("data/covers");
        let _ = std::fs::create_dir_all(dest_dir);
        if let Some(name) = source_path.file_name() {
            let dest = dest_dir.join(name);
            if let Ok(_) = std::fs::copy(&source_path, &dest) {
                return Some(dest.to_string_lossy().to_string());
            }
        }
        None
    }
}
