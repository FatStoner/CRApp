# CRApp (Character Repository Application)

**CRApp** is a high-performance, local-first Desktop Character Manager built with **Rust**, **SQLite**, and **egui**, designed for power users who need distinct organization and deep integration for their character cards and lorebooks.

![Status](https://img.shields.io/badge/Status-Beta-orange)
![License](https://img.shields.io/badge/License-MIT-blue)

## Overview

C.R.App leverages the power of a local SQLite database to handle thousands of characters instantly, with advanced search capabilities that go beyond simple filename matching.

## Key Features

-   **🚀 High Performance**: Built in Rust for blazing fast startup and low memory footprint.
-   **💾 Local-First Database**: All data is stored in a structured local SQLite database.
-   **🔍 Deep Search**: Search through character names, descriptions, and tags instantly.
-   **📖 Lorebooks & World Info**: Create and link extensive lorebooks to your characters.
-   **📂 Nested Collections**: Organize your library with a robust, hierarchical folder system.
-   **🔄 Import/Export**: Full support for TavernAI V2 cards (PNG) and JSON. Quick clipboard importing from **SpicyChat**, **JanitorAI**, **Chub.ai**, **AfterHour.app**, **GirlfriendGPT**, and **CraveU**. **Lorebook export** compatible with **SillyTavern** and **Chub.ai**.
-   **🧹 Auto-Cleanup**: Automatically removes unused images to keep your storage efficient.
-   **✨ Auto-Updates**: Stay up to date automatically with the built-in update system.

## Shortcuts

-   **Ctrl + S**: Save changes.
-   **Ctrl + F**: Quick search.
-   **Esc**: Go back / Cancel.

## Download

The latest version for Windows is available on the [Releases Page](https://github.com/JustJam-Dev/CRApp/releases/latest).

1.  Go to the **Releases** page.
2.  Expand the **Assets** section of the latest release.
3.  Download the ZIP archive for your platform (e.g., `CRApp-x86_64-pc-windows-gnu.zip`).
4.  **Extract the contents** to a folder.
5.  Run `crap.exe`!

> [!NOTE]
> **Subsequent Updates are Automatic!**
> Once installed, CRApp will automatically check for and install updates in the background whenever you launch it.

> [!TIP]
> **Why a separate folder?**
> The application creates a local database file (`crap.exe`) and other data files in the same directory where it's run. Keeping it in its own folder ensures your desktop or downloads folder stays clean!

## Building from Source

### For Developers

You can install the latest development version directly from the repository using Cargo:

```bash
cargo install --git https://github.com/JustJam-Dev/CRApp
```

*Note: You will need the Rust toolchain and SQLite development headers installed.*

## Project Status

**Current Status: Beta**

The project is currently in active development. Features may change.

> [!NOTE]
> The application **automatically creates a backup** (`crap_data.db.bak`) of your database every time it starts, before applying any updates. However, for critical data, manual backups are always recommended.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
