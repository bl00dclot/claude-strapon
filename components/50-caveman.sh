# caveman: compression proxy on 127.0.0.1:8787 that every Claude session routes through
# (settings env ANTHROPIC_BASE_URL), its native hooks, and its recovery MCP server.
# If the route is set and nothing listens on :8787, every session in both accounts fails,
# so the proxy also gets a systemd user unit that restarts it.
caveman_deps="node22 claude-code"
CAVEMAN_VERSION=2.1.0
CAVEMAN_ROUTE=http://127.0.0.1:8787/w/claude
CAVEMAN_UNIT=$HOME/.config/systemd/user/caveman-proxy.service

caveman_cli() { command -v caveman 2>/dev/null || { local b; b=$(node22_bin); [ -n "$b" ] && [ -x "$b/caveman" ] && echo "$b/caveman"; }; }
caveman_up() { ss -tlnH 'sport = :8787' 2>/dev/null | grep -q .; }
caveman_bins() { [ -x "$HOME/.caveman/bin/caveman-proxy" ]; }
caveman_route() { [ "$(settings_get '.env.ANTHROPIC_BASE_URL // empty')" = "$CAVEMAN_ROUTE" ]; }
caveman_hooks() { settings_get '[.hooks[]?[]?.hooks[]?.command] | map(select(test("caveman-proxy.*native-hook"))) | length' | grep -qv '^0$'; }
caveman_unit() { systemctl --user is-enabled caveman-proxy.service >/dev/null 2>&1; }
caveman_mcp() { claude_as "$CLAUDE_DIR" mcp get caveman >/dev/null 2>&1; }

caveman_detect() {
  [ -n "$(caveman_cli)" ] || { echo "missing	caveman CLI not installed"; return; }
  if caveman_route && ! caveman_up; then
    echo "broken	route set but nothing on :8787 — every claude session fails"; return
  fi
  local fix=() you=()
  caveman_bins  || you+=("proxy binaries")
  caveman_route || you+=("route")
  caveman_hooks || you+=("hooks")
  caveman_bins && ! caveman_unit && fix+=("not a service")
  caveman_mcp   || fix+=("MCP 'caveman' missing")
  local up; caveman_up && up=":8787 up" || up=":8787 down"
  if [ ${#fix[@]} -gt 0 ]; then
    echo "partial	$up · $(IFS=,; echo "${fix[*]}" | sed 's/,/ · /g')${you[*]:+ · run \`caveman claude\` once for: ${you[*]}}"
  elif [ ${#you[@]} -gt 0 ]; then
    echo "needs	$up · run \`caveman claude\` once to install: ${you[*]}"
  else
    echo "ok	$up · route · hooks · service · MCP"
  fi
}

caveman_plan() {
  [ -n "$(caveman_cli)" ] || echo "npm i -g @caveman-ai/cli@$CAVEMAN_VERSION      (with Node 22 first on PATH)"
  if ! caveman_unit; then
    echo "install files/caveman-proxy.service -> $CAVEMAN_UNIT"
    echo "systemctl --user daemon-reload && systemctl --user enable --now caveman-proxy.service"
    caveman_up && echo "  (a proxy already holds :8787; the unit retries every 15 s and takes over if it exits)"
  fi
  caveman_mcp || echo "caveman mcp install claude      (recovery MCP in account 1; claude2 copies it)"
  { caveman_bins && caveman_route && caveman_hooks; } || echo "then by hand: caveman claude      (installs proxy binaries, route, hooks)"
}

caveman_apply() {
  local b; b=$(node22_bin)
  [ -n "$(caveman_cli)" ] || PATH="$b:$PATH" npm i -g "@caveman-ai/cli@$CAVEMAN_VERSION"
  if caveman_bins && ! caveman_unit; then
    install -D -m0644 "$ROOT/files/caveman-proxy.service" "$CAVEMAN_UNIT"
    systemctl --user daemon-reload
    systemctl --user enable --now caveman-proxy.service || true   # bind fails while another proxy runs; Restart= retries
  fi
  caveman_mcp || PATH="$b:$PATH" "$(caveman_cli)" mcp install claude
  { caveman_bins && caveman_route && caveman_hooks; } || echo "  next, by hand: caveman claude   (installs proxy binaries, route, hooks), then re-run"
  return 0
}
