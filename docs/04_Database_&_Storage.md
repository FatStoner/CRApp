# Database & Storage

## Technology
-   **Database**: SQLite
-   **Driver**: `sqlx` (Rust) with `migrate` feature.
-   **Schema Management**: Versioned SQL migrations embedded in the binary.
-   **Safety**: Automatic startup backups.

## Migration System

The application uses `sqlx::migrate!` to manage database schema evolution.

-   **Location**: `migrations/` directory in the project root.
-   **Format**: Plain SQL files named with a timestamp prefix (e.g., `20260101000000_initial_schema.sql`).
-   **Behavior**:
    -   Migrations are compiled strictly into the `.exe`.
    -   On startup, the app checks the `_sqlx_migrations` table.
    -   Any missing migrations are applied atomically (inside a transaction).

### Adding a Migration
To modify the database (e.g., add a column):
1.  Create a new file in `migrations/`: `YYYYMMDDHHMMSS_description.sql`.
2.  Write the `ALTER TABLE` or `CREATE TABLE` statements.
3.  Run the app.

## Safety & Backups

To prevent data loss during updates, the application performs a **Safety Backup** during initialization (`src/db/mod.rs`):

1.  **Check**: Does `crap_data.db` exist?
2.  **Backup**: If yes, copy it to `crap_data.db.bak` immediately.
3.  **Migrate**: Only *after* the backup is secured does the migration runner start.

If a migration fails, the application will panic/crash to prevent partial data corruption, and the user can restore `crap_data.db.bak`.

## Schema

### Characters Table (`characters`)
Stores the main character data.
| Column | Type | Description |
|os | --- | --- |
| `id` | INTEGER PK | Auto-incrementing ID. |
| `name` | TEXT | Display name / File name. |
| `char_name` | TEXT | Internal character name. |
| `char_title` | TEXT | Subtitle. |
| `personality` | TEXT | |
| `scenario` | TEXT | |
| `example_dialogue` | TEXT | |
| `first_message` | TEXT | |
| `author_notes` | TEXT | |
| `avatar_path` | TEXT | Path to local file system. |
| `collection_id` | INTEGER FK | Links to `collections`. |
| `created_at` | DATETIME | |
| `updated_at` | DATETIME | |

### Character URLs Table (`character_urls`)
Stores multiple source links per character.
| Column | Type | Description |
| --- | --- | --- |
| `id` | INTEGER PK | |
| `character_id` | INTEGER FK | Links to `characters`. |
| `url` | TEXT | |
| `label` | TEXT | Optional service name. |

### Collections Table (`collections`)
Hierarchical folder structure.
| Column | Type | Description |
| --- | --- | --- |
| `id` | INTEGER PK | |
| `name` | TEXT | |
| `image_path` | TEXT | Path to custom icon file. |
| `parent_id` | INTEGER FK | Self-referencing FK for nesting. |

### Tags Tables
-   `tags`: Internal app tags.
-   `external_tags`: Tags imported from external sources.
-   `character_tags` / `character_external_tags`: Many-to-Many link tables.

### Lorebooks Table (`lorebooks`)
World info entries.
| Column | Type | Description |
| --- | --- | --- |
| `id` | INTEGER PK | |
| `title` | TEXT | |
| `description` | TEXT | Legacy column, synced with `content`. |
| `content` | TEXT | Main body text. |
| `cover_path` | TEXT | |

### Lorebook Entries Table (`lorebook_entries`)
Individual lore pieces within a book.
| Column | Type | Description |
| --- | --- | --- |
| `id` | INTEGER PK | |
| `lorebook_id` | INTEGER FK | Links to `lorebooks`. |
| `name` | TEXT | |
| `keywords` | TEXT | |
| `content` | TEXT | |

### Lorebook Tags
-   `lorebook_tags`: Links `lorebooks` to `tags`.

### Links
-   `character_lore_link`: Many-to-Many link between Characters and Lorebooks.

## File Storage
Non-text data is stored on the local filesystem, with paths stored in the database.
-   **Avatars**: Stored in `data/avatars/`.
-   **Collection Images**: Stored in `data/collection_images/`.
-   **Lorebook Covers**: Stored in `data/covers/`.
-   **Gallery**: Stored in `data/gallery/` (Reserved for character galleries, managed separately).
-   **Background**: Stored in `data/background/` (`default.png` and `custom.png`).
-   **Exports**: Saved to `exports/` (default dialog path).

## Automated Media Cleanup

To keep the storage clean, the application includes a **Media Cleanup** system (`src/cleaner.rs`).

-   **Trigger**: Runs automatically on every application startup.
-   **Logic**: 
    1. Scans `data/avatars/`, `data/collection_images/`, and `data/covers/`.
    2. Compares files on disk with paths stored in the SQLite database.
    3. Deletes any file found on disk that is no longer referenced in the database (orphaned files).
-   **Exclusions**: Specifically ignores `data/gallery/` to prevent data loss in managed galleries.
-   **Safety**: If an error occurs during cleanup, it is logged, but the application startup continues.

## Async Operations
All database operations are asynchronous (`async`/`await`). The UI thread spawns `tokio` tasks to perform DB writes or reads, preventing UI freezes. Results are communicated back via channels or by updating shared state wrapped in `Arc<Mutex>` (though `CrapApp` mostly reloads data after events).
