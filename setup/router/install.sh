#!/usr/bin/env bash
# Wire a second Claude account (claude2) onto the same router and config as claude1.
#
# The router itself (caveman-proxy on 127.0.0.1:8787) and its settings.json entries are
# installed by the caveman CLI — see docs/claude-router.md. This script only does the
# part caveman does not: make ~/.claude-account2 share ~/.claude, and copy the
# per-account MCP registrations (incl. caveman's recovery MCP) into its .claude.json.
#
# Dry run by default; pass --apply to change anything. Close every claude2 session
# first: a live session rewrites .claude.json and will race the MCP copy.
# The agent cannot run --apply itself (targets sit under ~/.claude*). Run it yourself.
set -euo pipefail

APPLY=0
[ "${1:-}" = "--apply" ] && APPLY=1

SRC="$HOME/.claude"
DST="${CLAUDE2_DIR:-$HOME/.claude-account2}"
LINKS=(settings.json settings.local.json CLAUDE.md RTK.md agents skills plugins)

command -v jq >/dev/null || { echo "jq required"; exit 1; }
run() { if [ "$APPLY" = 1 ]; then "$@"; else echo "  would: $*"; fi; }

echo "== 1. shared config symlinks ($DST -> $SRC) =="
[ -d "$DST" ] || run mkdir -p "$DST"
for f in "${LINKS[@]}"; do
  if [ "$(readlink "$DST/$f" 2>/dev/null)" = "$SRC/$f" ]; then
    echo "  ok      $f"
  elif [ -e "$DST/$f" ] || [ -L "$DST/$f" ]; then
    # Never clobber a real file: it may hold settings only claude2 has.
    echo "  CONFLICT $DST/$f exists and is not a link to $SRC/$f — move it aside by hand"
  else
    run ln -s "$SRC/$f" "$DST/$f"
  fi
done
# .caveman-active / .ponytail-active stay per-account files: caveman refuses symlinked
# flags, and SessionStart rewrites both from the shared defaults.

echo "== 2. MCP servers (~/.claude.json -> $DST/.claude.json) =="
SRCJSON="$HOME/.claude.json"
DSTJSON="$DST/.claude.json"
[ -f "$DSTJSON" ] || { echo "  $DSTJSON missing — log in once with claude2, then re-run"; exit 1; }
missing=$(jq -r --slurpfile d "$DSTJSON" \
  '.mcpServers // {} | keys - ($d[0].mcpServers // {} | keys) | .[]' "$SRCJSON")
if [ -z "$missing" ]; then
  echo "  ok      all claude1 MCP servers present"
else
  echo "  missing: $(echo "$missing" | tr '\n' ' ')"
  if [ "$APPLY" = 1 ]; then
    cp "$DSTJSON" "$DSTJSON.bak-$(date +%Y%m%d-%H%M%S)"
    jq --slurpfile s "$SRCJSON" '.mcpServers = ((.mcpServers // {}) + $s[0].mcpServers)' \
      "$DSTJSON" > "$DSTJSON.tmp" && mv "$DSTJSON.tmp" "$DSTJSON"
    echo "  copied"
  else
    echo "  would: merge them into $DSTJSON (backup first)"
  fi
fi

echo "== 3. router =="
if ss -tln 'sport = :8787' | grep -q 8787; then
  echo "  ok      listener on 127.0.0.1:8787"
else
  echo "  DOWN    nothing on :8787 — every claude session will fail until it is back."
  echo "          Start it with \`caveman claude\` (it launches the proxy), or remove"
  echo "          ANTHROPIC_BASE_URL from ~/.claude/settings.json to go direct."
fi

[ "$APPLY" = 1 ] || printf '\nDry run. Re-run with --apply.\n'
