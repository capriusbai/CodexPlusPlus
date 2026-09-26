#!/usr/bin/env bash
# Regenerate every platform icon from the single brand master.
#
#   assets/brand/aethercodex-mark.svg   preferred master (vector)
#   assets/brand/aethercodex-mark.png   raster master, >= 1024x1024
#
# To adopt the approved Archai master: replace the file(s) above and rerun this
# script. Nothing else in the tree hardcodes an icon.
#
# Usage: generate-icons.sh [--check]
#   --check  verify the generated icons are in sync with the master and exit
#            non-zero if not (used by CI).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BRAND_DIR="$ROOT/assets/brand"
SVG_MASTER="$BRAND_DIR/aethercodex-mark.svg"
PNG_MASTER="$BRAND_DIR/aethercodex-mark.png"

# Every icon the build consumes, regenerated from the master.
APP_ICON_DIR="$ROOT/apps/aethercodex-manager/src-tauri/icons"
ASSET_ICON_DIR="$ROOT/assets/images"
DOC_ICON_DIR="$ROOT/docs/images"

CHECK_ONLY=0
[ "${1:-}" = "--check" ] && CHECK_ONLY=1

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "error: $1 is required (apt install $2)" >&2
    exit 1
  }
}

# Render the master at an exact square size into $2.
#
# Output is always 32-bit RGBA: `tauri::generate_context!` rejects an icon
# without an alpha channel, and a fully opaque mark otherwise renders as RGB.
render() {
  local size="$1" out="$2"
  need convert imagemagick
  if [ -f "$SVG_MASTER" ] && command -v rsvg-convert >/dev/null 2>&1; then
    rsvg-convert -w "$size" -h "$size" "$SVG_MASTER" -o "$out.raw.png"
  elif [ -f "$PNG_MASTER" ]; then
    # `!` forces the exact size even if the master is not square.
    convert "$PNG_MASTER" -resize "${size}x${size}!" "$out.raw.png"
  else
    echo "error: no brand master found at $SVG_MASTER or $PNG_MASTER" >&2
    exit 1
  fi
  convert "$out.raw.png" -alpha on -define png:color-type=6 "PNG32:$out"
  rm -f "$out.raw.png"
}

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

# --- raster masters and the sizes each platform wants ---------------------
for size in 16 32 48 64 128 256 512 1024; do
  render "$size" "$STAGE/icon-$size.png"
done

# --- Windows .ico ---------------------------------------------------------
need icotool icoutils
icotool -c -o "$STAGE/icon.ico" \
  "$STAGE/icon-16.png" "$STAGE/icon-32.png" "$STAGE/icon-48.png" \
  "$STAGE/icon-64.png" "$STAGE/icon-128.png" "$STAGE/icon-256.png"

# --- macOS .icns ----------------------------------------------------------
# iconutil only exists on macOS; png2icns covers the Linux/CI case.
if command -v iconutil >/dev/null 2>&1; then
  ICONSET="$STAGE/AetherCodex.iconset"
  mkdir -p "$ICONSET"
  cp "$STAGE/icon-16.png"   "$ICONSET/icon_16x16.png"
  cp "$STAGE/icon-32.png"   "$ICONSET/icon_16x16@2x.png"
  cp "$STAGE/icon-32.png"   "$ICONSET/icon_32x32.png"
  cp "$STAGE/icon-64.png"   "$ICONSET/icon_32x32@2x.png"
  cp "$STAGE/icon-128.png"  "$ICONSET/icon_128x128.png"
  cp "$STAGE/icon-256.png"  "$ICONSET/icon_128x128@2x.png"
  cp "$STAGE/icon-256.png"  "$ICONSET/icon_256x256.png"
  cp "$STAGE/icon-512.png"  "$ICONSET/icon_256x256@2x.png"
  cp "$STAGE/icon-512.png"  "$ICONSET/icon_512x512.png"
  cp "$STAGE/icon-1024.png" "$ICONSET/icon_512x512@2x.png"
  iconutil -c icns "$ICONSET" -o "$STAGE/icon.icns"
elif command -v png2icns >/dev/null 2>&1; then
  png2icns "$STAGE/icon.icns" \
    "$STAGE/icon-16.png" "$STAGE/icon-32.png" "$STAGE/icon-48.png" \
    "$STAGE/icon-128.png" "$STAGE/icon-256.png" "$STAGE/icon-512.png" \
    >/dev/null
else
  echo "warning: neither iconutil nor png2icns found; skipping .icns" >&2
fi

# --- place the generated icons -------------------------------------------
# published path -> staged file
declare -A TARGETS=(
  ["$APP_ICON_DIR/icon.png"]="$STAGE/icon-256.png"
  ["$APP_ICON_DIR/icon.ico"]="$STAGE/icon.ico"
  ["$ASSET_ICON_DIR/aethercodex.png"]="$STAGE/icon-256.png"
  ["$ASSET_ICON_DIR/aethercodex.ico"]="$STAGE/icon.ico"
  ["$DOC_ICON_DIR/aethercodex.png"]="$STAGE/icon-256.png"
  ["$DOC_ICON_DIR/aethercodex.ico"]="$STAGE/icon.ico"
)
if [ -f "$STAGE/icon.icns" ]; then
  TARGETS["$APP_ICON_DIR/icon.icns"]="$STAGE/icon.icns"
fi

status=0
for target in "${!TARGETS[@]}"; do
  source_file="${TARGETS[$target]}"
  if [ "$CHECK_ONLY" = "1" ]; then
    if ! cmp -s "$source_file" "$target"; then
      echo "out of sync with the brand master: ${target#"$ROOT/"}" >&2
      status=1
    fi
  else
    mkdir -p "$(dirname "$target")"
    cp "$source_file" "$target"
    echo "wrote ${target#"$ROOT/"}"
  fi
done

if [ "$CHECK_ONLY" = "1" ]; then
  [ "$status" = "0" ] && echo "icons match the brand master"
  exit "$status"
fi

echo
echo "Icons regenerated from ${SVG_MASTER#"$ROOT/"}."
