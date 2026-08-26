#!/usr/bin/env bash
# Install the derived subagent roster and wire its hooks.
#
# The agent cannot run this itself: every target is under ~/.claude, which
# settings.json `deny` and CLAUDE.md rule 3 place off limits. Run it yourself.
#
# Idempotent. Backs up settings.json before touching it.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
AGENTS="$HOME/.claude/agents"
SETTINGS="$HOME/.claude/settings.json"

command -v jq >/dev/null || { echo "jq required"; exit 1; }

echo "== 1. roles =="
install -d "$AGENTS"
# Every role file in setup/agents/, discovered rather than listed. The 2026-08-25 pass added
# ssh-hazard-table and this list was not updated, so it was staged and never
# installed — one more inert artifact on top of the roles nobody had installed.
find "$ROOT/setup/agents" -maxdepth 1 -name '*.md' ! -name 'README.md' \
  -exec install -m0644 {} "$AGENTS/" \;
ls -1 "$AGENTS"

echo "== 2. hooks =="
cp "$SETTINGS" "$SETTINGS.bak-$(date +%Y%m%d-%H%M%S)"
jq --arg tc "$ROOT/scripts/agent-taskcompleted.sh" \
   --arg ga "$ROOT/scripts/git-add-scope-guard.sh" '
  .hooks //= {}
  | .hooks.TaskCompleted = [{hooks:[{type:"command", command:$tc}]}]
  | .hooks.PreToolUse = ((.hooks.PreToolUse // [])
      | map(select(.hooks[0].command != $ga))
      + [{matcher:"Bash", hooks:[{type:"command", command:$ga}]}])
' "$SETTINGS" > "$SETTINGS.tmp" && mv "$SETTINGS.tmp" "$SETTINGS"
jq -r '.hooks | keys[]' "$SETTINGS" | sed 's/^/  /'

echo "== 3. dependency =="
if command -v shellcheck >/dev/null; then
  echo "  shellcheck present"
else
  echo "  shellcheck MISSING — the TaskCompleted hook will warn and skip its"
  echo "  script checks until you run: sudo apt install shellcheck"
fi

cat <<'MSG'

Done. Absolute paths were baked in, so the hooks fire regardless of cwd.
Roles are global (~/.claude/agents); move them to <repo>/.claude/agents to scope
them to one repo instead.

Verify: start a new session and check `/agents` lists evidence-auditor,
runbook-author, ssh-hazard-table and state-author, then try `git add -A` — it
should be refused.
MSG
