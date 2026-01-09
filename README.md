# C.R.App (Character Repository Application)

**C.R.App** is a high-performance, local-first Desktop Character Manager built with **Rust**, **SQLite**, and **egui**. designed for power users who need distinct organization and deep integration for their character cards and lorebooks.

![Status](https://img.shields.io/badge/Status-Early_Beta-orange)
![License](https://img.shields.io/badge/License-MIT-blue)

## Overview

C.R.App ("CRAP" for short, but affectionately) moves away from web-based or Electron-heavy managers to provide a snappy, native experience. It leverages the power of a local SQLite database to handle thousands of characters instantly, with advanced search capabilities that go beyond simple filename matching.

## Key Features

-   **🚀 High Performance**: Built in Rust for blazing fast startup and low memory footprint.
-   **💾 Local-First Database**: All data is stored in a structured local SQLite database. No cloud dependencies.
-   **🔍 Deep Search**: Search through character names, descriptions, and tags instantly.
-   **📂 Nested Collections**: Organize your library with a robust, hierarchical folder system.
-   **🔄 Multi-Format Support**:
    -   **Import/Export**: Full support for `.crapp` (native), `.png` (embedded metadata), and JSON V2 standards.
-   **⚡ Native Windows Experience**: Optimized specifically for Windows desktop environments.

## Download

The latest version for Windows is available on the [Releases Page](https://github.com/JustJam-Dev/CRApp/releases/latest).

1.  Go to the **Releases** page.
2.  Expand the **Assets** section of the latest release.
3.  Download `crap.exe`.
4.  **Create a new folder** (e.g., `C.R.App`) and move `crap.exe` into it.
5.  Run it!

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

**Current Status: Early Beta**

The project is currently in active development. Features may change, and while the database schema is stable, always backup your `crap_data.db` before updating.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
