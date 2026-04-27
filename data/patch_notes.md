# Version 0.2.4 - Patch Notes (In Progress)

## Bugfixes
- **Folder Renaming**: Fixed a bug where renaming a subfolder would move it to the root and reset its display order.

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
