# Simple CPU Text Editor

A lightweight desktop text editor written in Rust.

This app intentionally uses a CPU-only rendering path built from:

- `egui` for editor UI/widgets and text editing behavior
- `winit` for windowing and input events
- `softbuffer` for presenting a software framebuffer
- `tiny-skia` for the CPU render target

## Features

- Multi-line text editing
- New/Open/Save/Save As actions
- Keyboard shortcuts (`Ctrl+N`, `Ctrl+O`, `Ctrl+S`)
- File status feedback and dirty indicator in the window title

## Run

```bash
cargo run
```

## Build

```bash
cargo build --release
```

The release binary is created at:

- Linux/macOS: `target/release/simple-editor`
- Windows (native build): `target/release/simple-editor.exe`

## Download Windows `.exe` from GitHub pre-releases

The workflow at `.github/workflows/build-windows-exe.yml` builds the app on `windows-latest`.

- For every tag matching `v*` (example: `v0.1.0-rc1`), it creates a **pre-release** and attaches `simple-editor.exe` to that release.
- It also uploads the `.exe` as an Actions artifact (`simple-editor-windows-exe`) for manual runs, branch pushes, and PR builds.

### Create a pre-release with `.exe`

1. Create and push a version tag:
   ```bash
   git tag v0.1.0-rc1
   git push origin v0.1.0-rc1
   ```
2. Wait for the **Build Windows EXE** workflow to finish.
3. Open GitHub **Releases** and download `simple-editor.exe` from the assets of tag `v0.1.0-rc1`.
