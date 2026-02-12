#!/usr/bin/env bash
set -euo pipefail

TARGET="x86_64-pc-windows-gnu"
APP_NAME="simple-editor"
DIST_DIR="dist/windows"

rustup target add "$TARGET"

cargo build --release --target "$TARGET"

mkdir -p "$DIST_DIR"
cp "target/$TARGET/release/${APP_NAME}.exe" "$DIST_DIR/${APP_NAME}.exe"

if command -v zip >/dev/null 2>&1; then
  zip -j "$DIST_DIR/${APP_NAME}-windows.zip" "$DIST_DIR/${APP_NAME}.exe"
  echo "Packaged EXE and ZIP at: $DIST_DIR"
else
  echo "Packaged EXE at: $DIST_DIR/${APP_NAME}.exe"
  echo "zip command not found; skipping ZIP archive creation"
fi
