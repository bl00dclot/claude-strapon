# Claude Code optimization stack — status

Generated: 2026-06-14
Auth model: Claude Max subscription (OAuth) — verified: NO ANTHROPIC_API_KEY /
ANTHROPIC_AUTH_TOKEN / ANTHROPIC_BASE_URL in env or shell configs.

Setup was run in "stage it" mode on the locked-down linux-ops-agent box: the operating
contract (`~/ops/CLAUDE.md`) and deny rule `Edit(~/.claude/**)` forbid the agent from
modifying its own guardrails. So everything that writes into `~/.claude/` was deferred to
`handoff-runbook.md` for the user to run. Status legend: ✅ done by agent | ⏳ pending (runbook).

## RTK (Rust Token Killer)
- ✅ Binary: v0.42.4 at ~/.local/bin/rtk (checksum verified by installer)
- ✅ Correct tool confirmed: `rtk gain` → "No tracking data yet" (not an error)
- ⏳ PreToolUse hook: NOT wired (runbook §1 — `rtk init -g`)
- Caveat: Bash tool only (native Read/Grep/Glob bypass it); can raise cost on file-heavy
  tasks (#582); hook can vanish after 1st command (#1821). Compression fights this box's
  evidence-first design — consider `--hook-only` or off for ops.

## CodeGraph
- ✅ Binary: v1.0.1, installed under Node 22.22.3 (nvm default set to 22)
- ⚠ Spec wanted Node 22.x; box had 24.13.1 → switched to 22 via nvm per user choice
- ⚠ No `Backend: native/wasm` line exists in v1.0.1 status/help — that check is N/A here
- ⚠ `codegraph` only on PATH when nvm has 22 active
- ⏳ MCP registration: NOT done (runbook §2 — `codegraph install`)
- ⏳ MANUAL confirm: /mcp shows codegraph GREEN inside a session (config ≠ connection, #172)
- Per-repo: `codegraph init` (user runs inside each repo)

## Model switching (native — no CCR)
- Default: claude-opus-4-8 (platform default, high effort)
- Per-session: /model claude-haiku-4-5-20251001 (fast) | /model claude-sonnet-4-6 (long ctx)
- ⏳ Optional ~/.bashrc `ANTHROPIC_MODEL` default: NOT set (runbook §3)
- CCR excluded: it sets ANTHROPIC_AUTH_TOKEN → overrides subscription OAuth
- Reference: ~/ops/setup/model-guide.md

## Caveman
- ⏳ NOT installed (runbook §4 — official installer wires its own hooks)
- ⚠ ~65% output reduction fights evidence-first design — consider /caveman on-demand only

## Effort router
- ✅ Script: ~/ops/effort-router.sh (chmod +x; tests pass: low/high/xhigh/low; exits 0 on
  empty/bad-json/closed-pipe — can never block a prompt)
- ✅ Merge tool: ~/ops/merge-effort-hook.py (created, NOT run)
- ⏳ UserPromptSubmit registration: NOT done (runbook §5 — run merge-effort-hook.py)
- Plain stdout (not JSON) — avoids hook-error bugs #17550/#13912/#44943
- Reference: ~/ops/setup/effort-map.md

## Known constraints
- ANTHROPIC_API_KEY / ANTHROPIC_AUTH_TOKEN in env silently overrides subscription — /status to check
- Caveman + effort-router both on UserPromptSubmit coexist fine (parallel, each appends context)
- Always validate settings.json with `python3 -m json.tool` after any edit (silent Zod discard)
- effort-router guidance is context, not a hard lock; /effort <level> overrides

## Remaining work
All in ~/ops/setup/handoff-runbook.md — run §0 (backup) through §6 (verify) in a normal
terminal. Decide per tool whether always-on compression/terseness belongs on THIS box,
given its evidence-first operating contract.
