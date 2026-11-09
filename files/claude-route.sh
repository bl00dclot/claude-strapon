# shellcheck shell=bash
# Sourced from ~/.bashrc by the claude-strapon caveman component.
#
# Routes claude (and the claude2 alias, which expands to `CLAUDE_CONFIG_DIR=... claude`)
# through caveman-proxy per launch. The route used to live in settings.json env, but
# Remote Control refuses any ANTHROPIC_BASE_URL that is not api.anthropic.com
# (_CLAUDE_CODE_ASSUME_FIRST_PARTY_BASE_URL does not cover it), and settings.json env
# beats the shell, so no launch could opt out. Remote Control launches go direct, the
# same rule `caveman wrap claude` applies: the `remote-control`/`rc` subcommand in first
# position, or `--remote-control[=name]` before `--`.
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
    ANTHROPIC_BASE_URL=http://127.0.0.1:8787/w/claude _CLAUDE_CODE_ASSUME_FIRST_PARTY_BASE_URL=1 \
      command claude "$@"
  fi
}
