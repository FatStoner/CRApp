use super::state::CrapApp;
use crate::ui::UiEvent;

impl CrapApp {
    /// Update character avatar from file (copies to data/avatars/)
    /// If character_id > 0, appends _{id} to filename.
    /// If character_id == 0, appends _{uuid} to filename.
    pub fn update_avatar_from_file(
        &self,
        source_path: std::path::PathBuf,
        character_id: i64,
    ) -> Option<String> {
        let dest_dir = std::path::Path::new("data/avatars");
        let _ = std::fs::create_dir_all(dest_dir);

        if let Some(file_stem) = source_path.file_stem().and_then(|s| s.to_str()) {
            if let Some(extension) = source_path.extension().and_then(|s| s.to_str()) {
                let new_filename = if character_id > 0 {
                    format!("{}_{}.{}", file_stem, character_id, extension)
                } else {
                    format!("{}_{}.{}", file_stem, uuid::Uuid::new_v4(), extension)
                };

                let dest = dest_dir.join(new_filename);
                if let Ok(_) = std::fs::copy(&source_path, &dest) {
                    return Some(dest.to_string_lossy().to_string());
                }
            }
        }
        None
    }

    /// Paste avatar from clipboard (saves to data/avatars/)
    pub fn paste_avatar_from_clipboard(&self, character_id: i64) -> Result<String, String> {
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => {
                // 1. Try raw image
                if let Ok(img_data) = clipboard.get_image() {
                    let width = img_data.width as u32;
                    let height = img_data.height as u32;
                    let bytes = img_data.bytes.into_owned();

                    if let Some(image_buffer) =
                        image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(
                            width, height, bytes,
                        )
                    {
                        let timestamp = chrono::Utc::now().timestamp_millis();
                        let filename = if character_id > 0 {
                            format!("pasted_avatar_{}_{}.png", timestamp, character_id)
                        } else {
                            format!("pasted_avatar_{}_{}.png", timestamp, uuid::Uuid::new_v4())
                        };
                        let dest_dir = std::path::Path::new("data/avatars");
                        let _ = std::fs::create_dir_all(dest_dir);
                        let dest_path = dest_dir.join(&filename);

                        if let Ok(_) = image_buffer.save(&dest_path) {
                            return Ok(dest_path.to_string_lossy().to_string());
                        }
                    }
                }

                // 2. Try text (file path) if image failed
                if let Ok(text) = clipboard.get_text() {
                    let clean_path = text.trim().trim_start_matches("file://");
                    let path = std::path::Path::new(clean_path);
                    if path.exists() && path.is_file() {
                        if let Ok(bytes) = std::fs::read(path) {
                            if let Ok(dynamic_img) = image::load_from_memory(&bytes) {
                                let timestamp = chrono::Utc::now().timestamp_millis();
                                let filename = if character_id > 0 {
                                    format!("pasted_avatar_{}_{}.png", timestamp, character_id)
                                } else {
                                    format!(
                                        "pasted_avatar_{}_{}.png",
                                        timestamp,
                                        uuid::Uuid::new_v4()
                                    )
                                };
                                let dest_dir = std::path::Path::new("data/avatars");
                                let _ = std::fs::create_dir_all(dest_dir);
                                let dest_path = dest_dir.join(&filename);

                                if let Ok(_) = dynamic_img.save(&dest_path) {
                                    return Ok(dest_path.to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                }

                Err("Clipboard does not contain image or valid path.".to_string())
            }
            Err(e) => Err(format!("Clipboard access failed: {}", e)),
        }
    }

    /// Add image to character gallery (async operation)
    pub fn add_gallery_image_async(&self, character_id: i64) {
        let tx = self.tx.clone();
        let ctx = self.ctx.clone();
        crate::task::spawn_supervised(ctx.clone(), async move {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("image", &["png", "jpg", "jpeg", "webp"])
                .pick_file()
            {
                let dest_dir = std::path::PathBuf::from(format!("data/gallery/{}", character_id));
                let _ = std::fs::create_dir_all(&dest_dir);
                if let Some(name) = path.file_name() {
                    let dest = dest_dir.join(name);
                    let _ = std::fs::copy(&path, &dest);
                    // Send specific event to clear cache for this new image
                    let _ = tx
                        .send(UiEvent::GalleryImageAdded(
                            dest.to_string_lossy().to_string(),
                        ))
                        .await;
                    let _ = tx.send(UiEvent::UiRepaint).await;
                }
            }
            ctx.request_repaint();
            Ok(())
        }, self.tx.clone());
    }

    /// Paste image to character gallery from clipboard
    pub fn paste_gallery_image_from_clipboard(&self, character_id: i64) -> Result<(), String> {
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => {
                // 1. Try raw image
                if let Ok(img_data) = clipboard.get_image() {
                    let width = img_data.width as u32;
                    let height = img_data.height as u32;
                    let bytes = img_data.bytes.into_owned();

                    if let Some(image_buffer) =
                        image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(
                            width, height, bytes,
                        )
                    {
                        let timestamp = chrono::Utc::now().timestamp_millis();
                        let filename = format!("pasted_{}.png", timestamp);
                        let gallery_dir = format!("data/gallery/{}", character_id);
                        let _ = std::fs::create_dir_all(&gallery_dir);
                        let dest_path = std::path::Path::new(&gallery_dir).join(&filename);

                        if let Ok(_) = image_buffer.save(&dest_path) {
                            let _ = self.tx.try_send(UiEvent::GalleryImageAdded(
                                dest_path.to_string_lossy().to_string(),
                            ));
                            return Ok(());
                        }
                    }
                }

                // 2. Try text (file path) if image failed
                if let Ok(text) = clipboard.get_text() {
                    let clean_path = text.trim().trim_start_matches("file://");
                    let path = std::path::Path::new(clean_path);
                    if path.exists() && path.is_file() {
                        if let Ok(bytes) = std::fs::read(path) {
                            if let Ok(dynamic_img) = image::load_from_memory(&bytes) {
                                let timestamp = chrono::Utc::now().timestamp_millis();
                                let filename = format!("pasted_{}.png", timestamp);
                                let gallery_dir = format!("data/gallery/{}", character_id);
                                let _ = std::fs::create_dir_all(&gallery_dir);
                                let dest_path = std::path::Path::new(&gallery_dir).join(&filename);

                                if let Ok(_) = dynamic_img.save(&dest_path) {
                                    let _ = self.tx.try_send(UiEvent::GalleryImageAdded(
                                        dest_path.to_string_lossy().to_string(),
                                    ));
                                    return Ok(());
                                }
                            }
                        }
                    }
                }

                Err("Clipboard does not contain image or valid path.".to_string())
            }
            Err(e) => Err(format!("Clipboard access failed: {}", e)),
        }
    }
    /// Load gallery images asynchronously with thumbnail generation
    pub fn load_gallery_images_async(&self, character_id: i64) {
        let tx = self.tx.clone();
        let ctx = self.ctx.clone();
        crate::task::spawn_supervised(ctx.clone(), async move {
            let gallery_dir = format!("data/gallery/{}", character_id);
            let thumb_dir = format!("data/.thumbnails/{}", character_id);
            let _ = std::fs::create_dir_all(&gallery_dir);
            let _ = std::fs::create_dir_all(&thumb_dir);

            let mut images = Vec::new();
            if let Ok(entries) = std::fs::read_dir(&gallery_dir) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_file() {
                            let path = entry.path();
                            if let Some(ext) = path
                                .extension()
                                .and_then(|e| e.to_str())
                                .map(|s| s.to_lowercase())
                            {
                                if ["png", "jpg", "jpeg", "webp"].contains(&ext.as_str()) {
                                    let filename = path.file_name().unwrap().to_string_lossy();
                                    let thumb_path = std::path::Path::new(&thumb_dir)
                                        .join(format!("{}.png", filename));

                                    let original_path_str = path.to_string_lossy().to_string();

                                    // Thumbnail logic
                                    let mut needs_thumb = true;
                                    if thumb_path.exists() {
                                        if let (Ok(orig_meta), Ok(thumb_meta)) = (
                                            std::fs::metadata(&path),
                                            std::fs::metadata(&thumb_path),
                                        ) {
                                            if let (Ok(orig_time), Ok(thumb_time)) =
                                                (orig_meta.modified(), thumb_meta.modified())
                                            {
                                                if thumb_time > orig_time {
                                                    needs_thumb = false;
                                                }
                                            }
                                        }
                                    }

                                    if needs_thumb {
                                        if let Ok(img) = image::open(&path) {
                                            // Resize to 300px max while preserving aspect ratio
                                            let thumb = img.thumbnail(300, 300);
                                            let _ = thumb.save(&thumb_path);
                                        }
                                    }

                                    let thumb_uri = crate::ui::utils::get_image_uri(
                                        &thumb_path.to_string_lossy(),
                                    );

                                    images.push(crate::ui::types::GalleryImage {
                                        path: original_path_str,
                                        thumbnail_uri: thumb_uri,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            images.sort_by(|a, b| a.path.cmp(&b.path));
            let _ = tx
                .send(UiEvent::GalleryImagesLoaded(character_id, images))
                .await;
            let _ = tx.send(UiEvent::UiRepaint).await;
            ctx.request_repaint();
            Ok(())
        }, self.tx.clone());
    }
}
