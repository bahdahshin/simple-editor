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

## Network/proxy troubleshooting for Cargo

If `cargo` fails to download crates (for example `CONNECT tunnel failed, response 403`), this is usually an environment/proxy restriction.

This repository includes `.cargo/config.toml` with safer defaults for proxied environments:

- `sparse` registry protocol
- `git-fetch-with-cli = true`
- HTTP/2 multiplexing disabled

If your environment still blocks external crates, configure an internal crates mirror and set Cargo to use it, e.g. with environment variables:

```bash
export CARGO_REGISTRIES_CRATES_IO_INDEX="sparse+https://<your-mirror>/index/"
export CARGO_REGISTRIES_CRATES_IO_PROTOCOL="sparse"
```

## Download Windows `.exe` from GitHub Releases

The workflow at `.github/workflows/build-windows-exe.yml` builds the app on `windows-latest`.

- For every tag matching `v*` (example: `v0.1.0`), it publishes `simple-editor.exe` directly to the corresponding GitHub Release.

### Create a release with `.exe`

1. Create and push a version tag:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
2. Wait for the **Build Windows EXE** workflow to finish.
3. Open GitHub **Releases** and download `simple-editor.exe` from the assets of tag `v0.1.0`.
