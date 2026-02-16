# Simple Rust Text Editor

A lightweight desktop text editor written in Rust with **Dioxus Desktop** and native file dialogs via `rfd`.

## What the app does

The current editor implementation provides:

- **New**: clears the current buffer and resets the active file path.
- **Open**: opens a native file picker and loads the selected text file.
- **Save**: writes to the current file, or opens a native save dialog if the file is new.
- **Live status messages**: shows operation results (ready/open/save/errors) in the toolbar.
- **Dynamic window title**: displays the current filename or `Untitled` when no file is selected.
- **Single-window desktop UI**: toolbar + full-height textarea, styled with in-app CSS.

> Note: this UI currently exposes file actions through toolbar buttons (not keyboard shortcuts).

## Tech stack

- Rust 2024 edition
- `dioxus` (desktop feature) for the desktop/webview UI
- `rfd` for native open/save dialogs

## Run locally

```bash
cargo run
```

## Build locally

```bash
cargo build --release
```

Output binary:

- Linux/macOS: `target/release/simple-editor`
- Windows: `target/release/simple-editor.exe`

## GitHub Actions workflow

The repository includes `.github/workflows/build-windows-exe.yml` with workflow name **Build Windows EXE**.

### Triggers

The workflow runs on:

- Manual dispatch (`workflow_dispatch`)
- Pull requests (`pull_request`)
- Pushes to branches: `main`, `master`, `work`
- Pushes of tags matching `v*`

### What it does

On `windows-latest`, it:

1. Checks out the repository.
2. Installs stable Rust.
3. Builds the project in release mode.
4. Uploads `target/release/simple-editor.exe` as an artifact named `simple-editor-windows-exe`.
5. For `v*` tags, creates/updates a **pre-release** and attaches the `.exe`.

## Creating a pre-release with the Windows executable

```bash
git tag v0.1.0-rc1
git push origin v0.1.0-rc1
```

Then open GitHub **Releases** and download `simple-editor.exe` from that tag's release assets after the workflow completes.
