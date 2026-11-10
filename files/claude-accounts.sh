# shellcheck shell=bash
# Sourced from ~/.bashrc by the claude-strapon accounts component (and by claude-route.sh).
#
# One launcher per Claude account, all sharing ~/.claude's settings through symlinks:
#   claude / claude1   account 1 (~/.claude)
#   claudeN            CLAUDE_CONFIG_DIR=~/.claude-accountN, for every such dir (N >= 2)
# Accounts are found when the shell starts, so one added later needs only a new shell.
#
# Every launcher drops the caveman route for launches that start Remote Control, which
# refuses any ANTHROPIC_BASE_URL that is not api.anthropic.com: `remote-control`/`rc` in
# first position, or `--remote-control[=name]` before `--` (the rule `caveman wrap claude`
# applies). Without a route in the environment that is a no-op.
#
# A name that is already an alias is left to the alias (claude2 on the dev box expands to
# `claude`, so it gets the rule anyway). A name that is an executable on PATH (the
# servers' claude1/claude2 wrappers) is wrapped, not replaced.

# _claude_launch CMD ARGS... — run CMD, direct to api.anthropic.com if ARGS start Remote Control.
_claude_launch() {
  local cmd=$1 a rc=0; shift
  case ${1-} in remote-control|rc) rc=1 ;; esac
  for a in "$@"; do
    case $a in
      --) break ;;
      --remote-control|--remote-control=*) rc=1 ;;
    esac
  done
  if [ "$rc" = 1 ]; then
    env -u ANTHROPIC_BASE_URL -u _CLAUDE_CODE_ASSUME_FIRST_PARTY_BASE_URL "$cmd" "$@"
  else
    command "$cmd" "$@"
  fi
}

function claude { _claude_launch claude "$@"; }

# _claude_account NAME [DIR] — define launcher NAME for the account in DIR (none = account 1).
_claude_account() {
  alias "$1" >/dev/null 2>&1 && return 0
  if type -P "$1" >/dev/null; then
    eval "function $1 { _claude_launch $1 \"\$@\"; }"
  elif [ -n "${2-}" ]; then
    eval "function $1 { CLAUDE_CONFIG_DIR=$(printf %q "$2") _claude_launch claude \"\$@\"; }"
  else
    eval "function $1 { ( unset CLAUDE_CONFIG_DIR; _claude_launch claude \"\$@\" ); }"
  fi
}

_claude_account claude1
for _claude_d in "$HOME"/.claude-account[1-9]*; do
  _claude_n=${_claude_d##*/.claude-account}
  case $_claude_n in *[!0-9]*) continue ;; esac
  [ "$_claude_n" -ge 2 ] && [ -d "$_claude_d" ] && _claude_account "claude$_claude_n" "$_claude_d"
done
unset _claude_d _claude_n
