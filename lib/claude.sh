# Where each Claude account keeps its state, and the one safe way to rewrite its JSON.
#
# Account 1 is the default config dir (~/.claude + ~/.claude.json). Account 2 is
# CLAUDE_CONFIG_DIR=~/.claude-account2, whose settings/agents/skills/plugins are
# symlinks into ~/.claude; only its .claude.json (login + MCP servers) is its own.

CLAUDE_DIR=${CLAUDE_DIR:-$HOME/.claude}
CLAUDE2_DIR=${CLAUDE2_DIR:-$HOME/.claude-account2}
CLAUDE_JSON=$HOME/.claude.json
SETTINGS=$CLAUDE_DIR/settings.json
NVM_DIR=${NVM_DIR:-$([ -d "$HOME/.config/nvm" ] && echo "$HOME/.config/nvm" || echo "$HOME/.nvm")}

settings_get() { [ -f "$SETTINGS" ] && jq -r "$1" "$SETTINGS" 2>/dev/null || true; }

# json_edit FILE FILTER [jq args...]
# Backup, rewrite through a tmp file, refuse to install anything jq cannot re-read.
# Claude Code silently discards a settings.json it cannot parse, so a bad write here
# would switch off every hook at once. Writes through symlinks to the real file.
json_edit() {
  local file filter=$2 tmp; file=$(readlink -f "$1"); shift 2
  (
    umask 077
    [ -f "$file" ] || echo '{}' > "$file"
    cp -p "$file" "$file.bak-$(date +%Y%m%d-%H%M%S)"
    tmp=$(mktemp "$file.tmp.XXXXXX")
    jq "$@" "$filter" "$file" > "$tmp" && jq empty "$tmp" && mv "$tmp" "$file" || { rm -f "$tmp"; exit 1; }
  )
}

# Accounts that exist on this machine, by config dir.
accounts() {
  echo "$CLAUDE_DIR"
  [ -f "$CLAUDE2_DIR/.claude.json" ] && echo "$CLAUDE2_DIR"
  return 0
}

# claude_as DIR ARGS... — run the claude CLI as the account that owns DIR.
claude_as() {
  local dir=$1; shift
  if [ "$dir" = "$CLAUDE_DIR" ]; then env -u CLAUDE_CONFIG_DIR claude "$@"
  else CLAUDE_CONFIG_DIR=$dir claude "$@"; fi
}

# Newest Node 22 under nvm, as a bin dir; empty if none.
node22_bin() {
  local d
  d=$(ls -d "$NVM_DIR"/versions/node/v22.*/ 2>/dev/null | sort -V | tail -n1)
  [ -n "$d" ] && echo "${d%/}/bin"
  return 0
}

# Interactive claude sessions, so a .claude.json write can warn about the race.
claude_sessions() { pgrep -x claude 2>/dev/null | wc -l; }
