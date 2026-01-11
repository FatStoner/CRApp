CREATE TABLE IF NOT EXISTS lorebook_tags (
    lorebook_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (lorebook_id, tag_id),
    FOREIGN KEY (lorebook_id) REFERENCES lorebooks(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);
