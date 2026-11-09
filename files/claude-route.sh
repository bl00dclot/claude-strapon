# shellcheck shell=bash
# Sourced from ~/.bashrc by the claude-strapon caveman component (and from root's
# .bashrc on server-a).
#
# Routes Claude Code through caveman-proxy from the shell environment instead of from
# settings.json env. Remote Control refuses any ANTHROPIC_BASE_URL that is not
# api.anthropic.com (_CLAUDE_CODE_ASSUME_FIRST_PARTY_BASE_URL does not cover it), and
# settings.json env beats the shell, so a settings route left no way to start it.
#
# The route is exported, so everything started from this shell inherits it: scripts,
# `sh -c` launchers, `claude -p` workers, the claude2 alias. The functions below strip
# it only for launches that start Remote Control, the same rule `caveman wrap claude`
# applies: `remote-control`/`rc` in first position, or `--remote-control[=name]` before
# `--`. Bypass the proxy with `unset ANTHROPIC_BASE_URL`.
export ANTHROPIC_BASE_URL=http://127.0.0.1:8787/w/claude _CLAUDE_CODE_ASSUME_FIRST_PARTY_BASE_URL=1

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
# Account wrapper scripts (the servers' claude1/claude2) exec claude and would pass the
# route straight through. An alias of the same name (claude2 on devbox) is not a
# file, expands to `claude`, and needs nothing here.
if type -P claude1 >/dev/null; then function claude1 { _claude_launch claude1 "$@"; }; fi
if type -P claude2 >/dev/null; then function claude2 { _claude_launch claude2 "$@"; }; fi
