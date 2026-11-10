# Effort routing map

Auto-set by `files/effort-router.sh` (this repo) on every prompt, through the
UserPromptSubmit hook that the `effort-router` component wires. Override any time with: /effort <level>
Set session-wide with env var: CLAUDE_CODE_EFFORT_LEVEL=xhigh claude

| Level  | Thinking budget             | Auto-triggered by (examples)                                      |
|--------|-----------------------------|-------------------------------------------------------------------|
| low    | Minimal / recall only       | rename, git status, ls, cat, typo, what is, check if, version     |
| high   | Extended (Opus 4.8 default) | implement, fix bug, write test, add feature, configure            |
| xhigh  | Deep / architecture         | architect, race condition, security audit, why does X, root cause |

Notes:
- medium reached only if no pattern matches (rare on real prompts)
- max never auto-assigned; use /effort max explicitly for extreme one-offs
- Opus 4.8 default is already high; low/xhigh are the meaningful deviations
- Patterns live in `files/effort-router.sh` — tune any time, no restart needed
- The model names in the table date from mid-2026; the levels and triggers are what matter
- The hook injects *context*, not a hard lock — Claude usually follows it; /effort overrides
