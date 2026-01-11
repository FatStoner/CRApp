CREATE TABLE IF NOT EXISTS lorebook_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    lorebook_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    keywords TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    FOREIGN KEY (lorebook_id) REFERENCES lorebooks(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_lorebook_entries_lorebook_id ON lorebook_entries(lorebook_id);
