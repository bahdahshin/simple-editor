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

## Windows / Hyper-V compatibility note

This app now uses the `wgpu` renderer and forces the DirectX backend on Windows (`dx12,dx11`) to avoid OpenGL 2.0+ requirements in virtualized Windows environments (including Hyper-V VMs).
