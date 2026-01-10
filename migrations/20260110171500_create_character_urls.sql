-- Migration: Create character_urls table
CREATE TABLE IF NOT EXISTS character_urls (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    character_id INTEGER NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    label TEXT
);
