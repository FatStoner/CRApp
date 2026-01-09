---
description: Instructions for AI agents on how to manage releases, branches, and builds for CRApp.
---

# CRApp Release & Management Workflow

This document serves as a guide for AI agents interacting with the `CRApp` repository. It outlines the procedures for managing releases, handling branches, and building the project.

## 1. Branching Strategy

-   **Main Branch**: `main`
    -   Address of truth. All stable code resides here.
    -   Direct commits are allowed for small fixes/features suitable for a single session.
    -   For complex features, create short-lived feature branches, then merge to `main`.

## 2. Release Process

Releases are automated via GitHub Actions using the `.github/workflows/release.yml` workflow.

### Triggering a Release

To publish a new release:

1.  **Ensure `main` is stable** and all changes are committed.
2.  **Create a semantic version tag** (e.g., `v0.1.0`, `v1.2.3`).
3.  **Push the tag** to the remote repository.

```bash
# Verify status
git status

# Create tag
git tag v0.1.0

# Push tag to trigger CI/CD
git push origin v0.1.0
```

### What Happens Next (CI/CD)
-   The GitHub Action `Build & Release (Windows)` is triggered.
-   It spins up a `windows-latest` runner.
-   Compiles the project using `cargo build --release --target x86_64-pc-windows-msvc`.
-   Creates a **Draft Release** on GitHub.
-   Attaches the compiled `crap.exe` as an asset.

## 3. Local Development & Building

The project is configured for **cross-compilation from Linux to Windows** using MinGW.

### Build Command (Windows Target)
To build the `.exe` locally on Linux:

```bash
cargo build --release --target x86_64-pc-windows-gnu
```

-   **Output**: `target/x86_64-pc-windows-gnu/release/crap.exe`
-   **Note**: This uses the `gnu` toolchain (MinGW), whereas the GitHub Action uses `msvc`. They are compatible for end-users, but `gnu` is easier to set up on Linux.

### standard Git Workflow
1.  `git pull origin main` - Get latest changes.
2.  Make changes.
3.  `cargo check` / `cargo build` - Verify.
4.  `git add .`
5.  `git commit -m "feat: description"`
6.  `git push origin main`

## 4. Documentation

-   **README.md**: User-facing documentation. Update this when features change significantly.
-   **LICENSE**: MIT License.
