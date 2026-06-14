#!/usr/bin/env bash
# effort-router.sh — UserPromptSubmit hook
# Classifies prompt complexity, prints a plain-text effort hint to stdout.
# UserPromptSubmit injects stdout into Claude's context on exit 0.
# Override any time with /effort <level>.
#
# Hardening: ALWAYS exits 0. Plain stdout only (no JSON — avoids hook-error bugs).
# Patterns are SINGLE-LINE strings — do NOT add backslash line-continuations inside
# a quoted regex; that injects literal trailing backslashes and silently breaks grep
# (every prompt would fall through to the default). Verified working June 2026.

# Read stdin; never fail if it's empty or the pipe is closed.
INPUT=$(cat 2>/dev/null || true)

# Extract prompt. Degrade silently if jq is missing.
if command -v jq >/dev/null 2>&1; then
  PROMPT=$(printf '%s' "$INPUT" | jq -r '.prompt // ""' 2>/dev/null || true)
else
  exit 0
fi

PROMPT_LOWER=$(printf '%s' "$PROMPT" | tr '[:upper:]' '[:lower:]' 2>/dev/null || true)
[ -z "$PROMPT_LOWER" ] && exit 0

# ── Classifier patterns (single-line each) ─────────────────────────────────
LOW_RE='\b(rename|typo|whitespace|indent|reformat|prettier|eslint fix|lint fix)\b|\b(variable name|constant name|placeholder)\b|replace [a-z_]+ with [a-z_]+|\b(what is|whats|check if|does .{0,30} exist)\b|\b(list (files|dirs|packages|modules)|show me|print out)\b|\bgit (status|log|diff|add|push|pull|fetch|stash|branch)\b|\b(ls|pwd|which|cat|head|tail|wc|du|df|uname)\b|--version\b'

XHIGH_RE='\b(architect|system design|redesign|overhaul|restructure)\b|refactor (the )?(entire|whole|all|everything)|\b(race condition|deadlock|memory leak|use.after.free|segfault|heap corruption)\b|security (vuln|audit|review|flaw|hole|exploit|issue)|auth (flow|system|bypass)|token expir|session hijack|database (schema|migration|sharding|index strategy)|performance (regression|bottleneck|profile|profiling)|why (is|does|isn.t|doesn.t|won.t|can.t) .{5,80}|\b(root cause|investigate|post.?mortem)\b|explain (how|why) .{10,80}|trade.?off|compare (approach|option|solution|architecture|strategy)|breaking change|api (contract|versioning)'

HIGH_RE='\b(implement|build)\b|create (a |the |new )|add (feature|support|endpoint|route|component|function|class|module|plugin)|fix (bug|issue|error|crash|test|failing|broken)|\bdebug\b|failing test|flaky test|write (test|spec|function|class|module|script|handler)|\b(integrate|configure|deploy)\b|set up|ci.?cd|dockerfile|\b(parse|serialize|transform|migrate|convert)\b|\b(async|concurrent|parallel|thread|queue|worker)\b|\b(hook|middleware|interceptor|wrapper|decorator)\b'

# ── Classify — first match wins. Baseline high (Opus 4.8 default) ───────────
EFFORT="high"
if printf '%s' "$PROMPT_LOWER" | grep -qiE "$LOW_RE" 2>/dev/null; then
  EFFORT="low"
elif printf '%s' "$PROMPT_LOWER" | grep -qiE "$XHIGH_RE" 2>/dev/null; then
  EFFORT="xhigh"
elif printf '%s' "$PROMPT_LOWER" | grep -qiE "$HIGH_RE" 2>/dev/null; then
  EFFORT="high"
fi

# ── Output: plain text to stdout, phrased as context, not a command ─────────
printf '[effort-router] Task complexity: %s. Match your reasoning depth to this level.\n' "$EFFORT" 2>/dev/null || true
exit 0
