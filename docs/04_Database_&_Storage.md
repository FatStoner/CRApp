# Database & Storage

## Technology
-   **Database**: SQLite
-   **Driver**: `rusqlite` (via `sqlx`)
-   **Schema Management**: Hardcoded `CREATE TABLE IF NOT EXISTS` queries in `db.rs` (init function).

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

### Collections Table (`collections`)
Hierarchical folder structure.
| Column | Type | Description |
| --- | --- | --- |
| `id` | INTEGER PK | |
| `name` | TEXT | |
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
| `description` | TEXT | |
| `cover_path` | TEXT | |

### Links
-   `character_lore_link`: Many-to-Many link between Characters and Lorebooks.

## File Storage
Non-text data is stored on the local filesystem, with paths stored in the database.
-   **Avatars**: Stored in `data/avatars/`.
-   **Exports**: Saved to `exports/` (default dialog path).

## Async Operations
All database operations are asynchronous (`async`/`await`). The UI thread spawns `tokio` tasks to perform DB writes or reads, preventing UI freezes. Results are communicated back via channels or by updating shared state wrapped in `Arc<Mutex>` (though `CrapApp` mostly reloads data after events).
