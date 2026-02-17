# UI Architecture Documentation

## Overview

The CRApp UI follows an **MVC-like pattern** with clear separation of concerns:
- **Views** - Pure UI rendering logic
- **Controllers** - Business logic and async operations
- **Components** - Reusable UI widgets
- **Panels** - Structural UI sections

**Total UI Codebase**: ~12,268 lines of code across modular architecture

---

## Directory Structure

```
src/ui/
├── components/          # Reusable UI components
│   └── popups/         # Categorized popup dialogs (827 LOC)
│       ├── mod.rs      # Popup dispatcher (94 LOC)
│       ├── deletion.rs # Delete confirmations (156 LOC)
│       ├── editing.rs  # Rename/unsaved changes (160 LOC)
│       ├── import_export.rs # DB import/export (265 LOC)
│       └── templates.rs # Template selection (152 LOC)
│
├── controllers/         # Business logic layer
│   ├── mod.rs          # Controller exports
│   ├── character_actions.rs # Character CRUD operations
│   ├── collection_actions.rs # Collection management
│   ├── import_export.rs # Import/export logic
│   ├── lorebook_actions.rs # Lorebook operations
│   └── template_actions.rs # Template operations
│
├── panels/             # Structural UI sections
│   └── side/          # Side panel (1,027 LOC)
│       ├── mod.rs     # Main panel logic (557 LOC)
│       └── tree.rs    # Tree rendering & search (470 LOC)
│
├── parsing/            # Data parsing utilities
│   ├── mod.rs
│   ├── chara.rs       # Character card parsing
│   ├── lorebook.rs    # Lorebook parsing
│   └── tavern.rs      # TavernAI format
│
├── views/              # Main view rendering
│   ├── browser/       # Character browser (1,208 LOC)
│   │   ├── mod.rs     # Main browser view (736 LOC)
│   │   ├── character_card.rs # Character cards (241 LOC)
│   │   └── collection_card.rs # Folder cards (231 LOC)
│   │
│   ├── editor/        # Content editors (2,323 LOC)
│   │   ├── character/ # Character editor sub-modules
│   │   │   ├── mod.rs
│   │   │   ├── main_data.rs
│   │   │   ├── notes.rs
│   │   │   ├── lorebooks.rs
│   │   │   └── gallery.rs
│   │   ├── lorebook/  # Lorebook editor sub-modules
│   │   │   ├── mod.rs
│   │   │   ├── metadata.rs
│   │   │   ├── entries.rs
│   │   │   └── characters.rs
│   │   ├── mod.rs     # Editor exports
│   │   ├── template.rs # Template editor
│   │   ├── toolbar.rs  # Editor toolbar
│   │   └── export.rs   # Export dialog
│   │
│   ├── search.rs      # Deep search view (490 LOC)
│   ├── settings.rs    # Settings window (183 LOC)
│   └── mod.rs         # View exports (8 LOC)
│
├── widgets.rs          # Custom UI widgets
├── utils.rs            # UI utilities
├── types.rs            # UI type definitions
└── mod.rs              # Main UI module
```

---

## Architecture Principles

### 1. **MVC-like Separation**

**Views** (Pure Rendering)
- Located in `views/` and `panels/`
- Only handle UI rendering and user input
- No direct database access or file I/O
- Call controller methods for business logic

**Controllers** (Business Logic)
- Located in `controllers/`
- Handle async operations (DB, file I/O)
- Trigger UI events via `UiEvent` enum
- No direct UI rendering

**Components** (Reusable Widgets)
- Located in `components/`
- Self-contained UI elements
- Can be used across multiple views

### 2. **Modular Organization**

Large files (>1000 LOC) are split into focused modules:

**Browser View** (was 1,222 LOC → now 3 files)
- `browser/mod.rs` - Main view logic, actions, helpers
- `browser/character_card.rs` - Character grid/list rendering
- `browser/collection_card.rs` - Folder grid/list rendering

**Side Panel** (was 1,023 LOC → now 2 files)
- `side/mod.rs` - Panel structure, mode switching
- `side/tree.rs` - Tree rendering, search, drag-drop

**Popups** (was 827 LOC → now 5 files)
- `popups/mod.rs` - Dispatcher routing
- `popups/deletion.rs` - Delete confirmations
- `popups/editing.rs` - Rename/unsaved changes
- `popups/import_export.rs` - Import/export dialogs
- `popups/templates.rs` - Template selection

**Editor Views** (was ~1,600 LOC → now 10+ files)
- `editor/character/` - Modular character editor (5 modules)
- `editor/lorebook/` - Modular lorebook editor (4 modules)

### 3. **Clear Responsibilities**

| Module | Responsibility | Size |
|--------|---------------|------|
| `views/browser/` | Character/collection browsing | 1,208 LOC |
| `views/editor/` | Content editing (char/lore/template) | 2,323 LOC |
| `panels/side/` | Navigation tree, mode switching | 1,027 LOC |
| `components/popups/` | Modal dialogs | 827 LOC |
| `controllers/` | Business logic, async ops, Mass Export | ~500 LOC |
| `parsing/` | Data format parsing | ~400 LOC |

---

## Key Design Patterns

### Action-Based UI Updates

Views emit **action enums** that are processed in the main update loop:

```rust
// Example: Browser actions
pub enum BrowserAction {
    OpenCharacter(i64),
    MoveCharacter(i64, Option<i64>),
    DeleteCharacter(i64),
    // ...
}

// Views collect actions
let mut actions = Vec::new();
render_character_card(ui, app, char, &mut actions);

// Main loop processes actions
for action in actions {
    match action {
        BrowserAction::OpenCharacter(id) => app.load_character(id),
        // ...
    }
}
```

### Event-Driven Architecture

Controllers trigger UI updates via events:

```rust
// Controller triggers event
tx.send(UiEvent::CharacterSaved(char_id)).ok();

// UI handles event
match event {
    UiEvent::CharacterSaved(id) => {
        app.refresh_character(id);
    }
}
```

### Module Re-exports

Parent modules re-export child functionality:

```rust
// browser/mod.rs
mod character_card;
mod collection_card;

pub use character_card::render_character_card;
pub use collection_card::render_subfolder_card;
```

---

## File Size Guidelines

**Target**: Keep files under 800 lines for maintainability

**Current Status**:
- ✅ All `components/` files < 300 LOC
- ✅ All `browser/` files < 750 LOC
- ✅ All `side/` files < 600 LOC
- ✅ `editor/character/` sub-modules < 700 LOC
- ✅ `editor/lorebook/` sub-modules < 350 LOC

---

## Future Refactoring Opportunities

1. **Search View** - Extract search result rendering (currently 490 LOC)
2. **Settings View** - Split into category-specific modules if expanded
3. **Controllers** - Add more granular controllers as features grow

---

## Testing Strategy

**Smoke Testing**:
1. Navigate through all views (browser, editor, search, settings)
2. Test character/lorebook CRUD operations
3. Verify popup dialogs (delete, rename, import/export)
4. Test side panel tree navigation and search

**Integration Points**:
- View ↔ Controller communication via actions/events
- Controller ↔ Database via async operations
- UI ↔ File system via import/export

---

### 4. **High-Performance Code Editor**

The `CodeEditor` (wrapper around `cosmic-text`) is optimized for large files (10MB+) by minimizing per-frame operations:

- **Visibility Culling**: Only lines visible on screen are processed for spell-checking and decoration.
- **Zero-Allocation Sync**: Uses a `Length + Hashing` heuristic for change detection. Full hashing is skipped if file length is identical to the last frame.
- **Shared Memory**: Spell-check results and line offsets are stored in `Arc` containers to avoid per-frame `Vec` cloning.
- **State Persistence**: Uses `egui::Memory` to track font changes, brightness, and search queries, preventing expensive re-layouts unless necessary.

---

### 5. **Asynchronous Asset Management**

The Character Gallery uses a multi-stage loading pipeline to maintain 60fps responsiveness during scrolling:

- **Background Scanning**: Directory enumeration and file filtering are offloaded to background threads.
- **On-Demand Thumbnailing**: High-resolution images are automatically downscaled (300px max) on the first load and cached in `data/.thumbnails`.
- **Pre-calculated URIs**: `file://` URIs are generated in background threads, eliminating per-frame string allocations and path resolutions.
- **Reference-Counted Caching**: Gallery data is stored in `Arc<Vec<GalleryImage>>`, allowing zero-copy sharing between UI and background tasks.

---

## Benefits of Current Architecture

✅ **Maintainability** - Small, focused files are easier to understand and modify
✅ **Testability** - Clear separation allows unit testing of controllers
✅ **Scalability** - Easy to add new views, controllers, or components
✅ **Readability** - Logical organization makes codebase navigation intuitive
✅ **Performance** - Heavy assets and large files are handled via culling, hashing, and background threading.

---

*Last Updated: 2026-02-17*
*Total UI LOC: 13,500+*
*Modules: 45+ files across 12 directories*
