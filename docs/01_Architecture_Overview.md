# Architecture Overview

## Introduction
CRApp (Character Repository Application) is a desktop application built in Rust for managing AI character cards and lorebooks. It uses the `eframe` (egui framework) library for its Immediate Mode GUI and SQLite for local data persistence.

## High-Level Architecture
The application is structured into four main layers:

1.  **Entry Point (`main.rs`)**: Initializes the database, configures the window (eframe options), and launches the application loop.
2.  **Application State (`ui/mod.rs` - `CrapApp`)**: Holds the runtime state, including database connection, cached data (characters, collections), and UI state (active tab, selected items).
3.  **UI Layer (`ui/`)**: Handles rendering and user interaction.
    -   **Orchestration**: `central_panel.rs` directs the main content area.
    -   **Views**: `browser.rs` (Gallery), `editor.rs` (Edit specific items).
    -   **Navigation**: `side_panel.rs`.
    -   **Parsing**: `parsing.rs` handles imports from external formats.
4.  **Data Layer (`db.rs`, `models.rs`)**:
    -   **Models**: Rust structs representing entities (Character, Lorebook, Tag).
    -   **Database**: Async SQLite operations via `sqlx`.

## Application Loop
The application runs on the main thread using `eframe::run_native`. The `CrapApp` struct implements `eframe::App`, where the `update` method is called every frame to redraw the UI.

### State Management
-   **Persistence**: Data is stored in `crap_data.db` (SQLite).
-   **Runtime**: Data is loaded into `Vec<Character>` etc. in `CrapApp` for fast access during rendering. Major changes (Save/Delete) trigger database updates and a reload of runtime state.
-   **Communication**: `tokio::sync::mpsc` channels are used for async tasks (like file I/O or DB operations) to communicate back to the UI thread via `UiEvent`.

## Directory Structure
```
src/
├── main.rs         # Entry point
├── lib.rs          # (Implicit/Optional) Shared logic
├── db.rs           # Database connection and queries
├── models.rs       # Data structures
├── card_v2.rs      # Export/Import format compatibility
└── ui/             # User Interface Logic
    ├── mod.rs      # CrapApp struct and event definitions
    ├── main.rs     # (Not used, UI logic matches files below)
    ├── central_panel.rs  # Main content area router
    ├── side_panel.rs     # Left navigation bar
    ├── browser.rs        # Character Grid/List view
    ├── editor.rs         # Character/Lorebook Editor
    ├── parsing.rs        # Text parsing logic
    ├── global_search.rs  # Global Deep Search
    └── widgets.rs        # Reusable UI components
```
