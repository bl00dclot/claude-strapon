# codegraph: code-graph MCP server. Registered with absolute node + shim paths: a bare
# `codegraph` command fails with ENOENT because Claude Code never loads nvm.
# Tools appear only in repos that ran `codegraph init` (left to you, per repo).
codegraph_deps="node22 claude-code"
CODEGRAPH_PKG=@colbymchenry/codegraph

codegraph_shim() { local b; b=$(node22_bin); [ -n "$b" ] && echo "${b%/bin}/lib/node_modules/$CODEGRAPH_PKG/npm-shim.js"; }
codegraph_status() { claude_as "$1" mcp get codegraph 2>/dev/null; }

codegraph_detect() {
  local shim; shim=$(codegraph_shim)
  [ -n "$shim" ] && [ -f "$shim" ] || { echo "missing	package not installed under Node 22"; return; }
  local d out bad=() v
  v=$(jq -r .version "$(dirname "$shim")/package.json" 2>/dev/null)
  for d in $(accounts); do
    out=$(codegraph_status "$d")
    if [ -z "$out" ]; then bad+=("$(basename "$d"): not registered")
    elif ! grep -q 'Connected' <<<"$out"; then bad+=("$(basename "$d"): $(grep -o 'Status: .*' <<<"$out")")
    elif ! grep -q "Args: $shim" <<<"$out"; then bad+=("$(basename "$d"): not the absolute-path form")
    fi
  done
  if [ ${#bad[@]} -eq 0 ]; then echo "ok	$v · MCP connected · per repo: codegraph init"
  else echo "partial	$v · $(IFS=';'; echo "${bad[*]}")"; fi
}

codegraph_plan() {
  local b; b=$(node22_bin)
  [ -f "$(codegraph_shim)" ] || echo "npm i -g $CODEGRAPH_PKG      (Node 22)"
  echo "for each account: claude mcp remove codegraph -s user; claude mcp add codegraph -s user -- $b/node $(codegraph_shim) serve --mcp"
}

codegraph_apply() {
  local b d; b=$(node22_bin)
  [ -f "$(codegraph_shim)" ] || PATH="$b:$PATH" npm i -g "$CODEGRAPH_PKG"
  for d in $(accounts); do
    claude_as "$d" mcp remove codegraph -s user >/dev/null 2>&1 || true
    claude_as "$d" mcp add codegraph -s user -- "$b/node" "$(codegraph_shim)" serve --mcp
  done
}
