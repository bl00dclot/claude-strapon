# effort-router: UserPromptSubmit hook that tags each prompt with a complexity level.
# The hook runs this repo's copy, by absolute path.
effort_router_deps="claude-code"
EFFORT_ROUTER=$ROOT/files/effort-router.sh

effort_router_cmds() { settings_get '[.hooks.UserPromptSubmit[]?.hooks[]?.command | select(test("effort-router"))] | .[]'; }

effort_router_detect() {
  local cmds; cmds=$(effort_router_cmds)
  if [ "$cmds" = "$EFFORT_ROUTER" ]; then echo "ok	hook -> $EFFORT_ROUTER"
  elif [ -n "$cmds" ]; then echo "partial	hook points elsewhere: $(echo "$cmds" | tr '\n' ' ')"
  else echo "missing	no UserPromptSubmit hook"; fi
}
effort_router_plan() {
  echo "settings.json: drop UserPromptSubmit entries running effort-router, add $EFFORT_ROUTER (timeout 10)"
}
effort_router_apply() {
  json_edit "$SETTINGS" '
    .hooks //= {}
    | .hooks.UserPromptSubmit = ((.hooks.UserPromptSubmit // [])
        | map(select([.hooks[]?.command | test("effort-router")] | any | not))
        + [{hooks: [{type: "command", command: $cmd, timeout: 10}]}])' --arg cmd "$EFFORT_ROUTER"
}
