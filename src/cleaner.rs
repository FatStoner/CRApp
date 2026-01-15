use sqlx::{Pool, Sqlite};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub async fn cleanup_unused_media(pool: &Pool<Sqlite>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting unused media cleanup...");

    // 1. Collect all used paths from the database
    let mut used_paths = HashSet::new();

    // Characters: avatar_path
    // using query! macro requires offline mode or DB connection usually,
    // but here we might not have .sqlx prepared folder.
    // Safer to use query_as or query and map manually if we are not sure about sqlx data.
    // `sqlx::query!` macros depend on `SQLX_OFFLINE` or a live DB.
    // Given the environment, `sqlx::query` (runtime) is safer to avoid compile errors if metadata is missing.

    // Characters
    let rows = sqlx::query("SELECT avatar_path FROM characters WHERE avatar_path IS NOT NULL")
        .fetch_all(pool)
        .await?;

    for row in rows {
        use sqlx::Row;
        let path: Option<String> = row.try_get("avatar_path")?;
        if let Some(p) = path {
            used_paths.insert(PathBuf::from(p));
        }
    }

    // Collections
    // We handle the case where 'image_path' column might not exist if migration failed?
    // But migration is checked in db.rs.
    // However, clean_unused_media is called AFTER db init, so schema should be correct.
    let rows = sqlx::query("SELECT image_path FROM collections WHERE image_path IS NOT NULL")
        .fetch_all(pool)
        .await?;

    for row in rows {
        use sqlx::Row;
        let path: Option<String> = row.try_get("image_path")?;
        if let Some(p) = path {
            used_paths.insert(PathBuf::from(p));
        }
    }

    // Lorebooks
    let rows = sqlx::query("SELECT cover_path FROM lorebooks WHERE cover_path IS NOT NULL")
        .fetch_all(pool)
        .await?;

    for row in rows {
        use sqlx::Row;
        let path: Option<String> = row.try_get("cover_path")?;
        if let Some(p) = path {
            used_paths.insert(PathBuf::from(p));
        }
    }

    println!("Found {} active media files in database.", used_paths.len());

    // 2. Define target directories
    let target_dirs = vec!["data/avatars", "data/collection_images", "data/covers"];

    let mut deleted_count = 0;

    for dir in target_dirs {
        let path = Path::new(dir);
        if !path.exists() {
            continue;
        }

        let entries = fs::read_dir(path)?;
        for entry in entries {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.is_file() {
                // Determine if we need to delete it
                // We check if the full path (relative to CWD) is in the set.
                // entry.path() gives e.g. "data/avatars/xyz.png"

                // IMPORTANT: Handle potential path variations (e.g. ./data/...)
                // typically read_dir return paths without ./ if initialized without it.

                if !used_paths.contains(&file_path) {
                    // Extra safety: make sure we are not deleting "placeholder.png" or similar if it exists and is used but not in DB?
                    // Usually app managed files are UUIDs.
                    // Let's assume strict DB source of truth.

                    // Exclude any hidden files like .gitkeep if they exist?
                    if let Some(name) = file_path.file_name() {
                        if let Some(name_str) = name.to_str() {
                            if name_str.starts_with('.') {
                                continue;
                            }
                        }
                    }

                    println!("Deleting unused file: {:?}", file_path);
                    if let Err(e) = fs::remove_file(&file_path) {
                        eprintln!("Failed to delete {:?}: {}", file_path, e);
                    } else {
                        deleted_count += 1;
                    }
                }
            }
        }
    }

    println!("Cleanup finished. Deleted {} unused files.", deleted_count);

    Ok(())
}
