-- Enable Foreign Keys
PRAGMA foreign_keys = ON;

-- Characters Table
CREATE TABLE IF NOT EXISTS characters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    char_name TEXT NOT NULL,
    char_title TEXT NOT NULL,
    personality TEXT NOT NULL,
    scenario TEXT NOT NULL DEFAULT '',
    example_dialogue TEXT NOT NULL DEFAULT '',
    first_message TEXT NOT NULL,
    author_notes TEXT NOT NULL,
    avatar_path TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    collection_id INTEGER REFERENCES collections(id) ON DELETE SET NULL
);

-- Collections Table
CREATE TABLE IF NOT EXISTS collections (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    parent_id INTEGER REFERENCES collections(id) ON DELETE CASCADE
);

-- Tags Table (App)
CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

-- Character Tags Link (App)
CREATE TABLE IF NOT EXISTS character_tags (
    character_id INTEGER REFERENCES characters(id) ON DELETE CASCADE,
    tag_id INTEGER REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (character_id, tag_id)
);

-- External Tags Table
CREATE TABLE IF NOT EXISTS external_tags (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

-- Character External Tags Link
CREATE TABLE IF NOT EXISTS character_external_tags (
    character_id INTEGER REFERENCES characters(id) ON DELETE CASCADE,
    tag_id INTEGER REFERENCES external_tags(id) ON DELETE CASCADE,
    PRIMARY KEY (character_id, tag_id)
);

-- Lorebooks Table
CREATE TABLE IF NOT EXISTS lorebooks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    cover_path TEXT
);

-- Character Lore Link
CREATE TABLE IF NOT EXISTS character_lore_link (
    character_id INTEGER NOT NULL,
    lore_id INTEGER NOT NULL,
    PRIMARY KEY (character_id, lore_id),
    FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE,
    FOREIGN KEY (lore_id) REFERENCES lorebooks(id) ON DELETE CASCADE
);

-- Settings Table
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
