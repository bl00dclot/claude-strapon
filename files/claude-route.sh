# shellcheck shell=bash
# Sourced from ~/.bashrc by the claude-strapon caveman component.
#
# Routes Claude Code through caveman-proxy from the shell environment instead of from
# settings.json env. Remote Control refuses any ANTHROPIC_BASE_URL that is not
# api.anthropic.com (_CLAUDE_CODE_ASSUME_FIRST_PARTY_BASE_URL does not cover it), and
# settings.json env beats the shell, so a settings route left no way to start it.
#
# The route is exported, so everything started from this shell inherits it: scripts,
# `sh -c` launchers, `claude -p` workers. The launchers in claude-accounts.sh (sourced
# below, so a .bashrc with only this line still gets them) strip it for launches that
# start Remote Control. Bypass the proxy with `unset ANTHROPIC_BASE_URL`.
export ANTHROPIC_BASE_URL=http://127.0.0.1:8787/w/claude _CLAUDE_CODE_ASSUME_FIRST_PARTY_BASE_URL=1
# shellcheck source=files/claude-accounts.sh
. "$(dirname "${BASH_SOURCE[0]}")/claude-accounts.sh"
