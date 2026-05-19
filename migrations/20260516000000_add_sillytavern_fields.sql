-- SillyTavern-specific fields, stored independently from main character data
ALTER TABLE characters ADD COLUMN st_name TEXT NOT NULL DEFAULT '';
ALTER TABLE characters ADD COLUMN st_description TEXT NOT NULL DEFAULT '';
ALTER TABLE characters ADD COLUMN st_personality TEXT NOT NULL DEFAULT '';
ALTER TABLE characters ADD COLUMN st_scenario TEXT NOT NULL DEFAULT '';
ALTER TABLE characters ADD COLUMN st_first_mes TEXT NOT NULL DEFAULT '';
ALTER TABLE characters ADD COLUMN st_mes_example TEXT NOT NULL DEFAULT '';
ALTER TABLE characters ADD COLUMN st_creator_notes TEXT NOT NULL DEFAULT '';
ALTER TABLE characters ADD COLUMN st_alternate_greetings_json TEXT;
ALTER TABLE characters ADD COLUMN st_creator TEXT NOT NULL DEFAULT '';
ALTER TABLE characters ADD COLUMN st_character_version TEXT NOT NULL DEFAULT '';
ALTER TABLE characters ADD COLUMN st_talkativeness REAL NOT NULL DEFAULT 0.5;
ALTER TABLE characters ADD COLUMN st_world TEXT NOT NULL DEFAULT '';
ALTER TABLE characters ADD COLUMN st_depth_prompt TEXT NOT NULL DEFAULT '';
ALTER TABLE characters ADD COLUMN st_depth_prompt_depth INTEGER NOT NULL DEFAULT 4;
ALTER TABLE characters ADD COLUMN st_depth_prompt_role TEXT NOT NULL DEFAULT 'system';
