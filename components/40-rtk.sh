# rtk (Rust Token Killer): PreToolUse(Bash) hook that compresses command output.
# Another project is also called rtk (Rust Type Kit); `rtk gain` tells them apart.
rtk_deps="claude-code"
rtk_detect() {
  command -v rtk >/dev/null || { echo "missing	not installed"; return; }
  rtk gain >/dev/null 2>&1 || { echo "broken	\`rtk\` on PATH is not Rust Token Killer (rtk gain fails)"; return; }
  local show v
  show=$(rtk init --show 2>/dev/null)
  v=$(rtk --version 2>/dev/null | awk '{print $2}')
  if grep -q '^\[ok\] Hook' <<<"$show" && grep -q '^\[ok\] settings.json' <<<"$show"; then
    echo "ok	$v · hook ok"
  else
    echo "partial	$v · hook not wired"
  fi
}
rtk_plan() {
  command -v rtk >/dev/null || echo "curl -fsSL https://raw.githubusercontent.com/rtk-ai/rtk/master/install.sh | sh"
  echo "rtk init -g --auto-patch      (hook + RTK.md + @RTK.md in $CLAUDE_DIR)"
}
rtk_apply() {
  if ! command -v rtk >/dev/null; then
    curl -fsSL https://raw.githubusercontent.com/rtk-ai/rtk/master/install.sh | sh
    export PATH="$HOME/.local/bin:$PATH"
  fi
  rtk init -g --auto-patch
}
