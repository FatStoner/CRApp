use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static CURRENT_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn cleanup_avatar(path_str: &str) {
    let path = Path::new(path_str);
    // Security check: Only delete if inside "data/avatars"
    // Normalize logic loosely by checking components or starts_with
    if path_str.replace("\\", "/").contains("data/avatars/") {
        if path.exists() {
            if let Err(e) = std::fs::remove_file(path) {
                eprintln!("Failed to delete old avatar {}: {}", path_str, e);
            } else {
                println!("Deleted old avatar: {}", path_str);
            }
        }
    }
}

pub fn get_image_uri(path: &str) -> String {
    if path.starts_with("file://") || path.contains("://") {
        return path.to_string();
    }

    // Check if absolute
    let path_obj = Path::new(path);
    if path_obj.is_absolute() {
        return format!("file://{}", path);
    }

    // Relative path: Resolve against cached current dir
    let cwd =
        CURRENT_DIR.get_or_init(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let abs = cwd.join(path);
    format!("file://{}", abs.to_string_lossy())
}
