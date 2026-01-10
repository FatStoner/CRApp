# Data Models

The application's data structures are defined in `src/models.rs`. They map directly to SQLite tables and intended application logic.

## Core Entities

### Character
Represents an AI character definition.
-   **Fields**:
    -   `id`: `i64` (Primary Key). `0` indicates a new, unsaved character.
    -   `name`: `String` (File Name / Display Name in list).
    -   `char_name`: `String` (Internal Name of the character).
    -   `char_title`: `String` (Subtitle/Role).
    -   `personality`: `String` (Description of personality).
    -   `scenario`: `String` (Context/Scenario).
    -   `first_message`: `String` (Greeting message).
    -   `example_dialogue`: `String` (Q&A examples).
    -   `avatar_path`: `Option<String>` (Path to local image file).
    -   `collection_id`: `Option<i64>` (Foreign Key to Collection).
    -   `app_tags`: `Vec<Tag>` (Internal organization tags).
    -   `external_tags`: `Vec<Tag>` (Tags imported from source, e.g., spicychat).

### Lorebook
Represents a collection of lore entries (World Info).
-   **Fields**:
    -   `id`: `i64`.
    -   `title`: `String`.
    -   `description`: `String`.
    -   `cover_path`: `Option<String>`.

### Collection
Represents a folder for organizing characters.
-   **Fields**:
    -   `id`: `i64`.
    -   `name`: `String`.
    -   `parent_id`: `Option<i64>` (Allows hierarchical folders).

### Tag
A simple label for filtering.
-   **Fields**:
    -   `id`: `i64`.
    -   `name`: `String`.

## Helper Enums

### AppMode
Defines the current main view state of the application.
-   `Characters`: Viewing character browser or editor.
-   `Lorebooks`: Viewing lorebook manager.
-   `Settings`: Application settings.
-   `DeepSearch`: Global search results.

### ThemeMode
-   `System`, `Light`, `Dark`.

## External Formats
### CharacterCardV2
Defined in `src/card_v2.rs`, this struct is used for exporting characters to a JSON format compatible with external tools (e.g., TavernAI, SpicyChat).
