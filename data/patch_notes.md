# Version 0.2.4 - Patch Notes

## Architectural Improvements
- **Asynchronous Threading & Task Supervision**: Refactored the core application initialization and background task execution. All async Tokio tasks are now managed by a robust supervisor (`spawn_supervised`) that intercepts both expected errors and unexpected panics, immediately notifying the UI thread and forcing instant canvas repaints without locking up.
- **Pre-GUI Gatekeeper & Crash Handler**: Implemented a strict startup initialization gatekeeper ensuring storage, logging, Tokio, and database subsystems boot cleanly before the GUI launches. Added a low-overhead panic hook dumping unhandled exceptions directly to `data/logs/crash.log` and exiting gracefully.
- **Database Corruption Resilience**: Enhanced database initialization with an automatic corruption isolation routine. If the SQLite database file or schema is malformed or corrupted, the system automatically isolates the corrupted file (`crap_data.corrupted.<timestamp>.db`) and boots a fresh database to prevent permanent lockout.
- **Improved Error Capture**: Integrated a professional logging system (`tracing`) that records application events and errors to `data/logs/`. Logs are rotated daily, and the application automatically cleans up old logs (keeping only the 5 most recent files).
- **Enhanced UI Feedback**: All background operations (CRUD actions, importing, exporting, settings) now propagate domain-specific errors (`AppError`, `DbError`) directly to the UI, displaying color-coded status toasts in real-time.

## UI & UX Improvements
- **Lorebook Layout Fixes**: Redesigned the Lorebook Entry editor to use a pinned layout. The "Save Entry" and "Delete" buttons are now anchored to the bottom of the screen, ensuring they are always accessible regardless of the content length or window scaling.
- **Granular Dirty State Tracking**: Lorebook entries now track unsaved changes individually.
    - Added visual indicators (`*`) in the entry list to show which entries have unsaved modifications.
    - Improved navigation: You can now switch between entries, add new ones, or paste entries from the clipboard without being interrupted by popups. Unsaved changes are preserved in memory and merged seamlessly.
- **Enhanced Logging**: Added comprehensive diagnostic logging (`tracing`) for all lorebook and character management operations, making it easier to troubleshoot background tasks.
- **Logging Visibility**: Background operations now provide better diagnostic information in case of failures.

## Bugfixes
- **Folder Renaming**: Fixed a bug where renaming a subfolder would move it to the root and reset its display order.
- **UI Scaling**: Fixed an issue where the Lorebook "Save Entry" button could be obscured when the content field was large.
- **Migration Stability**: Improved the robustness of the database migration system to better handle existing schemas.
- **Robust Path Sanitization**: Implemented a more comprehensive `sanitize_filename` system that automatically strips trailing spaces and periods, handles Windows reserved filenames (CON, PRN, etc.), and filters out control characters to improve filesystem compatibility across all platforms.
- **Export Diagnostic and Compatibility**: Enhanced mass-export operations with detailed logging and real-time UI error feedback for easier troubleshooting.
- **SillyTavern Export Compatibility**: Fixed compatibility issues when exporting lorebooks to SillyTavern. Corrected the `position` field format from string to integer mapping and added the required `enabled` boolean field to prevent SillyTavern from silently skipping exported entries.

# Version 0.2.3 - Patch Notes

## New functions

### Edit Character Section Improvements
- **Text Editor Improvements**: Remade text editor to enable customization options (font, font size, brightness) and better handling of selection and context menu. Optimized large file handling with culling and caching.
- **Quick Notes**: Added quick character notes.
- **Global Dictionary**: Implemented a global dictionary management system in settings for the spellchecker. You can also add your own words to the dictionary either through the settings or the context menu of the character editor.
- **NSFW Marking**: Added NSFW marking to characters. For now this only affects the blur system but it might be expanded in the future.
- **Blur/Unblur System**: Implemented a blur system. This include global blur setting, per-character blur setting and bluring characters marked as NSFW. You can also change blur state temporarily by right-clicking on the character image.

### Integration
- **SillyTavern/Chub.ai Lorebook Export**: Added support for lorebook exports compatible with SillyTavern, Chub.ai and afterhour.app.
- **Chub.ai Character Import**: Added clipboard import support for Chub.ai character edit pages.

## UI & UX Improvements
- **Embedded Patch Notes**: You can view these patch notes directly in the application settings.
- **Context Menu**: Added icons to some context menu options across the application.
- **Context Menu Sizing**: Refined context menu layout to prevent width issues and disabled text wrapping.
- **Performance**: Implemented asynchronous background thumbnailing and pre-calculated URIs for the character gallery to prevent freezes.

## Bugfixes
- **JPG Support**: Restored full support for JPG images.

# Version 0.2.2 - Patch Notes

## New functions

### Auto-Update System
Implemented a complete end-to-end auto-update system. This is the last update you need to download and swap .exe manually!

### Advanced Export Capabilities
- Added mass export for collections with multiple format selections.
- Added Export to one file option (Grid PNG, Detailed HTML list).
- Implemented context-aware export buttons for "All Characters" and "Favorites" views.

### New Importing from clipboard
- Added support for clipboard import from afterhour.app.

### Statistics and Token Counting
- Added a generalized statistics popup with token breakdowns for folders and current views.
- Implemented granular token counting settings.
- Added character and token counters specifically for the Lorebook view.

### Gallery Features
- Added Lightbox zoom and pan functionality to the image gallery.
- Implemented dynamic gallery navigation and clipboard support.

### UI & Navigation
- **Options Window**: Refactored into a tabbed interface (General, Tokens, Update, About) and made the window movable/fixed-size.
- **Sidebar**: Added "Unfold all" navigation option. (Right click on 'Uncategorized' section)
- **History**: Implemented browser-style navigation history and smart tab switching. Look under right click of back button.
- **Spell Check**: Added optional spell check with global settings and per-section overrides.
- **Avatar**: Added a context menu to the avatar image.
- **Background**: Added configurable background image scaling options.

## Minor changes/functions

### UX Improvements
- Gallery image deletion now requires confirmation.
- Lorebook entry deletion now automatically selects the nearest remaining entry.
- Added redirection to the Templates view if a user tries to apply a template but none exist.
- Added version number and GitHub link to the settings "About" section.
- Added ability to Copy and Paste entire Lorebook entries.

### Visual Changes
- Character tags are now displayed in the List View.
- Improved Character Editor UI layout and scrollbar behavior.

## Bugfixes
- Fixed gallery refresh bugs and image loading failures.
- Fixed character name import logic to correctly separate file name from display name.
- Fixed an issue where Character IDs were not preserved during import, causing duplicates.
- Fixed navigation history not properly saving Lorebook ID when navigating from the Lorebook Characters view.
- Ensured unique avatar filenames by automatically appending the Character ID preventing deleting avatars for multiple characters.
- Fixed navigation history not properly saving Lorebook ID when navigating from the Lorebook Characters view.
- Fixed issues with Character dirty state persistence (unsaved changes warning).
- Fixed Lorebook save logic consistency.
