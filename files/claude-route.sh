# shellcheck shell=bash
# Sourced from ~/.bashrc by the claude-strapon caveman component.
#
# Routes Claude Code through caveman-proxy from the shell environment instead of from
# settings.json env. Remote Control refuses any ANTHROPIC_BASE_URL that is not
# api.anthropic.com (_CLAUDE_CODE_ASSUME_FIRST_PARTY_BASE_URL does not cover it), and
# settings.json env beats the shell, so a settings route left no way to start it.
#
# The route is exported, so everything started from this shell inherits it: scripts,
# `sh -c` launchers, `claude -p` workers, the claude2 alias. The claude() function
# strips it only for launches that start Remote Control, the same rule
# `caveman wrap claude` applies: `remote-control`/`rc` in first position, or
# `--remote-control[=name]` before `--`. Bypass the proxy with `unset ANTHROPIC_BASE_URL`.
export ANTHROPIC_BASE_URL=http://127.0.0.1:8787/w/claude _CLAUDE_CODE_ASSUME_FIRST_PARTY_BASE_URL=1

claude() {
  local a rc=0
  case ${1-} in remote-control|rc) rc=1 ;; esac
  for a in "$@"; do
    case $a in
      --) break ;;
      --remote-control|--remote-control=*) rc=1 ;;
    esac
  done
  if [ "$rc" = 1 ]; then
    env -u ANTHROPIC_BASE_URL -u _CLAUDE_CODE_ASSUME_FIRST_PARTY_BASE_URL claude "$@"
  else
    command claude "$@"
  fi
}
