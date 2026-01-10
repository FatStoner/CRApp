-- Migration: Add display_order to collections
-- We add the column with a default value of 0 initially.
ALTER TABLE collections ADD COLUMN display_order INTEGER NOT NULL DEFAULT 0;

-- Update existing rows to have display_order equal to their ID
-- This preserves the "creation time" sort order (assuming IDs increment)
UPDATE collections SET display_order = id;
