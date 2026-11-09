# caveman: compression proxy on 127.0.0.1:8787 that Claude sessions route through, its
# native hooks, and its recovery MCP server.
# The route is set per launch by a shell function (files/claude-route.sh, sourced from
# ~/.bashrc), not by settings.json env: Remote Control refuses a non-api.anthropic.com
# base URL and settings env beats the shell, so a settings route blocks Remote Control
# for both accounts. `caveman claude` writes the settings route on first enable; apply
# moves it. If a route is set and nothing listens on :8787, every routed session fails,
# so the proxy also gets a systemd user unit that restarts it.
caveman_deps="node22 claude-code"
CAVEMAN_VERSION=2.1.0
CAVEMAN_ROUTE=http://127.0.0.1:8787/w/claude
CAVEMAN_UNIT=$HOME/.config/systemd/user/caveman-proxy.service
CAVEMAN_RC_FILE=$ROOT/files/claude-route.sh
CAVEMAN_RC_LINE=". \"$CAVEMAN_RC_FILE\""
BASHRC=$HOME/.bashrc

caveman_cli() { command -v caveman 2>/dev/null || { local b; b=$(node22_bin); [ -n "$b" ] && [ -x "$b/caveman" ] && echo "$b/caveman"; }; }
caveman_up() { ss -tlnH 'sport = :8787' 2>/dev/null | grep -q .; }
caveman_bins() { [ -x "$HOME/.caveman/bin/caveman-proxy" ]; }
caveman_settings_route() { [ "$(settings_get '.env.ANTHROPIC_BASE_URL // empty')" = "$CAVEMAN_ROUTE" ]; }
caveman_shell_route() { grep -qxF "$CAVEMAN_RC_LINE" "$BASHRC" 2>/dev/null && grep -qF "$CAVEMAN_ROUTE" "$CAVEMAN_RC_FILE"; }
caveman_hooks() { settings_get '[.hooks[]?[]?.hooks[]?.command] | map(select(test("caveman-proxy.*native-hook"))) | length' | grep -qv '^0$'; }
caveman_unit() { systemctl --user is-enabled caveman-proxy.service >/dev/null 2>&1; }
caveman_mcp() { claude_as "$CLAUDE_DIR" mcp get caveman >/dev/null 2>&1; }

caveman_detect() {
  [ -n "$(caveman_cli)" ] || { echo "missing	caveman CLI not installed"; return; }
  if { caveman_settings_route || caveman_shell_route; } && ! caveman_up; then
    echo "broken	route set but nothing on :8787 — every claude session fails"; return
  fi
  local fix=() you=()
  caveman_bins  || you+=("proxy binaries")
  caveman_hooks || you+=("hooks")
  caveman_bins && ! caveman_unit && fix+=("not a service")
  caveman_settings_route && fix+=("route in settings.json blocks Remote Control")
  caveman_bins && ! caveman_shell_route && fix+=("shell route missing")
  caveman_mcp   || fix+=("MCP 'caveman' missing")
  local up; caveman_up && up=":8787 up" || up=":8787 down"
  if [ ${#fix[@]} -gt 0 ]; then
    echo "partial	$up · $(IFS=,; echo "${fix[*]}" | sed 's/,/ · /g')${you[*]:+ · run \`caveman claude\` once for: ${you[*]}}"
  elif [ ${#you[@]} -gt 0 ]; then
    echo "needs	$up · run \`caveman claude\` once to install: ${you[*]}"
  else
    echo "ok	$up · shell route (Remote Control direct) · hooks · service · MCP"
  fi
}

caveman_plan() {
  [ -n "$(caveman_cli)" ] || echo "npm i -g @caveman-ai/cli@$CAVEMAN_VERSION      (with Node 22 first on PATH)"
  if ! caveman_unit; then
    echo "install files/caveman-proxy.service -> $CAVEMAN_UNIT"
    echo "systemctl --user daemon-reload && systemctl --user enable --now caveman-proxy.service"
    caveman_up && echo "  (a proxy already holds :8787; the unit retries every 15 s and takes over if it exits)"
  fi
  if caveman_bins; then
    caveman_shell_route || echo "append to $BASHRC: $CAVEMAN_RC_LINE"
    caveman_settings_route && echo "remove env.ANTHROPIC_BASE_URL + env._CLAUDE_CODE_ASSUME_FIRST_PARTY_BASE_URL from $SETTINGS   (Remote Control refuses a proxied base URL)"
    { caveman_shell_route && ! caveman_settings_route; } || echo "  then open a new shell: running sessions keep the old route"
  fi
  caveman_mcp || echo "caveman mcp install claude      (recovery MCP in account 1; claude2 copies it)"
  { caveman_bins && caveman_hooks; } || echo "then by hand: caveman claude      (installs proxy binaries, hooks), re-run to move its route"
  return 0
}

caveman_apply() {
  local b; b=$(node22_bin)
  [ -n "$(caveman_cli)" ] || PATH="$b:$PATH" npm i -g "@caveman-ai/cli@$CAVEMAN_VERSION"
  if caveman_bins && ! caveman_unit; then
    install -D -m0644 "$ROOT/files/caveman-proxy.service" "$CAVEMAN_UNIT"
    systemctl --user daemon-reload
    systemctl --user enable --now caveman-proxy.service || true   # bind fails while another proxy runs; Restart= retries
  fi
  # Shell route first, then drop the settings one, so no step leaves claude unrouted.
  if caveman_bins; then
    caveman_shell_route || printf '\n# caveman route for claude/claude2; Remote Control launches go direct (claude-strapon)\n%s\n' \
      "$CAVEMAN_RC_LINE" >> "$BASHRC"
    if caveman_settings_route; then
      json_edit "$SETTINGS" 'del(.env.ANTHROPIC_BASE_URL, .env._CLAUDE_CODE_ASSUME_FIRST_PARTY_BASE_URL)' || return 1
    fi
  fi
  caveman_mcp || PATH="$b:$PATH" "$(caveman_cli)" mcp install claude
  { caveman_bins && caveman_hooks; } || echo "  next, by hand: caveman claude   (installs proxy binaries, hooks), then re-run"
  return 0
}
