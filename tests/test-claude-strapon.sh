#!/usr/bin/env bash
# Runs claude-strapon against a throwaway HOME with stub claude/rtk/ss/systemctl/npm/curl/sudo,
# stdin closed. Nothing here can reach the real ~/.claude, the user's systemd, or the network.
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
export HOME=$T/home NVM_DIR=$T/home/.nvm
unset CLAUDE_CONFIG_DIR CLAUDE_DIR CLAUDE2_DIR
mkdir -p "$HOME/.claude" "$HOME/.claude-account2" "$T/bin" "$NVM_DIR/versions/node/v22.9.0/bin"
fail() { echo "FAIL: $*"; exit 1; }

# --- stubs -----------------------------------------------------------------------------
stub() { printf '#!/usr/bin/env bash\n%s\n' "$2" > "$T/bin/$1"; chmod +x "$T/bin/$1"; }
stub node 'echo v22.9.0'; cp "$T/bin/node" "$NVM_DIR/versions/node/v22.9.0/bin/node"
for c in npm curl sudo apt-get gum caveman; do stub "$c" "echo \"stub $c called: \$*\" >&2; exit 99"; done
rm "$T/bin/gum" "$T/bin/caveman"   # absent, not failing
stub ss 'exit 0'                    # nothing listening on :8787
stub systemctl 'exit 1'
stub pgrep 'exit 1'
stub rtk 'case $1 in gain) exit 0;; --version) echo "rtk 0.42.4";; init) printf "[ok] Hook: rtk hook claude\n[ok] settings.json: RTK hook configured\n";; esac'
# claude: MCP servers live in the account's .claude.json; plugins in a json file.
stub claude '
cfg=${CLAUDE_CONFIG_DIR:+$CLAUDE_CONFIG_DIR/.claude.json}; cfg=${cfg:-$HOME/.claude.json}
case "$1 $2" in
  "--version "*) echo "2.1.308 (Claude Code)";;
  "mcp get") jq -e --arg n "$3" ".mcpServers[\$n]" "$cfg" >/dev/null 2>&1 || exit 1; echo "Status: ✔ Connected";;
  "plugin list") cat "$HOME/plugins.json";;
  "plugin marketplace") printf "  ❯ claude-plugins-official\n  ❯ caveman\n  ❯ ponytail\n";;
  *) echo "stub claude: $*" >&2; exit 99;;
esac'
export PATH="$T/bin:/usr/bin:/bin"

jq -n '{mcpServers: {codegraph: {}, dune: {}}}' > "$HOME/.claude.json"
jq -n '{mcpServers: {codegraph: {}}, oauthAccount: {x: 1}}' > "$HOME/.claude-account2/.claude.json"
for f in settings.json settings.local.json CLAUDE.md RTK.md agents skills plugins; do
  ln -s "$HOME/.claude/$f" "$HOME/.claude-account2/$f"
done
grep -v '^#' "$ROOT/files/plugins.txt" | jq -R '{id: ., scope: "user", enabled: true}' \
  | jq -s '. + [{id: "stripe@claude-plugins-official", scope: "user", enabled: true}]' > "$HOME/plugins.json"
# a stale effort-router hook from the ops days, next to an unrelated hook that must survive
jq -n '{hooks: {UserPromptSubmit: [
  {hooks: [{type: "command", command: "/old/ops/effort-router.sh"}]},
  {hooks: [{type: "command", command: "other-hook"}]}]}}' > "$HOME/.claude/settings.json"

snap() { (cd "$T/home" && find . -type f -newer "$T/stamp" | sort); }
run() { "$ROOT/claude-strapon" "$@" </dev/null; }

# --- 1. dry run and no-terminal runs change nothing ----------------------------------
touch "$T/stamp"; sleep 1
out=$(run --dry-run) || fail "dry run exited non-zero"
grep -q 'Dry run: nothing changed' <<<"$out" || fail "dry run did not stop: $out"
grep -q 'remove: stripe' <<<"$out" || fail "board missed stripe removal"
grep -q 'MCP missing: dune' <<<"$out" || fail "board missed claude2 MCP gap"
grep -q 'hook points elsewhere' <<<"$out" || fail "board missed stale effort-router"
out=$(run) || fail "no-tty run exited non-zero"
grep -q 'No terminal: stopping after the preview' <<<"$out" || fail "no-tty run did not stop"
[ -z "$(snap)" ] || fail "read-only runs wrote: $(snap)"
echo "ok 1  dry run + no-tty change nothing"

# --- 2. apply the file-only components ----------------------------------------------
ONLY=effort-router,model,agents-skills,claude2
out=$(run --yes --only "$ONLY") || fail "apply exited non-zero: $out"
S=$HOME/.claude/settings.json
[ "$(jq -r '.model' "$S")" = 'opus[1m]' ] || fail "model not set"
[ "$(jq -r '[.hooks.UserPromptSubmit[].hooks[].command] | join(",")' "$S")" = "other-hook,$ROOT/files/effort-router.sh" ] \
  || fail "hooks wrong: $(jq -c .hooks "$S")"
[ -L "$HOME/.claude-account2/settings.json" ] || fail "apply replaced the account-2 settings link"
[ "$(jq -r '.mcpServers | keys | join(",")' "$HOME/.claude-account2/.claude.json")" = codegraph,dune ] || fail "MCP not merged"
[ "$(jq -r '.oauthAccount.x' "$HOME/.claude-account2/.claude.json")" = 1 ] || fail "merge lost account state"
[ "$(stat -c %a "$HOME/.claude-account2/.claude.json")" = 600 ] || fail ".claude.json not 0600"
cmp -s "$ROOT/files/agents/state-author.md" "$HOME/.claude/agents/state-author.md" || fail "agents not installed"
ls "$S".bak-* >/dev/null 2>&1 || fail "no settings backup"
grep -q 'All picked components read ok' <<<"$out" || fail "rescan not ok: $out"
echo "ok 2  apply wrote settings, merged MCP, kept links + account state, backed up"

# --- 3. second run is a no-op ---------------------------------------------------------
n=$(ls "$S".bak-* | wc -l)
out=$(run --yes --only "$ONLY") || fail "re-run exited non-zero"
grep -q 'Nothing to fix' <<<"$out" || fail "re-run was not a no-op: $out"
[ "$(ls "$S".bak-* | wc -l)" = "$n" ] || fail "re-run took another backup"
echo "ok 3  re-run: nothing to fix, no new backup"

# --- 4. a bad write never lands -------------------------------------------------------
before=$(cat "$S")
( . "$ROOT/lib/claude.sh"; json_edit "$S" '.model = ' ) 2>/dev/null && fail "bad jq filter succeeded"
[ "$(cat "$S")" = "$before" ] || fail "bad write changed settings.json"
ls "$S".tmp.* >/dev/null 2>&1 && fail "tmp file left behind"
echo "ok 4  invalid edit leaves settings.json untouched"

# --- 5. a route without a proxy is reported broken -------------------------------------
stub caveman 'exit 0'
json_tmp=$(mktemp); jq '.env.ANTHROPIC_BASE_URL = "http://127.0.0.1:8787/w/claude"' "$S" > "$json_tmp" && mv "$json_tmp" "$S"
out=$(run --dry-run --only caveman) || fail "caveman dry run failed"
grep -q 'route set but nothing on :8787' <<<"$out" || fail "dead route not flagged: $out"
echo "ok 5  route with nothing on :8787 is flagged broken"

# --- 6. interactive path, plain prompts, under a pseudo-terminal ------------------------
json_tmp=$(mktemp); jq 'del(.model)' "$S" > "$json_tmp" && mv "$json_tmp" "$S"
out=$(printf '\ny\n' | script -qec "'$ROOT/claude-strapon' --only model" /dev/null | tr -d '\r') \
  || fail "interactive run failed: $out"
grep -q 'numbers to toggle' <<<"$out" || fail "plain picker not shown: $out"
grep -q 'Apply these? \[y/N\]' <<<"$out" || fail "no confirm prompt"
[ "$(jq -r '.model' "$S")" = 'opus[1m]' ] || fail "interactive apply did not set model"
echo "ok 6  interactive: pick, confirm, apply under a terminal"
echo "PASS"
