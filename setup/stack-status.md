# Claude Code optimization stack — status

Last verified: 2026-06-15 (live, in-session). Supersedes the 2026-06-14 "staged" status.
Auth model: Claude Max subscription (OAuth) — verified live: ANTHROPIC_API_KEY,
ANTHROPIC_AUTH_TOKEN, ANTHROPIC_BASE_URL all unset.

The 2026-06-14 pass staged everything that writes into `~/.claude/` into
`handoff-runbook.md` because the ops contract (`~/ops/CLAUDE.md`) + deny rule
`Edit(~/.claude/**)` forbid the agent from touching its own guardrails. The user has
since run the runbook: every step (§0–§6) is now live. This file records the verified
post-activation state. See `activation-guide.md` for how each piece turns on.

Status legend: ✅ live & verified | ⚠ caveat | ◻ optional, not done by choice.

## RTK (Rust Token Killer)
- ✅ Binary: v0.42.4 at ~/.local/bin/rtk
- ✅ PreToolUse(Bash) hook: `rtk hook claude` present in ~/.claude/settings.json
- ✅ `rtk init --show`: hook, RTK.md, @RTK.md (~/.claude/CLAUDE.md), settings.json all [ok]
- ✅ Working: `rtk gain` → 244 cmds, 4.4M tokens saved (98.5% on the input side)
- No per-repo init required — global hook covers every repo. Optional per-repo extras:
  `./.rtk/filters.toml` (custom filters) and a local ./CLAUDE.md block (cosmetic).
- ⚠ Bash tool only (native Read/Grep/Glob bypass it); can RAISE cost on file-heavy
  tasks (#582, gain measures input only); hook can vanish after 1st cmd (#1821).
- ⚠ Always-on compression fights this box's evidence-first design — see "Open decision".

## CodeGraph
- ✅ Binary: v1.0.1, installed under Node 22.22.3 (nvm default = 22)
- ✅ MCP registration: ~/.claude.json has codegraph with ABSOLUTE node + npm-shim paths
  (zero PATH dependence) — fixes the nvm ENOENT trap from the bare `command:"codegraph"`.
- ✅ Verified connected (claude mcp get → ✔; initialize → serverInfo 1.0.1).
- ⚠ Green but exposes NO tools until `codegraph init` creates `.codegraph/` in a repo.
- ◻ Per-repo `codegraph init`: none run yet. ~/ops is NOT indexed → MCP inactive here.
- ⚠ `codegraph` on PATH only when nvm has 22 active; running sessions show stale ENOENT
  until /mcp reconnect or restart.

## Model switching (native — no CCR)
- ✅ Default pinned in ~/.claude/settings.json: model = opus[1m] (high effort default)
- Per-session: /model claude-haiku-4-5-20251001 (fast) | /model claude-sonnet-4-6 (long ctx)
- ◻ ~/.bashrc ANTHROPIC_MODEL export: not set — moot, model already pinned in settings.
- CCR excluded: it sets ANTHROPIC_AUTH_TOKEN → overrides subscription OAuth.
- Reference: ~/ops/setup/model-guide.md

## Caveman
- ✅ Installed as a plugin (~/.claude/plugins/cache/caveman); SessionStart +
  UserPromptSubmit hooks fire from the plugin, not from settings.json's hooks block.
- ✅ Active this session: "CAVEMAN MODE ACTIVE — level: full".
- ◻ Statusline badge: NOT configured (no `statusLine` key in settings.json). Cosmetic.
- ⚠ ~65% output reduction fights evidence-first design — see "Open decision".

## Effort router
- ✅ Script: ~/ops/effort-router.sh (chmod +x; exits 0 on empty/bad-json/closed-pipe)
- ✅ Registered on UserPromptSubmit in ~/.claude/settings.json (via merge-effort-hook.py)
- ✅ Live: "rename" → low, "fix bug" → high (verified in-session)
- Plain stdout (not JSON) — avoids hook-error bugs #17550/#13912/#44943
- Reference: ~/ops/setup/effort-map.md

## Known constraints
- ANTHROPIC_API_KEY / ANTHROPIC_AUTH_TOKEN in env silently overrides subscription — /status to check
- Caveman + effort-router both on UserPromptSubmit coexist fine (parallel, each appends context)
- Always validate settings.json with `python3 -m json.tool` after any edit (silent Zod discard)
- effort-router guidance is context, not a hard lock; /effort <level> overrides

## Open decision (unresolved)
RTK compression and Caveman terseness are both always-on. Both work against this box's
evidence-first operating contract ("read live, uncompressed output"). Not yet decided
whether either belongs always-on here, or should be `--hook-only` / on-demand `/caveman`.

## Remaining work
- ◻ Configure Caveman statusline badge (cosmetic).
- ◻ Run `codegraph init` per repo where graph queries are wanted (none yet).
- ◻ Resolve the always-on compression/terseness decision above.
Everything in handoff-runbook.md §0–§6 is done.
