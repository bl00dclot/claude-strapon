# Node 22 under nvm: caveman's CLI and codegraph are npm globals pinned to it, and the
# MCP registrations use its absolute paths so they never depend on nvm being loaded.
node22_detect() {
  local b; b=$(node22_bin)
  if [ -n "$b" ]; then echo "ok	$("$b/node" --version) under $NVM_DIR"; return; fi
  if [ -s "$NVM_DIR/nvm.sh" ]; then echo "missing	nvm present, no Node 22"
  else echo "needs	nvm not found at $NVM_DIR; install nvm, then re-run"; fi
}
node22_plan() { echo ". $NVM_DIR/nvm.sh && nvm install 22"; }
node22_apply() {
  # nvm.sh trips over set -u and returns non-zero from harmless checks.
  ( set +eu; . "$NVM_DIR/nvm.sh"; nvm install 22 )
  [ -n "$(node22_bin)" ]
}
