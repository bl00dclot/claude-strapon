# claude2: second account (CLAUDE_CONFIG_DIR=~/.claude-account2) sharing account 1's
# settings, hooks, skills and plugins by symlink. Its .claude.json (login + MCP
# servers) cannot be a link, so missing MCP servers are copied across.
CLAUDE2_LINKS=(settings.json settings.local.json CLAUDE.md RTK.md agents skills plugins)

claude2_missing_mcp() {
  [ -f "$CLAUDE2_DIR/.claude.json" ] && [ -f "$CLAUDE_JSON" ] || return 0
  jq -r --slurpfile d "$CLAUDE2_DIR/.claude.json" \
    '.mcpServers // {} | keys - ($d[0].mcpServers // {} | keys) | .[]' "$CLAUDE_JSON"
}

claude2_detect() {
  [ -d "$CLAUDE2_DIR" ] || { echo "needs	no second account; to add one, log in: CLAUDE_CONFIG_DIR=$CLAUDE2_DIR claude"; return; }
  local f ok=0 conflict=()
  for f in "${CLAUDE2_LINKS[@]}"; do
    if [ "$(readlink "$CLAUDE2_DIR/$f" 2>/dev/null)" = "$CLAUDE_DIR/$f" ]; then ok=$((ok + 1))
    elif [ -e "$CLAUDE2_DIR/$f" ] || [ -L "$CLAUDE2_DIR/$f" ]; then conflict+=("$f"); fi
  done
  if [ ${#conflict[@]} -gt 0 ]; then
    echo "broken	not links, move aside by hand: ${conflict[*]}"; return
  fi
  [ -f "$CLAUDE2_DIR/.claude.json" ] || { echo "needs	log in once: CLAUDE_CONFIG_DIR=$CLAUDE2_DIR claude"; return; }
  local miss; miss=$(claude2_missing_mcp | paste -sd' ')
  if [ $ok -lt ${#CLAUDE2_LINKS[@]} ]; then echo "partial	$ok/${#CLAUDE2_LINKS[@]} links${miss:+ · MCP missing: $miss}"
  elif [ -n "$miss" ]; then echo "partial	links ok · MCP missing: $miss"
  else echo "ok	${#CLAUDE2_LINKS[@]}/${#CLAUDE2_LINKS[@]} links · MCP in sync"; fi
}

claude2_plan() {
  local f
  for f in "${CLAUDE2_LINKS[@]}"; do
    [ -e "$CLAUDE2_DIR/$f" ] || [ -L "$CLAUDE2_DIR/$f" ] || echo "ln -s $CLAUDE_DIR/$f $CLAUDE2_DIR/$f"
  done
  local miss; miss=$(claude2_missing_mcp | paste -sd' ')
  [ -z "$miss" ] || echo "copy MCP servers into $CLAUDE2_DIR/.claude.json: $miss"
  [ -f "$CLAUDE2_DIR/.claude.json" ] || echo "then by hand: CLAUDE_CONFIG_DIR=$CLAUDE2_DIR claude   (log in), re-run"
}

claude2_apply() {
  local f
  install -d "$CLAUDE2_DIR"
  for f in "${CLAUDE2_LINKS[@]}"; do
    [ -e "$CLAUDE2_DIR/$f" ] || [ -L "$CLAUDE2_DIR/$f" ] || ln -s "$CLAUDE_DIR/$f" "$CLAUDE2_DIR/$f"
  done
  [ -f "$CLAUDE2_DIR/.claude.json" ] || { echo "  log in once as claude2, then re-run to copy MCP servers"; return 0; }
  [ -z "$(claude2_missing_mcp)" ] || json_edit "$CLAUDE2_DIR/.claude.json" \
    '.mcpServers = ($s[0].mcpServers + (.mcpServers // {}))' --slurpfile s "$CLAUDE_JSON"
}
