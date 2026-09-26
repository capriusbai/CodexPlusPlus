#!/usr/bin/env bash
# Build the Linux release artifacts: a Debian package (Ubuntu 24.04 / 26.04,
# Debian 13+) and a distro-agnostic tarball.
#
# Usage: package-linux.sh <version> [arch]
#   arch defaults to the Debian architecture of the host (amd64 / arm64).
#   BINARY_DIR overrides where the release binaries are read from.
set -euo pipefail

VERSION="${1:-0.0.0}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
DIST="$ROOT/dist/linux"
BINARY_DIR="${BINARY_DIR:-$ROOT/target/release}"
ICON_SOURCE="$ROOT/apps/aethercodex-manager/src-tauri/icons/icon.png"

SILENT_BINARY="aethercodex"
MANAGER_BINARY="aethercodex-manager"
PACKAGE_NAME="aethercodex"
ICON_NAME="aethercodex"
INSTALL_PREFIX="/usr/lib/$PACKAGE_NAME"
MAINTAINER="Archai <haobai@gmail.com>"
HOMEPAGE="https://github.com/capriusbai/CodexPlusPlus"
# Package name shipped before the rename to AetherCodex. Declared below so
# apt replaces it cleanly instead of fighting over /usr/bin symlinks.
LEGACY_PACKAGE_NAME="codex-plus-plus"

host_deb_arch() {
  if command -v dpkg >/dev/null 2>&1; then
    dpkg --print-architecture
    return
  fi
  case "$(uname -m)" in
    x86_64) echo "amd64" ;;
    aarch64 | arm64) echo "arm64" ;;
    *) uname -m ;;
  esac
}

# Normalize whatever the caller passed (amd64/x64/x86_64, arm64/aarch64) into
# the Debian architecture plus the short label used in asset file names.
ARCH_INPUT="${2:-$(host_deb_arch)}"
case "$ARCH_INPUT" in
  amd64 | x64 | x86_64) DEB_ARCH="amd64"; ASSET_ARCH="x64" ;;
  arm64 | aarch64) DEB_ARCH="arm64"; ASSET_ARCH="arm64" ;;
  *)
    echo "error: unsupported architecture: $ARCH_INPUT" >&2
    exit 1
    ;;
esac

STAGE="$DIST/stage/$PACKAGE_NAME-$VERSION-$DEB_ARCH"
TARBALL_NAME="AetherCodex-$VERSION-linux-$ASSET_ARCH"
TARBALL_STAGE="$DIST/stage/$TARBALL_NAME"
DEB="$DIST/AetherCodex-$VERSION-linux-$ASSET_ARCH.deb"
TARBALL="$DIST/$TARBALL_NAME.tar.gz"

for binary in "$SILENT_BINARY" "$MANAGER_BINARY"; do
  if [ ! -x "$BINARY_DIR/$binary" ]; then
    echo "error: binary not found or not executable: $BINARY_DIR/$binary" >&2
    exit 1
  fi
done

rm -rf "$DIST"
mkdir -p "$DIST"

desktop_entry() {
  # $1 = display name, $2 = Exec target, $3 = manager (true/false)
  local name="$1"
  local exec_path="$2"
  local manager="$3"
  printf '[Desktop Entry]\n'
  printf 'Type=Application\n'
  printf 'Version=1.0\n'
  printf 'Name=%s\n' "$name"
  if [ "$manager" = "true" ]; then
    printf 'Name[zh_CN]=AetherCodex 管理工具\n'
    printf 'GenericName=AetherCodex control panel\n'
    printf 'GenericName[zh_CN]=AetherCodex 管理工具\n'
    printf 'Comment=Launch, repair, configure and update the AetherCodex enhancements\n'
    printf 'Comment[zh_CN]=启动、检查、修复、更新 AetherCodex 增强功能\n'
    printf 'Exec="%s" %%U\n' "$exec_path"
    printf 'StartupNotify=true\n'
    printf 'StartupWMClass=%s\n' "$MANAGER_BINARY"
  else
    printf 'GenericName=AetherCodex silent launcher\n'
    printf 'GenericName[zh_CN]=AetherCodex 静默启动入口\n'
    printf 'Comment=Start Codex and inject the AetherCodex enhancements\n'
    printf 'Comment[zh_CN]=启动 Codex 并注入 AetherCodex 增强功能\n'
    printf 'Exec="%s"\n' "$exec_path"
    printf 'StartupNotify=false\n'
  fi
  printf 'Icon=%s\n' "$ICON_NAME"
  printf 'Terminal=false\n'
  printf 'Categories=Development;Utility;\n'
  printf 'Keywords=codex;codex++;launcher;\n'
  printf 'X-Codex-Plus-Plus-Version=%s\n' "$VERSION"
}

stage_payload() {
  # Lay out the files both artifacts share, rooted at $1.
  local root="$1"
  local exec_prefix="$2"
  mkdir -p "$root$INSTALL_PREFIX"
  install -m 0755 "$BINARY_DIR/$SILENT_BINARY" "$root$INSTALL_PREFIX/$SILENT_BINARY"
  install -m 0755 "$BINARY_DIR/$MANAGER_BINARY" "$root$INSTALL_PREFIX/$MANAGER_BINARY"
  # The manager copies this icon into the user icon theme when it installs the
  # user-level entrypoints, so it has to sit next to the binaries.
  install -m 0644 "$ICON_SOURCE" "$root$INSTALL_PREFIX/$ICON_NAME.png"
  if [ -d "$BINARY_DIR/user_scripts" ]; then
    cp -R "$BINARY_DIR/user_scripts" "$root$INSTALL_PREFIX/user_scripts"
  fi

  mkdir -p "$root/usr/share/applications"
  desktop_entry "AetherCodex" "$exec_prefix/$SILENT_BINARY" false \
    > "$root/usr/share/applications/$PACKAGE_NAME.desktop"
  desktop_entry "AetherCodex Manager" "$exec_prefix/$MANAGER_BINARY" true \
    > "$root/usr/share/applications/$PACKAGE_NAME-manager.desktop"
  chmod 0644 "$root/usr/share/applications/$PACKAGE_NAME.desktop" \
    "$root/usr/share/applications/$PACKAGE_NAME-manager.desktop"

  mkdir -p "$root/usr/share/icons/hicolor/256x256/apps"
  install -m 0644 "$ICON_SOURCE" \
    "$root/usr/share/icons/hicolor/256x256/apps/$ICON_NAME.png"
}

build_deb() {
  rm -rf "$STAGE"
  mkdir -p "$STAGE/DEBIAN"
  stage_payload "$STAGE" "$INSTALL_PREFIX"

  # Relative links inside /usr, per Debian policy: /usr/bin/x -> ../lib/...
  mkdir -p "$STAGE/usr/bin"
  ln -sf "../lib/$PACKAGE_NAME/$SILENT_BINARY" "$STAGE/usr/bin/$SILENT_BINARY"
  ln -sf "../lib/$PACKAGE_NAME/$MANAGER_BINARY" "$STAGE/usr/bin/$MANAGER_BINARY"

  mkdir -p "$STAGE/usr/share/doc/$PACKAGE_NAME"
  cat > "$STAGE/usr/share/doc/$PACKAGE_NAME/copyright" <<COPYRIGHT
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: AetherCodex
Source: $HOMEPAGE

Files: *
Copyright: 2026 Archai
           2025-2026 BigPizzaV3
Comment: AetherCodex is a rebranded fork of Codex++ by BigPizzaV3
 (https://github.com/BigPizzaV3/CodexPlusPlus). The upstream copyright notice
 is retained as the MIT licence requires.
License: MIT
 Permission is hereby granted, free of charge, to any person obtaining a copy
 of this software and associated documentation files (the "Software"), to deal
 in the Software without restriction, including without limitation the rights
 to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 copies of the Software, and to permit persons to whom the Software is
 furnished to do so, subject to the following conditions:
 .
 The above copyright notice and this permission notice shall be included in all
 copies or substantial portions of the Software.
 .
 THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 SOFTWARE.
COPYRIGHT

  printf '%s (%s) stable; urgency=medium\n\n  * AetherCodex %s\n\n -- %s  %s\n' "$PACKAGE_NAME" \
    "$VERSION" "$VERSION" "$MAINTAINER" "$(date -R)" \
    | gzip -9n > "$STAGE/usr/share/doc/$PACKAGE_NAME/changelog.Debian.gz"
  chmod 0644 "$STAGE/usr/share/doc/$PACKAGE_NAME/changelog.Debian.gz" \
    "$STAGE/usr/share/doc/$PACKAGE_NAME/copyright"

  local installed_size
  installed_size="$(du -ks "$STAGE" | cut -f1)"

  cat > "$STAGE/DEBIAN/control" <<CONTROL
Package: $PACKAGE_NAME
Version: $VERSION
Section: devel
Priority: optional
Architecture: $DEB_ARCH
Maintainer: $MAINTAINER
Homepage: $HOMEPAGE
Installed-Size: $installed_size
Conflicts: $LEGACY_PACKAGE_NAME
Replaces: $LEGACY_PACKAGE_NAME
Provides: $LEGACY_PACKAGE_NAME
Depends: libc6, libcairo2, libdbus-1-3, libgdk-pixbuf-2.0-0 | libgdk-pixbuf2.0-0, libglib2.0-0t64 | libglib2.0-0, libgtk-3-0t64 | libgtk-3-0, libjavascriptcoregtk-4.1-0, libsoup-3.0-0, libwebkit2gtk-4.1-0
Recommends: xdg-utils
Description: External enhancement launcher and manager for the Codex App
 AetherCodex starts the Codex App from an external launcher and injects its
 enhancement scripts over the Chromium DevTools Protocol, leaving the original
 Codex installation untouched.
 .
 The package installs two entry points: "AetherCodex", a silent launcher that only
 starts Codex and injects the enhancements, and "AetherCodex Manager", a control
 panel for launching, checking, repairing, updating, configuring relay
 injection and managing user scripts.
CONTROL

  cat > "$STAGE/DEBIAN/postinst" <<'POSTINST'
#!/bin/sh
set -e

if [ "$1" = "configure" ]; then
  if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q /usr/share/applications || true
  fi
  if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
  fi
fi

exit 0
POSTINST

  cat > "$STAGE/DEBIAN/postrm" <<'POSTRM'
#!/bin/sh
set -e

if [ "$1" = "remove" ] || [ "$1" = "purge" ]; then
  if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q /usr/share/applications || true
  fi
  if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
  fi
fi

exit 0
POSTRM

  chmod 0755 "$STAGE/DEBIAN/postinst" "$STAGE/DEBIAN/postrm"

  # `--root-owner-group` keeps the package reproducible when built as a
  # non-root CI user.
  dpkg-deb --build --root-owner-group "$STAGE" "$DEB" >/dev/null
}

build_tarball() {
  rm -rf "$TARBALL_STAGE"
  mkdir -p "$TARBALL_STAGE"
  # The tarball is relocatable, so its desktop entries are rewritten by
  # install.sh once the user picks a prefix.
  stage_payload "$TARBALL_STAGE/payload" "$INSTALL_PREFIX"

  mkdir -p "$TARBALL_STAGE/bin"
  mv "$TARBALL_STAGE/payload$INSTALL_PREFIX/$SILENT_BINARY" "$TARBALL_STAGE/bin/"
  mv "$TARBALL_STAGE/payload$INSTALL_PREFIX/$MANAGER_BINARY" "$TARBALL_STAGE/bin/"
  mv "$TARBALL_STAGE/payload$INSTALL_PREFIX/$ICON_NAME.png" "$TARBALL_STAGE/bin/"
  if [ -d "$TARBALL_STAGE/payload$INSTALL_PREFIX/user_scripts" ]; then
    mv "$TARBALL_STAGE/payload$INSTALL_PREFIX/user_scripts" "$TARBALL_STAGE/bin/"
  fi
  rm -rf "$TARBALL_STAGE/payload"

  cat > "$TARBALL_STAGE/install.sh" <<'INSTALL'
#!/usr/bin/env sh
# Install AetherCodex for the current user only (no root required).
set -eu

HERE="$(cd "$(dirname "$0")" && pwd)"
PREFIX="${PREFIX:-$HOME/.local}"
APP_DIR="$PREFIX/lib/aethercodex"

mkdir -p "$APP_DIR" "$PREFIX/bin"
cp -f "$HERE/bin/aethercodex" "$HERE/bin/aethercodex-manager" "$APP_DIR/"
cp -f "$HERE/bin/aethercodex.png" "$APP_DIR/"
chmod 0755 "$APP_DIR/aethercodex" "$APP_DIR/aethercodex-manager"
if [ -d "$HERE/bin/user_scripts" ]; then
  rm -rf "$APP_DIR/user_scripts"
  cp -R "$HERE/bin/user_scripts" "$APP_DIR/user_scripts"
fi
ln -sf "$APP_DIR/aethercodex" "$PREFIX/bin/aethercodex"
ln -sf "$APP_DIR/aethercodex-manager" "$PREFIX/bin/aethercodex-manager"

# The manager owns the desktop entries and the icon theme, so let it write
# them; this keeps a tarball install and a .deb install in sync.
if ! "$APP_DIR/aethercodex-manager" --install-entrypoints; then
  cat <<NOTE
Could not create the application menu entries automatically.
Check the runtime dependencies first:
  sudo apt install libwebkit2gtk-4.1-0 libgtk-3-0t64 xdg-utils
then run:
  $APP_DIR/aethercodex-manager --install-entrypoints
NOTE
fi

echo "AetherCodex installed to $APP_DIR"
INSTALL

  cat > "$TARBALL_STAGE/uninstall.sh" <<'UNINSTALL'
#!/usr/bin/env sh
set -eu

PREFIX="${PREFIX:-$HOME/.local}"
APP_DIR="$PREFIX/lib/aethercodex"
DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}"

rm -f "$PREFIX/bin/aethercodex" "$PREFIX/bin/aethercodex-manager"
rm -f "$DATA_DIR/applications/aethercodex.desktop" \
      "$DATA_DIR/applications/aethercodex-manager.desktop"
rm -f "$DATA_DIR/icons/hicolor/256x256/apps/aethercodex.png"
rm -rf "$APP_DIR"

echo "AetherCodex removed from $APP_DIR (settings in ~/.aethercodex were kept)"
UNINSTALL

  chmod 0755 "$TARBALL_STAGE/install.sh" "$TARBALL_STAGE/uninstall.sh"

  cat > "$TARBALL_STAGE/README.txt" <<README
AetherCodex $VERSION — Linux ($ASSET_ARCH)

Runtime requirements (Ubuntu 24.04 / 26.04, Debian 13+):
  sudo apt install libwebkit2gtk-4.1-0 libgtk-3-0t64 xdg-utils

Install for the current user:
  ./install.sh

Remove:
  ./uninstall.sh

Binaries:
  bin/aethercodex          silent launcher (starts Codex, injects AetherCodex)
  bin/aethercodex-manager  control panel

$HOMEPAGE
README

  tar -czf "$TARBALL" -C "$DIST/stage" "$TARBALL_NAME"
}

verify_deb() {
  local listing="$DIST/stage/deb-contents.txt"
  dpkg-deb --info "$DEB" >/dev/null
  # Snapshot the listing instead of piping into grep: an early `grep -q` exit
  # would kill dpkg-deb with SIGPIPE and trip `set -o pipefail`.
  dpkg-deb --contents "$DEB" > "$listing"
  for path in \
    "$INSTALL_PREFIX/$SILENT_BINARY" \
    "$INSTALL_PREFIX/$MANAGER_BINARY" \
    "/usr/bin/$SILENT_BINARY" \
    "/usr/bin/$MANAGER_BINARY" \
    "/usr/share/applications/$PACKAGE_NAME.desktop" \
    "/usr/share/applications/$PACKAGE_NAME-manager.desktop" \
    "/usr/share/icons/hicolor/256x256/apps/$ICON_NAME.png" \
    "/usr/share/doc/$PACKAGE_NAME/copyright"; do
    if ! grep -q -- " \.${path}\( ->.*\)\?\$" "$listing"; then
      echo "error: $path missing from $DEB" >&2
      return 1
    fi
  done
  if command -v desktop-file-validate >/dev/null 2>&1; then
    desktop-file-validate "$STAGE/usr/share/applications/$PACKAGE_NAME.desktop"
    desktop-file-validate "$STAGE/usr/share/applications/$PACKAGE_NAME-manager.desktop"
  fi
  echo "verified: $DEB"
}

verify_tarball() {
  local listing="$DIST/stage/tarball-contents.txt"
  tar -tzf "$TARBALL" > "$listing"
  for path in \
    "$TARBALL_NAME/bin/$SILENT_BINARY" \
    "$TARBALL_NAME/bin/$MANAGER_BINARY" \
    "$TARBALL_NAME/bin/$ICON_NAME.png" \
    "$TARBALL_NAME/install.sh" \
    "$TARBALL_NAME/uninstall.sh"; do
    if ! grep -qx -- "$path" "$listing"; then
      echo "error: $path missing from $TARBALL" >&2
      return 1
    fi
  done
  echo "verified: $TARBALL"
}

build_deb
build_tarball
verify_deb
verify_tarball

echo "$DEB"
echo "$TARBALL"
