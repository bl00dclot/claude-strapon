# Claude Code, native install (~/.local/bin/claude, self-updating).
claude_code_detect() {
  if command -v claude >/dev/null; then echo "ok	$(claude --version 2>/dev/null)"
  else echo "missing	not installed"; fi
}
claude_code_plan() { echo "curl -fsSL https://claude.ai/install.sh | bash"; }
claude_code_apply() {
  curl -fsSL https://claude.ai/install.sh | bash
  export PATH="$HOME/.local/bin:$PATH"
  command -v claude >/dev/null && echo "  first run: start \`claude\` once to log in"
}
