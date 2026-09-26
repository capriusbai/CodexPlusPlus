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
if have orca; then
  printf 'path: %s\n' "$(command -v orca | tilde)"
  orca --version 2>&1 | head -3
else
  echo 'orca: not on PATH'
  # The desktop app ships the CLI; these are where it usually lands.
  for candidate in \
    "/Applications/Orca.app/Contents/Resources/app/cli" \
    "$HOME/.local/bin/orca" \
    "/usr/local/bin/orca" \
    "/opt/Orca/orca"; do
    [ -e "$candidate" ] && printf 'found unlinked: %s\n' "$(echo "$candidate" | tilde)"
  done
fi

section 'Application install'
for candidate in \
  "/Applications/Orca.app" \
  "$HOME/Applications/Orca.app" \
  "/opt/Orca" \
  "/usr/lib/orca" \
  "$HOME/.local/share/orca"; do
  if [ -e "$candidate" ]; then
    printf '%s\n' "$(echo "$candidate" | tilde)"
    du -sh "$candidate" 2>/dev/null | tilde
  fi
done
if have dpkg-query; then dpkg-query -W -f='deb: ${Package} ${Version}\n' orca 2>/dev/null; fi
if have brew; then brew list --cask 2>/dev/null | grep -i orca | sed 's/^/brew cask: /'; fi

section 'User data directories'
for candidate in \
  "$HOME/.orca" \
  "$HOME/.config/Orca" \
  "$HOME/.config/orca" \
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
for root in "$HOME/.orca" "$HOME/.config/Orca" "$HOME/Library/Application Support/Orca"; do
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
if have orca; then
  for query in 'account list' 'project list' 'orchestration run-list'; do
    printf '\n$ orca %s --json\n' "$query"
    # shellcheck disable=SC2086 -- the query is a fixed, space-separated command.
    timeout 20 orca $query --json 2>&1 | head -40 | tilde
  done
else
  echo 'skipped: orca CLI not available'
fi

section 'Done'
echo 'Nothing was modified. Review before sharing, then send this file.'
