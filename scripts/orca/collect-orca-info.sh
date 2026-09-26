#!/usr/bin/env bash
# Collect what the AetherCodex × Orca integration design needs to know about a
# local Orca install. Read-only: it never writes, starts, stops or reconfigures
# anything.
#
#   bash scripts/orca/collect-orca-info.sh                 # the survey
#   bash scripts/orca/collect-orca-info.sh --with-sources   # + adaptor source
#
# The default survey lists names, sizes and Orca's own read-only queries, and
# reads no file contents except plugin manifests. `--with-sources` additionally
# prints the source of the agent adaptor / hooks it found, because the design
# needs to see the interface shape, not just the filename. Those files are
# passed through a redactor for token-shaped values, but redaction is a
# safety net and not a guarantee:
#
#   REVIEW THE OUTPUT BEFORE SHARING IT, ESPECIALLY WITH --with-sources.
#
# Paths under your home directory appear as ~, but project names in worktree
# paths are not redacted.
set -uo pipefail

WITH_SOURCES=0
for arg in "$@"; do
  case "$arg" in
    --with-sources) WITH_SOURCES=1 ;;
    -h|--help) sed -n '2,20p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) printf 'unknown option: %s (see --help)\n' "$arg" >&2; exit 2 ;;
  esac
done

# Caps so a stray large file cannot turn the report into something unreadable.
MAX_FILE_BYTES=65536
MAX_SOURCE_FILES=40

section() { printf '\n=== %s ===\n' "$1"; }
# Keep the home directory out of the report.
tilde() { sed "s#${HOME}#~#g"; }
have() { command -v "$1" >/dev/null 2>&1; }

# Blank out token-shaped values. Matches by value shape (known credential
# prefixes, JWTs) and by key name, so it survives unfamiliar formats better
# than a prefix list alone.
redact() {
  sed -E \
    -e 's/(sk-[A-Za-z0-9_-]{6})[A-Za-z0-9_-]{8,}/\1…REDACTED/g' \
    -e 's/(gh[pousr]_[A-Za-z0-9]{4})[A-Za-z0-9]{8,}/\1…REDACTED/g' \
    -e 's/(eyJ[A-Za-z0-9_-]{4})[A-Za-z0-9._-]{16,}/\1…REDACTED/g' \
    -e 's/([Bb]earer )[A-Za-z0-9._-]{12,}/\1…REDACTED/g' \
    -e 's/((secret|password|passwd|api[_-]?key|access[_-]?token|refresh[_-]?token|[a-z]*token)"?'"'"'?[[:space:]]*[:=][[:space:]]*"?'"'"'?)[A-Za-z0-9._~+\/-]{12,}/\1…REDACTED/gI' \
    | tilde
}

printf 'Orca deployment report — %s\n' "$(date -u '+%Y-%m-%d %H:%M UTC')"
printf 'host: %s %s\n' "$(uname -s)" "$(uname -m)"
printf 'sources: %s\n' "$([ "$WITH_SOURCES" -eq 1 ] && echo 'included (review before sharing)' || echo 'not included')"

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

# The roots any Orca user data lives under, on every platform.
ORCA_ROOTS=(
  "$HOME/.orca"
  "$HOME/.config/Orca"
  "$HOME/.config/orca"
  "$HOME/.config/orca-ide"
  "$HOME/Library/Application Support/Orca"
  "$HOME/AppData/Roaming/Orca"
)

section 'User data directories'
for candidate in "${ORCA_ROOTS[@]}"; do
  if [ -d "$candidate" ]; then
    printf '\n%s\n' "$(echo "$candidate" | tilde)"
    # Names and sizes only; no file contents. Chromium's own cache entries are
    # dropped so the app's real directories are not pushed out of the listing.
    ls -la "$candidate" 2>/dev/null \
      | grep -vE ' (Cache|Code Cache|GPUCache|Dawn[A-Za-z]*Cache|Crashpad|Dictionaries|Local Storage|Session Storage|Shared Dictionary|Partitions|blob_storage|Cookies.*|DIPS.*|SharedStorage.*|Network Persistent State|Local State|Preferences|Singleton.*|\.updaterId)$' \
      | head -30 | tilde
  fi
done

section 'Plugins'
# Where a plugin would be installed, and what is installed there. An
# orca-plugin.json manifest is what the integration needs to see.
for root in "${ORCA_ROOTS[@]}"; do
  candidate="$root/plugins"
  if [ -d "$candidate" ]; then
    printf '\nplugin dir: %s\n' "$(echo "$candidate" | tilde)"
    ls -la "$candidate" 2>/dev/null | tilde
    find "$candidate" -maxdepth 3 -name 'orca-plugin.json' 2>/dev/null | while read -r manifest; do
      printf '\n--- %s ---\n' "$(echo "$manifest" | tilde)"
      redact <"$manifest"
    done
  fi
done

section 'Agent profiles / adaptors'
# The user-built agent adaptor is not necessarily a standard plugin: it may sit
# anywhere under the Orca roots, as a hooks directory or an "extension"
# directory. Match on hooks/extension/adaptor/agent names, and on directories
# as well as files, so a whole adaptor tree is found rather than one file of it.
ADAPTOR_HITS=()
for root in "${ORCA_ROOTS[@]}"; do
  [ -d "$root" ] || continue
  while IFS= read -r hit; do
    [ -n "$hit" ] && ADAPTOR_HITS+=("$hit")
  done < <(
    find "$root" -maxdepth 4 \
      \( -iname '*agent*' -o -iname '*adapt*' -o -iname '*hook*' \
         -o -iname '*extension*' -o -iname 'orca-plugin.json' \) \
      -not -path '*/node_modules/*' \
      -not -path '*/Cache/*' -not -path '*/Code Cache/*' \
      -not -path '*/GPUCache/*' -not -path '*/Crashpad/*' \
      2>/dev/null | sort | head -60
  )
done

if [ "${#ADAPTOR_HITS[@]}" -eq 0 ]; then
  echo 'no agent adaptor / hook / extension paths found'
else
  for hit in "${ADAPTOR_HITS[@]}"; do
    if [ -d "$hit" ]; then
      printf '\ndir: %s\n' "$(echo "$hit" | tilde)"
      # A directory match is the interesting case: list what is inside it,
      # which the previous name-pattern-only search missed entirely.
      ls -la "$hit" 2>/dev/null | head -25 | tilde
    else
      printf 'file: %-60s %s bytes\n' "$(echo "$hit" | tilde)" "$(wc -c <"$hit" 2>/dev/null || echo '?')"
    fi
  done
fi

section 'Adaptor sources'
if [ "${#ADAPTOR_HITS[@]}" -eq 0 ]; then
  echo 'nothing to read'
elif [ "$WITH_SOURCES" -eq 0 ]; then
  echo 'not included. To send the adaptor source as well, re-run with:'
  echo '  bash scripts/orca/collect-orca-info.sh --with-sources > orca-info.txt'
  echo 'and review the file before sharing it.'
else
  printed=0
  # Text sources only: the interface shape lives in code and manifests, and a
  # binary or a database would only corrupt the report.
  for hit in "${ADAPTOR_HITS[@]}"; do
    [ -d "$hit" ] && continue
    case "${hit##*.}" in
      ts|tsx|js|mjs|cjs|json|jsonc|yaml|yml|sh|bash|zsh|py|toml|md|txt) ;;
      *) continue ;;
    esac
    [ -r "$hit" ] || { printf '\n--- %s --- (not readable)\n' "$(echo "$hit" | tilde)"; continue; }
    if [ "$printed" -ge "$MAX_SOURCE_FILES" ]; then
      printf '\n(stopped after %s files)\n' "$MAX_SOURCE_FILES"
      break
    fi
    size="$(wc -c <"$hit" 2>/dev/null || echo 0)"
    printf '\n--- %s (%s bytes) ---\n' "$(echo "$hit" | tilde)" "$size"
    [ "$size" -gt "$MAX_FILE_BYTES" ] && printf '(first %s bytes only)\n' "$MAX_FILE_BYTES"
    head -c "$MAX_FILE_BYTES" "$hit" | redact
    printed=$((printed + 1))
  done
  [ "$printed" -eq 0 ] && echo 'no readable text sources among the matches'
fi

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
for root in "${ORCA_ROOTS[@]}"; do
  [ -d "$root" ] || continue
  find "$root" -maxdepth 4 -type d -name 'codex*home*' 2>/dev/null | head -10 | tilde
done

section 'Orca state (read-only queries)'
if [ -n "$ORCA_BIN" ]; then
  for query in 'account list' 'project list' 'orchestration run-list'; do
    printf '\n$ %s %s --json\n' "$ORCA_BIN" "$query"
    # shellcheck disable=SC2086 -- the query is a fixed, space-separated command.
    # Base64 icons are elided: they are megabytes of noise, not design input.
    timeout 20 "$ORCA_BIN" $query --json 2>&1 \
      | sed -E 's#"(src|icon|image)": "data:[^"]{40,}"#"\1": "<data-uri elided>"#' \
      | head -60 | tilde
  done
else
  echo 'skipped: Orca IDE CLI not found (nothing was run)'
fi

section 'Done'
echo 'Nothing was modified.'
if [ "$WITH_SOURCES" -eq 1 ]; then
  echo 'This report contains file contents. Review it before sharing.'
else
  echo 'Review before sharing, then send this file.'
fi
