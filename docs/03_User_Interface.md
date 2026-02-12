# User Interface Architecture

The UI is built using `egui` (Immediate Mode GUI). The rendering logic is primarily located in `src/ui/`.

## Module Structure

### 0. Core Logic (`ui/mod.rs`)
The `mod.rs` file serves as the main entry point for the UI module. It implements the `eframe::App` trait and orchestrates the overall update loop, delegating specific tasks to specialized modules.

### 0a. UI Event Loop (`ui/events.rs`)
Handles all asynchronous events arriving from the backend via a channel.
-   **Responsibilities**:
    -   Processing `UiEvent` variants (Loaded data, Saved confirmations, Errors).
    -   Updating the central application state (`CrapApp`) based on these events.
    -   Triggering UI repaints when new data arrives.

### 0b. Utility Functions (`ui/utils.rs`)
Houses shared logic unrelated to direct rendering.
-   **Key Functions**:
    -   `get_image_uri`: Standardized path-to-URI conversion with caching.
    -   `cleanup_avatar`: Logic for moving/renaming avatar files.

### 1. Central Panel (`ui/central_panel.rs`)
The `central_panel.rs` module acts as the orchestrator for the main content area.
-   **Responsibilities**:
    -   Determining which view to render based on `AppMode` and `CentralView` state.
    -   Hosting the consolidated "Import from Clipboard" modal with clear platform support.
    -   Delegating rendering to sub-modules.

### 1b. Options Window (`ui/options_window.rs`)
A dedicated modal window for application-wide settings.
-   **Features**:
    -   **Theme**: Toggle Light/Dark/System theme.
    -   **UI Scale**: Adjust global UI scaling.
    -   **Background**: Toggle custom background image and file picker integration.

### 2. Browser View (`ui/browser.rs`)
Handles the "Gallery" or "Browser" view where characters are listed.
-   **Features**:
    -   **Grid/List Display**: Renders characters and subfolders as cards.
        -   **Character Tags**: Displays both App and External tags in List View (separately) and up to 3 tags in Grid View.
        -   **Folder Images**: Custom square icons can be assigned to folders.
    -   **Filtering**: By collection, search term, and tags.
    -   **Sorting**: Alphabetical, Date, etc. (Ascending/Descending).
    -   **Navigation**: Folder traversal for collections.
    -   **Favorites View**: Dedicated filtered view for favorite characters.
    -   **Context Menu**: Right-click actions (Move, Delete, Edit, Toggle Favorite, **Change Icon**).
    -   **Clipboard Support**: Large "Paste from Clipboard" button supporting local files, web links, and raw image data (Windows/Linux compatible).
    -   **Background**: Supports a custom background image (configured in Options).

### 3. Editor View (`ui/editor.rs`)
Handles the detailed editing interfaces.
-   **Character Editor**:
    -   **Header**: Displays the character's internal name (`name` field) in parentheses next to the title for easy identification across tabs.
    -   **Fields**: Name, Title, Personality, Scenario, Examples, etc.
        -   **Context Menu**: Custom Cut/Copy/Paste actions with selection persistence.
    -   **Avatars**: Image preview, clipboard pasting, file browsing.
    -   **Tags**: Management of App and External tags.
    -   **Lorebooks**: Selection of linked lorebooks with "Go to Lorebook" navigation button.
    -   **Navigation**: "Back" and "Up" buttons with **Unsaved Changes Protection**.
        -   **History Context Menu**: Right-clicking the "Back" button opens a menu of the navigation stack, allowing jumping to specific past states.
        -   **Lorebook Entry Tracking**: The navigation history captures the specific Lorebook entry being viewed, including its ID and Name, ensuring accurate restoration even if data isn't fully loaded in the background.
    -   **Export**: Export to .crapp (Native), .json (SpicyChat), .md, or .png (Card).
    -   **Legacy Compat**: Supports importing V1 and V2 PNG cards.
-   **Lorebook Editor**:
    -   **Import**: "IMPORT" button allowing users to paste HTML source from SpicyChat (both Edit and Profile views) to automatically allow population of Title, Description, and Entries.
    -   **Metadata**: Title, Description (Content), Tags, and Cover management.
    -   **Tabbed Section**:
        -   **Entries Tab**: Master-Detail view for managing individual entries. Feature a swapped layout (Editor on Left, List on Right) and dynamic count badge.
        -   **Characters Tab**: Gallery view of all characters linked to this lorebook with dynamic count badge.
    -   **Sync**: Automatically synchronizes entry edits to the main lorebook object before saving or navigating, ensuring no data loss.
    -   **Dirty State Tracking**: Implements robust dirty state detection by comparing deep copies of the lorebook (including entries and tags) with the saved state.
        -   **Tag Reloading**: Reloads tags from the database immediately after saving to ensure ID consistency and prevent false "dirty" flags.
    -   **Safety**: Delete confirmation for lore entries.
    -   **Quick Search**: In-editor search bar that highlights matches and automatically jumps to matching entries.

### 4. Parsing (`ui/parsing/`)
Dedicated logic for parsing character data from external text sources (clipboard).
-   **Structure**: Split into format-specific modules (`janitor.rs`, `spicychat.rs`, `crave.rs`, etc.) for maintainability.
-   **Entry Point**: `ui/parsing/mod.rs` serves as a facade, exporting types and delegating detection to `detection.rs`.
-   **Function**: `parse_clipboard(text: &str) -> ParsedCharacterData`.
-   **Function**: `parse_spicychat_lorebook(html: &str) -> ParsedLorebookData`.
-   **Logic**:
    -   **Character**: Heuristic parsing of unstructured text to extract Name, Persona, Scenario, etc.
        -   **Supported Sources**:
            -   **JanitorAI**: Edit & Profile Pages (Text).
            -   **CraveU.ai**: Edit Page (Text).
            -   **GirlfriendGPT**: View Page (Text).
            -   **SpicyChat.ai**: Chatbots (Text) & Lorebooks (HTML/Text).
    -   **Lorebook**: Dual-mode HTML parsing (Edit View vs Profile View) with intelligent dispatching to handle source site structure variations.

### 5. Side Panel (`ui/side_panel.rs`)
The left-hand navigation bar.
-   **Features**:
    -   App Mode switching (Characters / Lorebooks / Settings).
    -   **Favorites**: Dedicated section for quickly accessing favorite characters. Displays a heart (\u2764) next to favorite character names.
    -   Collection Tree (recursive rendering of folders).
    -   Global Search input.
        -   Filters the character tree in real-time.
        -   **Matches**: Character Name, Title, Tags, and **Linked Lorebook Titles**.
        -   **Deep Search**:
        -   **Query**: Full-text search across all character and lorebook fields.
        -   **Lorebook Matching**: Searches Title, Description/Content, Tags, and all Entry fields.
        -   **Folder Filter**: Ability to limit search to a specific Collection and all its descendants.
        -   **Safe Snippets**: Contextual snippet extraction with UTF-8 boundary protection.
    -   **Lorebooks**:
        -   **Unified Aesthetic**: Lorebook rows are designed to match the character tree, with a height of **48.0px**.
        -   **Thumbnails**: Center-aligned **40.0px** thumbnails (or initials) are displayed for each entry.
        -   **Vertical Gaps**: Selection highlights are slightly shrunk (to **44.0px**) to ensure a clear 4.0px gap between items, preventing visual overlap.
        -   **Interaction**: Title labels are non-selectable and set to `Sense::hover()` to ensure smooth click-through for row selection.
        -   **Context Menu**: Right-click actions for quick deletion and management.
    -   **Smart Tab Switching**:
        -   The application tracks the `last_active_character_id` and `last_active_lorebook_id`.
        -   Clicking "Characters" or "Lorebooks" automatically restores the last active item (or the first available), avoiding "empty editor" states in both the UI and navigation history.


### 6. Reusable Components (`ui/widgets.rs`)
Shared UI elements and helper functions.
-   **Text Context Menu**: Custom right-click menu for `TextEdit` fields providing Cut, Copy, and Paste functionality.
    -   **Selection Persistence**: Implements a "sticky selection" mechanism using `egui` temporary data to ensure Cut/Copy operations work even when the text field loses focus due to the menu opening.
-   **Avatar Crop Rendering**: Unified logic for painting 1:1 cropped images with "Zoom/Cover" effects.
-   **Snippet Extraction**: Heuristic logic for extracting search result snippets from large text blocks.

-   `status_message`: Toast notifications.

### 6. Lightbox Viewer (`ui/lightbox.rs`)
A dedicated module for the high-priority image overlay.
-   **Features**:
    -   **Dynamic Interaction Zones**:
        -   **Background**: The viewer intelligently calculates the image's centered position. Clicking any "empty" space (the dimmed background) immediately closes the viewer.
        -   **Navigation**: High-priority left/right zones (15% screen width) allow cycling through the gallery context.
        -   **Image Protection**: The image itself consumes click events, preventing accidental closure when interacting with the main view.
    -   **Zoom and Pan**:
        -   **Controls**: Supports mouse scroll wheel for zooming and click-and-drag for panning.
        -   **On-Screen UI**: Overlay controls for zooming in/out, current zoom percentage, and a "Reset" button.
        -   **Constraints**: Zoom levels restricted between 10% (0.1x) and 500% (5.0x).
        -   **Navigation Reset**: Zoom and pan state are automatically reset when navigating to another image or closing the viewer.
    -   **Implementation**: Utilizes `egui::Image::load_for_size` and custom `gallery_zoom`/`gallery_pan` state in `CrapApp` to ensure accurate rect-to-screen mapping and smooth interactivity.

## UI Event Loop & State
The `CrapApp` struct (in `ui/app.rs` and `ui/types.rs`) holds all the transient UI state. The main update loop in `ui/mod.rs` calls `ui/events::handle_ui_events` at the start of every frame to process pending backend messages.
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
-   **Esc**: Navigate back. Acts identical to the **Back button**. Returns to previous view or parent folder using the navigation history. Triggers "Unsaved Changes" warning in Editor (Character/Lorebook). Only works when no text field is focused.
-   **Back / Up Buttons**: Utilize `request_back()` and `request_collection_switch()` to ensure state is saved before navigating.
    -   **Navigation History**: The application maintains a navigation stack, allowing the user to return to previous folders or views after deep searches or editor sessions.
    -   **Unsaved Changes**: If the editor detects changes (via boolean flags like `lorebook_has_changes`), the navigation is intercepted, and a confirmation popup is displayed.

## Performance & Optimization
To ensure a responsive UI, especially with large numbers of characters (thousands), several optimizations are implemented:

### 1. Asynchronous Image Loading
-   **Old Approach**: Blocking `std::fs` calls and synchronous image decoding on the main thread.
-   **Current Approach**: Leverages `egui`'s asynchronous image loader. Path resolution uses a custom helper `crate::ui::utils::get_image_uri` which:
    -   Converts paths to `file://` URIs efficiently.
    -   Caches the current working directory using `OnceLock` to avoid repeated system calls.

### 2. View Culling (Virtualization)
Renders and processes only the items currently visible in the viewport.
-   **Sidebar**: `render_tree` checks `ui.is_rect_visible(rect)` before painting avatars or text.
-   **Browser Grid**: Skips processing and rendering for cards outside the scroll area.
-   **Browser List**: Skips expensive avatar painting for non-visible rows.

### 3. Lightbox (Fullscreen Gallery Viewer)
-   **Features**:
    -   **Dynamic Interaction Zones**:
        -   **Background**: Clicking dimmed background closes the viewer.
        -   **Navigation**: Left/right 15% width zones for cycling.
        -   **Zoom/Pan**: Mouse wheel zoom (0.1x - 5.0x) and drag-to-pan.
        -   **UI Controls**: Percentage display, +/- buttons, and Reset.
-   **Implementation**: Accurate rect-to-screen mapping using `load_for_size` and transient state reset on navigation.
