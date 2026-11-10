#!/usr/bin/env bash
# The launcher functions in files/claude-accounts.sh and files/claude-route.sh, each case
# in a clean `bash --norc` with a fake claude that prints what it was started with.
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
fail() { echo "FAIL: $*"; exit 1; }

F=$T/fn; mkdir -p "$F"
printf '#!/bin/sh\necho "url=${ANTHROPIC_BASE_URL-} fp=${_CLAUDE_CODE_ASSUME_FIRST_PARTY_BASE_URL-} dir=${CLAUDE_CONFIG_DIR-} args=$*"\n' > "$F/claude"
sed 's/^echo "/echo "claude2 /' "$F/claude" > "$F/claude2"   # a server-style account wrapper on PATH
chmod +x "$F/claude" "$F/claude2"

H=$T/home
mkdir -p "$H/.claude-account2" "$H/.claude-account3" "$H/.claude-account02" "$H/.claude-accountx" "$H/.claude-account1"
H2="$T/h o'me"; mkdir -p "$H2/.claude-account3"
R=http://127.0.0.1:8787/w/claude

# want FILE HOME CMD EXPECTED — parent env carries a route, as a shell opened from a routed session would.
want() {
  local got
  got=$(env -i HOME="$2" PATH="$F:/usr/bin:/bin" ANTHROPIC_BASE_URL=inherited _CLAUDE_CODE_ASSUME_FIRST_PARTY_BASE_URL=1 \
    bash --norc -c ". '$ROOT/files/$1'; $3" 2>&1) || true
  [ "$got" = "$4" ] || fail "[$1] '$3' gave '$got', want '$4'"
}
route() { want claude-route.sh "$H" "$1" "$2"; }

# Route file: claude and every launcher routed, Remote Control direct.
route 'claude'                           "url=$R fp=1 dir= args="
route 'claude -p remote-control'         "url=$R fp=1 dir= args=-p remote-control"
route 'claude -p x -- --remote-control'  "url=$R fp=1 dir= args=-p x -- --remote-control"
route 'claude --remote-control-session-name-prefix box' "url=$R fp=1 dir= args=--remote-control-session-name-prefix box"
route 'claude remote-control --name a'   "url= fp= dir= args=remote-control --name a"
route 'claude --remote-control phone'    "url= fp= dir= args=--remote-control phone"
route 'claude --remote-control=phone'    "url= fp= dir= args=--remote-control=phone"
route 'CLAUDE_CONFIG_DIR=/a2 claude rc'  "url= fp= dir=/a2 args=rc"
route 'CLAUDE_CONFIG_DIR=/a2 claude'     "url=$R fp=1 dir=/a2 args="
route 'sh -c "claude -p x"'              "url=$R fp=1 dir= args=-p x"
route 'unset ANTHROPIC_BASE_URL; claude' "url= fp=1 dir= args="
route 'claude2'                          "claude2 url=$R fp=1 dir= args="
route 'claude2 rc'                       "claude2 url= fp= dir= args=rc"
route 'claude1'                          "url=$R fp=1 dir= args="
route 'CLAUDE_CONFIG_DIR=/x claude1 rc'  "url= fp= dir= args=rc"
route 'claude3'                          "url=$R fp=1 dir=$H/.claude-account3 args="
route 'claude3 rc'                       "url= fp= dir=$H/.claude-account3 args=rc"
route 'claude3 -p x --remote-control=p'  "url= fp= dir=$H/.claude-account3 args=-p x --remote-control=p"
route 'type -t claude02 claudex claude0 || echo none' "none"
echo "ok 1  route: claude, claude1, claude2 wrapper, claude3 routed; Remote Control direct"

# Accounts file alone (caveman not set up): launchers work, nothing forces a route.
want claude-accounts.sh "$H" 'claude3'    "url=inherited fp=1 dir=$H/.claude-account3 args="
want claude-accounts.sh "$H" 'claude3 rc' "url= fp= dir=$H/.claude-account3 args=rc"
want claude-accounts.sh "$H" 'claude'     "url=inherited fp=1 dir= args="
echo "ok 2  accounts file alone: launchers without a forced route"

# A HOME with a space and a quote.
want claude-route.sh "$H2" 'claude3' "url=$R fp=1 dir=$H2/.claude-account3 args="
echo "ok 3  quoted HOME"

# An alias of the same name (claude2 on the dev box) must survive sourcing.
got=$(env -i HOME="$H" PATH="/usr/bin:/bin" bash --norc -ic "alias claude2='echo aliased'
. '$ROOT/files/claude-route.sh'
claude2" 2>/dev/null)
[ "$got" = aliased ] || fail "alias claude2 broken by sourcing: '$got'"
echo "ok 4  alias wins over the launcher"
echo PASS
