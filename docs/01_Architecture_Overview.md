# Architecture Overview

## Introduction
CRApp (Character Repository Application) is a desktop application built in Rust for managing AI character cards and lorebooks. It uses the `eframe` (egui framework) library for its Immediate Mode GUI and SQLite for local data persistence.

## High-Level Architecture
The application is structured into four main layers:

1.  **Entry Point (`main.rs`)**: Initializes the database, configures the window (eframe options), and launches the application loop.
2.  **Application State (`ui/app.rs`, `ui/types.rs`)**: Defines `CrapApp` and associated enums/structs. Holds runtime state, database connections, and UI flags.
3.  **UI Layer (`ui/`)**: Specialized modules for rendering and logic.
    -   **Event Handling**: `events.rs` manages the async event loop.
    -   **Orchestration**: `mod.rs` (App trait) and `central_panel.rs` (Content routing).
    -   **Views**: `browser.rs` (Gallery), `editor/` (Specific item editors), `options_window.rs`.
    -   **Navigation**: `side_panel.rs`.
    -   **Overlays**: `lightbox.rs` (Image viewer).
    -   **Utilities**: `utils.rs` (Shared logic).
    -   **Parsing**: `parsing/` module handles format-specific logic.
4.  **Data Layer (`db.rs`, `models.rs`)**:
    -   **Models**: Rust structs representing entities (Character, Lorebook, Tag).
    -   **Database**: Async SQLite operations via `sqlx`.

## Application Loop
The application runs on the main thread using `eframe::run_native`. The `CrapApp` struct implements `eframe::App`, where the `update` method is called every frame to redraw the UI.

### State Management
-   **Persistence**: Data is stored in `crap_data.db` (SQLite).
-   **Runtime**: Data is loaded into `Vec<Character>` etc. in `CrapApp` for fast access during rendering. Major changes (Save/Delete) trigger database updates and a reload of runtime state.
-   **Navigation History**: A built-in history stack (in `CrapApp`) tracks user movement between folders and app modes, enabling consistent "Back" button behavior.
-   **Event Loop**: `ui/events.rs` processes backend messages at the start of each frame, decoupling UI rendering from async data updates.

## Directory Structure
```
src/
├── main.rs         # Entry point
├── lib.rs          # (Implicit/Optional) Shared logic
├── db.rs           # Database connection and queries
├── cleaner.rs      # Unused media cleanup logic
├── models.rs       # Data structures
├── card_v2.rs      # Export/Import format compatibility
└── ui/             # User Interface Layer
    ├── mod.rs      # Main entry point and App trait implementation
    ├── app.rs      # CrapApp struct definition
    ├── types.rs    # UI-specific enums and data types
    ├── events.rs   # Async event loop handler
    ├── utils.rs    # Shared utility functions (get_image_uri, etc.)
    ├── lightbox.rs # Fullscreen image viewer logic
    ├── central_panel.rs  # Main content area router
    ├── side_panel.rs     # Left navigation bar (with culling)
    ├── browser.rs        # Character Grid/List view (with culling)
    ├── editor/           # Sub-modules for editors (Character, Lorebook)
    ├── parsing/    # Modular text and HTML parsing logic
    │   ├── mod.rs  # Entry point and format detection
    │   └── ...     # Format-specific modules (janitor, spicychat, etc.)
    ├── global_search.rs  # Global Deep Search
    ├── popups.rs         # Unified modal handling
    ├── text_highlight.rs # Search highlight logic
    └── widgets.rs        # Reusable UI components
```
