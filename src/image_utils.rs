use image::GenericImageView;
use std::path::Path;

#[allow(dead_code)]
pub fn process_collection_image(path: &str) -> Option<String> {
    let img = image::open(path).ok()?;
    let (w, h) = img.dimensions();
    let size = w.min(h);

    // Crop 1:1 from top-left
    let cropped = img.crop_imm(0, 0, size, size);

    let final_img = if size > 512 {
        cropped.resize(512, 512, image::imageops::FilterType::Lanczos3)
    } else {
        cropped
    };

    let uuid = uuid::Uuid::new_v4();
    let save_path = format!("data/collection_images/{}.png", uuid);

    if let Some(parent) = Path::new(&save_path).parent() {
        std::fs::create_dir_all(parent).ok()?;
    }

    final_img.save(&save_path).ok()?;

    Some(save_path)
}

#[allow(dead_code)]
pub fn process_clipboard_image() -> Option<String> {
    let mut clipboard = arboard::Clipboard::new().ok()?;
    let image_data = clipboard.get_image().ok()?;

    let img = image::RgbaImage::from_raw(
        image_data.width as u32,
        image_data.height as u32,
        image_data.bytes.into_owned(),
    )?;

    let dyn_img = image::DynamicImage::ImageRgba8(img);
    let (w, h) = dyn_img.dimensions();
    let size = w.min(h);

    // Crop 1:1
    let cropped = dyn_img.crop_imm(0, 0, size, size);

    let final_img = if size > 512 {
        cropped.resize(512, 512, image::imageops::FilterType::Lanczos3)
    } else {
        cropped
    };

    let uuid = uuid::Uuid::new_v4();
    let save_path = format!("data/collection_images/{}.png", uuid);

    if let Some(parent) = Path::new(&save_path).parent() {
        std::fs::create_dir_all(parent).ok()?;
    }

    final_img.save(&save_path).ok()?;

    Some(save_path)
}
