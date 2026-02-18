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
