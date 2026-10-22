# gum: draws this wizard's prompts. Without it the wizard falls back to numbered prompts.
gum_detect() {
  if command -v gum >/dev/null; then echo "ok	$(gum --version 2>/dev/null | head -n1)"; return; fi
  local c; c=$(apt-cache policy gum 2>/dev/null | awk '/Candidate/{print $2}')
  if [ -n "$c" ] && [ "$c" != "(none)" ]; then echo "missing	not installed (apt has $c)"
  else echo "needs	not installed and apt has no gum package; see github.com/charmbracelet/gum"; fi
}
gum_plan() { echo "sudo apt-get install -y gum      (asks for your sudo password)"; }
gum_apply() {
  ui_interactive || { echo "  sudo needs a terminal; run: sudo apt-get install -y gum"; return 1; }
  sudo apt-get install -y gum
}
