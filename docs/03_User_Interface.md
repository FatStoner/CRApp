# User Interface Architecture

The UI is built using `egui` (Immediate Mode GUI). The rendering logic is primarily located in `src/ui/`.

## Module Structure

### 1. Central Panel (`ui/central_panel.rs`)
The `central_panel.rs` module acts as the orchestrator for the main content area.
-   **Responsibilities**:
    -   Determining which view to render based on `AppMode` and `CentralView` state.
    -   Hosting the "Import from Clipboard" modal.
    -   Delegating rendering to sub-modules.

### 2. Browser View (`ui/browser.rs`)
Handles the "Gallery" or "Browser" view where characters are listed.
-   **Features**:
    -   **Grid/List Display**: Renders characters as cards.
    -   **Filtering**: By collection, search term, and tags.
    -   **Sorting**: Alphabetical, Date, etc. (Ascending/Descending).
    -   **Navigation**: Folder traversal for collections.
    -   **Favorites View**: Dedicated filtered view for favorite characters.
    -   **Visual Indicators**: Red heart (\u2764) watermark on favorite character cards.
    -   **Context Menu**: Right-click actions (Move, Delete, Edit, Toggle Favorite).

### 3. Editor View (`ui/editor.rs`)
Handles the detailed editing interfaces.
-   **Character Editor**:
    -   **Fields**: Name, Title, Personality, Scenario, Examples, etc.
    -   **Avatars**: Image preview, clipboard pasting, file browsing.
    -   **Tags**: Management of App and External tags.
    -   **Export**: Export to .crapp (Native), .json (SpicyChat), .md, or .png (Card).
-   **Lorebook Editor**:
    -   Title, Description, and Cover management.

### 4. Parsing (`ui/parsing.rs`)
Dedicated logic for parsing character data from external text sources (clipboard).
-   **Function**: `parse_clipboard(text: &str) -> ParsedCharacterData`.
-   **Logic**: Heuristic parsing of unstructured text to extract Name, Persona, Scenario, etc.

### 5. Side Panel (`ui/side_panel.rs`)
The left-hand navigation bar.
-   **Features**:
    -   App Mode switching (Characters / Lorebooks / Settings).
    -   **Favorites**: Dedicated section for quickly accessing favorite characters. Displays a heart (\u2764) next to favorite character names.
    -   Collection Tree (recursive rendering of folders).
    -   Global Search input.
    -   **Deep Search**:
        -   **Query**: Full-text search across all character and lorebook fields.
        -   **Folder Filter**: Ability to limit search to a specific Collection and all its descendants.
        -   **Safe Snippets**: Heuristic snippet extraction with UTF-8 boundary protection and result limiting for performance.

## UI Event Loop & State
The `CrapApp` struct (in `ui/mod.rs`) holds all the transient UI state:
-   `active_char_tab`: Current tab in Editor (Main/Notes/Lorebooks).
-   `popup_state`: Handling of confirmation dialogs.
-   `is_saving`: Visual feedback during async operations.
-   `status_message`: Toast notifications.

### Borrow Checker Patterns
To manage ownership in the immediate mode loop, complex views (like `render_editor_view`) often use a "Take and Restore" pattern for `selected_character` to allow simultaneous mutable access to the character data and the rest of the generic app state (e.g., for accessing lists of collections or lorebooks).

## Keyboard Shortcuts
The application supports the following keyboard shortcuts for improved efficiency:
-   **Ctrl + S**: Save the current character (in Editor view).
-   **Enter**: Add a tag when the input field is focused (in Editor view).
-   **Esc**: Navigate back (Browser/Editor). Returns to previous view or parent folder. Triggers unsaved changes warning in Editor. Only works when no text field is focused.
