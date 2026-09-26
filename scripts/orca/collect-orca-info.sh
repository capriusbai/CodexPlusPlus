#!/usr/bin/env bash
# Collect what the AetherCodex × Orca integration design needs to know about a
# local Orca install. Read-only: it never writes, starts, stops or reconfigures
# anything, and it does not read credential files.
#
#   bash scripts/orca/collect-orca-info.sh            # print a report
#   bash scripts/orca/collect-orca-info.sh > orca.txt # save it to send on
#
# Review the output before sharing. Paths under your home directory appear as
# ~, but project names in worktree paths are not redacted.
set -uo pipefail

section() { printf '\n=== %s ===\n' "$1"; }
# Keep the home directory out of the report.
tilde() { sed "s#${HOME}#~#g"; }
have() { command -v "$1" >/dev/null 2>&1; }

printf 'Orca deployment report — %s\n' "$(date -u '+%Y-%m-%d %H:%M UTC')"
printf 'host: %s %s\n' "$(uname -s)" "$(uname -m)"

section 'Orca CLI'
# Name collision: Ubuntu ships GNOME Orca (the screen reader) as `orca` and
# /usr/bin/orca. Orca-the-IDE knows this and installs as `orca-ide` on Linux
# (electron-builder executableName/packageName). Never run subcommands against
# a binary that turns out to be the screen reader.
ORCA_BIN=''
for name in orca-ide orca; do
  have "$name" || continue
  version="$($name --version 2>&1 | head -3)"
  if printf '%s' "$version" | grep -qiE 'AT-SPI|screen reader'; then
    printf 'skipped %s: this is GNOME Orca, the screen reader, not Orca IDE\n' \
      "$(command -v "$name" | tilde)"
    continue
  fi
  ORCA_BIN="$name"
  printf 'path: %s\n' "$(command -v "$name" | tilde)"
  printf '%s\n' "$version"
  break
done
if [ -z "$ORCA_BIN" ]; then
  echo 'Orca IDE CLI: not found on PATH'
  # Where the packaged app puts it when the PATH symlink is missing.
  for candidate in \
    "/opt/orca-ide/bin/orca-ide" \
    "/opt/Orca/bin/orca-ide" \
    "/usr/lib/orca-ide/bin/orca-ide" \
    "/usr/local/bin/orca-ide" \
    "$HOME/.local/bin/orca-ide" \
    "/Applications/Orca.app/Contents/Resources/app/out/cli/index.js"; do
    if [ -e "$candidate" ]; then
      printf 'found unlinked: %s\n' "$(echo "$candidate" | tilde)"
      [ -z "$ORCA_BIN" ] && ORCA_BIN="$candidate"
    fi
  done
fi

section 'Application install'
for candidate in \
  "/Applications/Orca.app" \
  "$HOME/Applications/Orca.app" \
  "/opt/orca-ide" \
  "/opt/Orca" \
  "/usr/lib/orca-ide" \
  "$HOME/.local/share/orca-ide"; do
  if [ -e "$candidate" ]; then
    printf '%s\n' "$(echo "$candidate" | tilde)"
    du -sh "$candidate" 2>/dev/null | tilde
  fi
done
if have dpkg-query; then
  # orca-ide is the IDE; a bare `orca` row is the GNOME screen reader.
  dpkg-query -W -f='deb: ${Package} ${Version}\n' orca-ide 2>/dev/null
  dpkg-query -W -f='deb: ${Package} ${Version}  <- GNOME screen reader, unrelated\n' \
    orca 2>/dev/null
fi
if have brew; then brew list --cask 2>/dev/null | grep -i orca | sed 's/^/brew cask: /'; fi

section 'User data directories'
for candidate in \
  "$HOME/.orca" \
  "$HOME/.config/Orca" \
  "$HOME/.config/orca" \
  "$HOME/.config/orca-ide" \
  "$HOME/Library/Application Support/Orca" \
  "$HOME/AppData/Roaming/Orca"; do
  if [ -d "$candidate" ]; then
    printf '\n%s\n' "$(echo "$candidate" | tilde)"
    # Names and sizes only; no file contents.
    ls -la "$candidate" 2>/dev/null | head -25 | tilde
  fi
done

section 'Plugins'
# Where a plugin would be installed, and what is installed there. An
# orca-plugin.json manifest is what the integration needs to see.
for candidate in \
  "$HOME/.orca/plugins" \
  "$HOME/.config/Orca/plugins" \
  "$HOME/.config/orca/plugins" \
  "$HOME/.config/orca-ide/plugins" \
  "$HOME/Library/Application Support/Orca/plugins" \
  "$HOME/AppData/Roaming/Orca/plugins"; do
  if [ -d "$candidate" ]; then
    printf '\nplugin dir: %s\n' "$(echo "$candidate" | tilde)"
    ls -la "$candidate" 2>/dev/null | tilde
    find "$candidate" -maxdepth 3 -name 'orca-plugin.json' 2>/dev/null | while read -r manifest; do
      printf '\n--- %s ---\n' "$(echo "$manifest" | tilde)"
      cat "$manifest" | tilde
    done
  fi
done

section 'Agent profiles / adaptors'
# The user-built agent adaptor most likely lives as a profile or plugin
# contribution. Anything matching is listed by name so it can be shared.
for root in "$HOME/.orca" "$HOME/.config/Orca" "$HOME/.config/orca" \
            "$HOME/.config/orca-ide" "$HOME/Library/Application Support/Orca"; do
  [ -d "$root" ] || continue
  find "$root" -maxdepth 4 \
    \( -iname '*agent*profile*' -o -iname '*adapt*' -o -iname '*agent*.json' \
       -o -iname '*agent*.yaml' -o -iname '*agent*.yml' -o -iname '*agent*.ts' \) \
    -not -path '*/node_modules/*' 2>/dev/null | head -40 | tilde
done

section 'Codex homes visible to Orca'
printf 'CODEX_HOME env: %s\n' "${CODEX_HOME:-<unset>}"
for candidate in "$HOME/.codex" "$HOME/.aethercodex/codex-home"; do
  if [ -d "$candidate" ]; then
    printf '\n%s\n' "$(echo "$candidate" | tilde)"
    # File names only — auth.json contents are never printed.
    ls -1 "$candidate" 2>/dev/null | head -20
  fi
done
# Orca's per-account managed homes, if any.
for root in "$HOME/.orca" "$HOME/Library/Application Support/Orca" "$HOME/.config/Orca"; do
  [ -d "$root" ] || continue
  find "$root" -maxdepth 4 -type d -name 'codex*home*' 2>/dev/null | head -10 | tilde
done

section 'Orca state (read-only queries)'
if [ -n "$ORCA_BIN" ]; then
  for query in 'account list' 'project list' 'orchestration run-list'; do
    printf '\n$ %s %s --json\n' "$ORCA_BIN" "$query"
    # shellcheck disable=SC2086 -- the query is a fixed, space-separated command.
    timeout 20 "$ORCA_BIN" $query --json 2>&1 | head -40 | tilde
  done
else
  echo 'skipped: Orca IDE CLI not found (nothing was run)'
fi

section 'Done'
echo 'Nothing was modified. Review before sharing, then send this file.'
