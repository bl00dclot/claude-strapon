# statusline: caveman's mode badge. Cosmetic, so not pre-ticked. The script lives under
# the plugin cache and its path changes when the caveman plugin updates.
statusline_optional=1
statusline_deps="plugins"

statusline_script() {
  ls -d "$CLAUDE_DIR"/plugins/cache/caveman/caveman/*/src/hooks/caveman-statusline.sh 2>/dev/null | sort -V | tail -n1
}
statusline_detect() {
  local cmd path; cmd=$(settings_get '.statusLine.command // empty')
  if [ -n "$cmd" ]; then
    path=$(sed -E 's/^bash +"?([^"]+)"?$/\1/' <<<"$cmd")
    if [ -f "$path" ]; then echo "ok	$(basename "$path")"
    else echo "broken	points at a missing script (plugin updated?)"; fi
    return
  fi
  [ -n "$(statusline_script)" ] && echo "missing	not configured" || echo "missing	not configured; caveman plugin not installed yet"
}
statusline_plan() { echo "settings.json: statusLine = bash \"$(statusline_script)\""; }
statusline_apply() {
  local s; s=$(statusline_script)
  [ -n "$s" ] || { echo "  caveman plugin not installed"; return 1; }
  json_edit "$SETTINGS" '.statusLine = {type: "command", command: ("bash \"" + $s + "\"")}' --arg s "$s"
}
