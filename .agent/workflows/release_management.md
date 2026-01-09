---
description: Instructions for AI agents on how to manage releases, branches, and builds for CRApp.
---

# CRApp Release & Management Workflow

This document serves as a guide for AI agents interacting with the `CRApp` repository. It outlines the procedures for managing releases, handling branches, and building the project.

## 1. Branching Strategy

-   **Main Branch**: `main`
    -   Address of truth. All stable code resides here.
    -   Feature branches are merged into `main`.

## 2. Release Process (Local Automation)

Releases are now managed **locally** via the `deploy.sh` script, which uses the GitHub CLI (`gh`). GitHub Actions have been disabled.

### Triggering a Release

To publish a new stable version (updates the `latest` tag):

1.  **Ensure you are authenticated**: Run `gh auth status`. If not, run `gh auth login`.
2.  **Run the deployment script**:

```bash
./deploy.sh
```

### What `deploy.sh` Does
1.  **Builds** the project for Windows (`x86_64-pc-windows-gnu`).
2.  **Commits & Pushes** all current changes to `main`.
3.  **Deletes** the existing `latest` release on GitHub.
4.  **Creates** a new `latest` release and uploads `crap.exe`.

## 3. Local Development

### Build Command
To build the `.exe` locally on Linux:

```bash
cargo build --release --target x86_64-pc-windows-gnu
```

### Git Workflow
Since `deploy.sh` handles committing and pushing, you can simply work and then run `./deploy.sh` to save and publish. For intermediate saves:
1.  `git add .`
2.  `git commit -m "feat: description"`
3.  `./deploy.sh` (when ready to release)

## 4. Documentation

-   **README.md**: User-facing documentation.
-   **deploy.sh**: The source of truth for the release process.
