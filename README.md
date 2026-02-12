# Simple Rust Text Editor

A lightweight desktop text editor written in Rust using `egui/eframe`.

## Features

- New file
- Open file
- Save file
- Save file as...
- Dirty-state indicator (`*` in title)
- Status bar messages
- Monospace multiline editor area

## Run locally

```bash
cargo run
```

## Build release locally

```bash
cargo build --release
```

The release binary is created at:

- Linux/macOS: `target/release/simple-editor`
- Windows (native build): `target/release/simple-editor.exe`

## Download a Windows `.exe` from GitHub Actions

This repository includes a workflow at `.github/workflows/build-windows-exe.yml` that builds `simple-editor.exe` on `windows-latest` and uploads it as an artifact.

### How to get the `.exe`

1. Push your branch to GitHub.
2. Open the **Actions** tab.
3. Run **Build Windows EXE** (or use the run triggered by push/PR).
4. Open the workflow run and download the artifact named **`simple-editor-windows-exe`**.
5. Extract/download `simple-editor.exe` and run it on Windows.
