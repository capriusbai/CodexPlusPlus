# AetherCodex

<p align="center">
  <img src="docs/images/aethercodex.png" alt="AetherCodex icon" width="160">
</p>

<p align="center">
  <a href="README.md">中文</a> | English
</p>

<p align="center">
  <img alt="Release" src="https://img.shields.io/github/v/release/capriusbai/CodexPlusPlus">
  <img alt="Stars" src="https://img.shields.io/github/stars/capriusbai/CodexPlusPlus">
  <img alt="License" src="https://img.shields.io/github/license/capriusbai/CodexPlusPlus">
  <img alt="Rust" src="https://img.shields.io/badge/rust-1.85%2B-orange">
  <img alt="Tauri" src="https://img.shields.io/badge/tauri-2.x-24C8DB">
</p>

AetherCodex is an external enhancement launcher and manager for the Codex App. It does not modify the original Codex installation. Instead, it starts Codex externally and injects enhancements through the Chromium DevTools Protocol.

## Quick Start

Download the latest installer from [GitHub Releases](https://github.com/capriusbai/CodexPlusPlus/releases):

- Windows: `AetherCodex-*-windows-x64-setup.exe`
- macOS Intel: `AetherCodex-*-macos-x64.dmg`
- macOS Apple Silicon: `AetherCodex-*-macos-arm64.dmg`
- Linux x86_64: `AetherCodex-*-linux-x64.deb` or `AetherCodex-*-linux-x64.tar.gz`
- Linux arm64: `AetherCodex-*-linux-arm64.deb` or `AetherCodex-*-linux-arm64.tar.gz`

After installation, two entry points are available:

- `AetherCodex`: a silent launcher. It does not show the manager UI and only starts Codex with AetherCodex injection.
- `AetherCodex Manager`: a Tauri control panel for launch, diagnostics, repair, updates, relay injection, enhancements, and user scripts.

The Windows installer creates desktop and Start Menu shortcuts. The macOS DMG installs `/Applications/AetherCodex.app` and `/Applications/AetherCodex 管理工具.app`. The Linux `.deb` installs into `/usr/lib/aethercodex/` and registers `AetherCodex` and `AetherCodex Manager` in the application menu.

### Ubuntu 26.04 / 24.04, Debian 13+

The `.deb` targets Ubuntu 24.04 and newer (including 26.04) and Debian 13 and newer, and relies on the WebKitGTK 4.1 runtime those releases ship:

```bash
sudo apt install ./AetherCodex-1.3.0-linux-x64.deb
```

`apt` pulls in the runtime dependencies (`libwebkit2gtk-4.1-0`, `libgtk-3-0t64`, and friends). If you install with `dpkg -i` instead and it reports missing dependencies, run `sudo apt -f install`.

To remove it:

```bash
sudo apt remove aethercodex
```

### Other distributions (tar.gz)

The `.tar.gz` needs no package manager and installs into `~/.local` for the current user:

```bash
tar -xzf AetherCodex-1.3.0-linux-x64.tar.gz
cd AetherCodex-1.3.0-linux-x64
./install.sh            # or PREFIX=/opt/aethercodex ./install.sh
```

Make sure the distribution provides the WebKitGTK 4.1 runtime first (`libwebkit2gtk-4.1-0` on Ubuntu/Debian, `webkit2gtk4.1` on Fedora, `webkit2gtk-4.1` on Arch). Run `./uninstall.sh` from the same directory to remove it.

`install.sh` calls `aethercodex-manager --install-entrypoints` to write the user-level `.desktop` entries. You can rerun that command at any time to rebuild them, or pass `--uninstall-entrypoints` to remove them.

## About this fork

AetherCodex is a rebranded fork of [Codex++](https://github.com/BigPizzaV3/CodexPlusPlus), maintained by Archai.
The upstream project is MIT-licensed by BigPizzaV3; the external launcher and
CDP injection design, the enhancement scripts and most of the feature work come
from upstream, and the copyright notice is retained in the package `copyright`
file as the licence requires.

What this fork changes:

- branding, naming, package identity and interface visuals move to
  AetherCodex / the Archai CI-VI;
- Linux releases are added (Ubuntu 24.04 and newer, including 26.04);
- automatic updates resolve against this fork's releases rather than upstream's.

This fork removes upstream's recommendation/ad feature and donation entries:
no remote ad list is fetched and no donation codes appear in the interface.
Upstream's sponsors, chat groups and donation channels belong to the upstream
project; to support the original author, go to the
[upstream repository](https://github.com/BigPizzaV3/CodexPlusPlus).

## Brand and interface

The interface follows the Archai CI/VI working standard (`ARCHAI-CIVI-001`):
square geometry, hairline rules and restrained semantic colour, with no
decorative shadow, gradient or glassmorphism.

| Colour | sRGB | Meaning |
| --- | --- | --- |
| Archai Orange | `#ED9527` | brand anchor, section marker. **Never** small body copy on a light surface (2.35:1 on white) |
| Aether Blue | `#2F6FED` | connection, interface, link, editable field |
| Evidence Green | `#2F855A` | verified / released / closed status backed by evidence only |
| Engineering Graphite | `#0B0D10` | primary typography and technical authority |
| Secondary Ink | `#626A78` | metadata and secondary copy |
| Soft Surface | `#F7F9FC` | non-semantic surface layering |
| Hairline | `#DDE3EC` | rules and table boundaries |

Blue and green are lightened in the dark theme and darkened in the light theme
so both clear WCAG AA body contrast (4.5:1). Orange gets a darkened ink variant
for text and icons on light surfaces.

### Replacing the logo

Every icon is generated from one master:

```bash
# Replace assets/brand/aethercodex-mark.svg (or .png, >= 1024x1024), then:
bash scripts/brand/generate-icons.sh          # regenerate Windows .ico / macOS .icns / Linux PNG / app icon
bash scripts/brand/generate-icons.sh --check  # verify the icons match the master (CI runs this)
```

The master is the Aether logo supplied by the project owner, vector-traced
with `potrace` and placed on the graphite ground. Provenance, hash and usage
limits are in [`assets/brand/README.md`](assets/brand/README.md).

## Highlights

- Rust backend and silent launcher with no extra runtime requirement.
- Tauri + React manager with dark/light theme support.
- External CDP injection. No `app.asar` patching and no DLL writes into the Codex installation.
- Relay injection mode with multiple relay profiles, `AetherCodex` provider configuration, and a one-click switch back to official ChatGPT login mode.
- Traditional enhancement mode with plugin entry unlock, forced plugin install, session delete, Markdown export, project move, Timeline, and more.
- Independent user script management with startup injection.
- Provider Sync to keep historical sessions visible after switching providers.
- Zed open entry detects remote SSH context and opens the matching remote file in Zed Remote Development from Codex.
- Upstream worktree creation: create new worktrees from `upstream/<base-branch>` after fetching the remote branch, reducing conflicts caused by stale local HEAD state.
- GitHub Release updates. Both the manager and silent launcher can detect available updates.
- Windows single instance, no console window, administrator manifest, and system Desktop path detection.
- Separate macOS x64 and arm64 DMGs. The silent launcher hides its Dock icon.
- Separate Linux x64 and arm64 `.deb` packages plus a portable `.tar.gz`, registering XDG `.desktop` entries and a hicolor icon.

## Relay Injection

Relay injection is for users who are already logged in with an official ChatGPT account in Codex/ChatGPT and want model requests to go through a custom compatible API.

The boundary of this hybrid mode is:

- The official ChatGPT/Codex login state still owns Codex App account features and the plugin entry.
- The relay profile only controls the Base URL, key, and model names used for model requests.
- The compatible API provider is not tied to any specific vendor; it only needs to match the selected upstream protocol and Codex configuration.
- Clearing API mode should return Codex to the official login mode so the official account and plugins keep working.

Before applying relay injection, run a minimal preflight:

1. Make sure Codex has detected the ChatGPT login state and the plugin entry is available.
2. Confirm the custom Base URL is reachable and supports the selected upstream protocol, such as a Responses-compatible endpoint.
3. Test the target key with the smallest useful auth probe, such as a model-list request or a short message request.
4. Only record whether the key exists and whether auth passed. Do not paste real keys into logs, screenshots, or issues.
5. Make sure `~/.codex/config.toml` has a backup so clearing API mode can safely roll back.

In the manager's Relay Injection page:

1. Make sure ChatGPT login status is detected.
2. Add one or more relay profiles with Base URL and Key.
3. Select the active profile and apply relay injection.
4. Launch `AetherCodex`.

AetherCodex writes configuration similar to this into `~/.codex/config.toml`:

```toml
model_provider = "AetherCodex"

[model_providers.AetherCodex]
name = "AetherCodex"
wire_api = "responses"
requires_openai_auth = true
base_url = "https://example.com/v1"
experimental_bearer_token = "sk-..."
```

To return to the official login mode, use the clear API mode button in the Relay Injection page. This removes `OPENAI_API_KEY` related configuration and switches Codex back to official ChatGPT authentication.

## Enhancements

Enhancements are controlled in the manager. Enhancement injection is enabled by default. When disabled, AetherCodex will not inject its menu or scripts.

When relay injection mode is active, plugin entry unlock and forced plugin install are unnecessary, and the UI will say so. Other enhancements, including session delete, export, move, Timeline and user scripts, can still be used.

## Updates and Packages

AetherCodex publishes installers through GitHub Releases. Windows builds an NSIS installer, macOS builds separate Intel x64 and Apple Silicon arm64 DMGs, and Linux builds `.deb` and `.tar.gz` artifacts for x64 and arm64.

The manager's About page can check and start updates. When the silent launcher finds a new version, it opens the manager directly on the update prompt. Only assets matching the current OS and CPU architecture are offered: on Linux the `.deb` wins, falling back to the same-architecture `.tar.gz` when no matching `.deb` exists.

## Data Locations

- Codex config: `~/.codex/config.toml`
- Codex auth state: `~/.codex/auth.json`
- Codex local database: `~/.codex/state_5.sqlite`
- AetherCodex state and logs: `~/.aethercodex/`
- Provider Sync backups: `~/.codex/backups_state/provider-sync`

## FAQ

### The AetherCodex menu does not appear

Make sure Codex was launched from the `AetherCodex` entry instead of the original Codex entry. You can also inspect the Diagnostics and Logs pages in the manager.

### The plugin says the backend is disconnected

First test the helper endpoint:

```powershell
Invoke-RestMethod -Method Post -Uri http://127.0.0.1:57321/backend/status -Body "{}" -ContentType "application/json"
```

If the endpoint works but the plugin still times out, it is usually a Codex page CDP bridge or script cache issue. Restart AetherCodex, or check manager logs for `renderer.script_loaded`, `bridge.request`, and `bridge.response`.

### How is Upstream worktree different from Codex native creation?

AetherCodex updates the remote branch first, then creates the worktree as if you ran:

```bash
git worktree add -b <new-branch> <worktree-path> upstream/<base-branch>
```

The new worktree starts from the fresh remote tracking branch instead of the local HEAD used by the current session. If AetherCodex cannot safely recognize the current Codex version's native worktree form, use the AetherCodex menu entry and enter the repository path, branch name, worktree path, remote, and base branch manually.

### macOS says the app cannot be opened or is damaged

Unsigned and unnotarized builds may be blocked by Gatekeeper. Allow the app in System Settings -> Privacy & Security. For formal distribution, configure Apple Developer ID signing and notarization.

### Does it support Intel Macs?

Yes. Releases provide both `macos-x64.dmg` and `macos-arm64.dmg`. Intel Macs should use the x64 package, while Apple Silicon Macs should use the arm64 package.

### The manager window is blank on Linux

That is a WebKitGTK rendering problem, not an injection failure. Try disabling the accelerated renderer first:

```bash
WEBKIT_DISABLE_DMABUF_RENDERER=1 aethercodex-manager
```

Since Ubuntu 24.04 the kernel ships `kernel.apparmor_restrict_unprivileged_userns=1`, which stops processes that rely on the bubblewrap sandbox from creating a user namespace. The AetherCodex manager itself does not use that sandbox, but the Codex App it launches (Electron/Chromium) can be affected. If Codex itself fails to start, load the profile Ubuntu ships for this:

```bash
sudo apt install apparmor-profiles
sudo install -m 0644 /usr/share/apparmor/extra-profiles/bwrap-userns-restrict /etc/apparmor.d/bwrap-userns-restrict
sudo apparmor_parser -r /etc/apparmor.d/bwrap-userns-restrict
```

### AetherCodex cannot find the Codex App on Linux

Linux has no fixed install location like the Microsoft Store or `/Applications`, so AetherCodex looks under `/opt`, `/usr/lib`, `/usr/share`, `~/.local/share`, and `~/Applications` for a directory holding a Codex executable (`Codex`, `codex`, or `codex-app`), plus any `.AppImage` whose name contains `codex`.

Only directories that really contain an executable are accepted, so a `codex` CLI on `PATH` is never mistaken for the desktop app. If detection fails, set the Codex App path manually in the manager's settings — an `.AppImage` file path works too.

## Development

```bash
# Frontend checks
cd apps/aethercodex-manager
npm install
npm run check
npm run vite:build

# Rust checks
cd ../..
cargo fmt --check
cargo test
cargo build --release
```

Project structure:

```text
apps/
  aethercodex-launcher/          Silent launcher
  aethercodex-manager/           Tauri manager
assets/inject/
  renderer-inject.js            Enhancement script injected into Codex
crates/
  aethercodex-core/              Launch, injection, config, update, install, bridge
  aethercodex-data/              Session data, export, Provider Sync
scripts/installer/
  windows/AetherCodex.nsi     Windows NSIS installer
  macos/package-dmg.sh          macOS DMG packager
  linux/package-linux.sh        Linux .deb and .tar.gz packager
```

Building the Linux artifacts locally (run `cargo build --release` first):

```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev desktop-file-utils
cargo build --release
bash scripts/installer/linux/package-linux.sh 1.3.0     # architecture defaults to the host; pass x64 / arm64 to override
```

Artifacts land in `dist/linux/`. CI pins the build to `ubuntu-24.04`: its glibc and WebKitGTK 4.1 soname are the oldest the artifacts have to support, and newer Ubuntu releases (25.10, 26.04) stay backwards compatible.

## Community and Support

Report problems and suggestions for this fork in [Issues](https://github.com/capriusbai/CodexPlusPlus/issues).

The Codex++ community channels (QQ, WeChat, Telegram) belong to the upstream
project; find them in the [upstream repository](https://github.com/BigPizzaV3/CodexPlusPlus).

## Friendly Links

- [Upstream project Codex++](https://github.com/BigPizzaV3/CodexPlusPlus)
- [LINUX DO](https://linux.do)

## Notes

AetherCodex is an external enhancement tool and does not modify original Codex App files. If a future Codex App update changes page structure, the injection script may need updates.
