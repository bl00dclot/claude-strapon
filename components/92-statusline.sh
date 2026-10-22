# statusline: caveman's mode badge. Cosmetic, so not pre-ticked. The script lives under
# the plugin cache, whose path changes on every caveman plugin update, so the command
# looks up the newest copy each time it runs instead of pinning one.
statusline_optional=1
statusline_deps="plugins"
# shellcheck disable=SC2016  # expanded by the statusline shell, not here
STATUSLINE_CMD='bash -c '\''s=$(ls -1t "$HOME"/.claude/plugins/cache/caveman/caveman/*/src/hooks/caveman-statusline.sh 2>/dev/null | awk "NR==1"); [ -n "$s" ] && exec bash "$s"'\'

statusline_script() {
  ls -d "$CLAUDE_DIR"/plugins/cache/caveman/caveman/*/src/hooks/caveman-statusline.sh 2>/dev/null | sort -V | tail -n1
}
statusline_detect() {
  local cmd path; cmd=$(settings_get '.statusLine.command // empty')
  if [ -n "$cmd" ]; then
    # A pinned `bash "<path>"` breaks on the next plugin update; anything else is judged by you.
    if [[ $cmd =~ ^bash\ +\"?(/[^\"]+)\"?$ ]]; then
      path=${BASH_REMATCH[1]}
      if [ -f "$path" ]; then echo "partial	pinned to one plugin version: $(basename "$path")"
      else echo "broken	missing script: $path (plugin updated?)"; fi
    elif [ "$cmd" = "$STATUSLINE_CMD" ]; then echo "ok	caveman badge (follows plugin updates)"
    else echo "ok	custom: $cmd"; fi
    return
  fi
  [ -n "$(statusline_script)" ] && echo "missing	not configured" || echo "missing	not configured; caveman plugin not installed yet"
}
statusline_plan() { echo "settings.json: statusLine = $STATUSLINE_CMD"; }
statusline_apply() {
  [ -n "$(statusline_script)" ] || { echo "  caveman plugin not installed"; return 1; }
  json_edit "$SETTINGS" '.statusLine = {type: "command", command: $c}' --arg c "$STATUSLINE_CMD"
}
