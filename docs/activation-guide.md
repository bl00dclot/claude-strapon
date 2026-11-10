# Claude Code optimization stack — activation guide
> **Historical.** Predates claude-strapon; paths and steps are from the original by-hand setup. Current procedure: [components.md](components.md). Index: [README.md](README.md).

How the stack turns on, what scope each piece lives at, and what (if anything) you run
per repo or per session. The stack is **already active on this box** (verified
2026-06-15); the commands below are idempotent — run them only to reproduce on a fresh
machine or to repair a vanished hook.

Companion docs: `stack-status.md` (current state), `handoff-runbook.md` (step-by-step
first install), `model-guide.md`, `effort-map.md`.

---

## Components and their scope

| Component      | Activated by                          | Scope        | Per-repo step?            |
|----------------|---------------------------------------|--------------|---------------------------|
| RTK            | PreToolUse(Bash) hook `rtk hook claude` | global       | **No**                    |
| effort-router  | UserPromptSubmit hook                  | global       | No                        |
| Caveman        | plugin SessionStart + UserPromptSubmit | global       | No                        |
| Model default  | `settings.json` `model` key            | global       | No (or `/model` per session) |
| CodeGraph      | MCP server in `~/.claude.json`         | global reg.  | **Yes — `codegraph init`** |

Only **CodeGraph** needs a per-repo step. Everything else is global and repo-agnostic.

---

## RTK — how it works

RTK ("Rust Token Killer") is one binary (`~/.local/bin/rtk`) plus one hook. Two layers:

1. **Interception (hook).** The PreToolUse(Bash) hook `rtk hook claude` fires before
   every Bash tool call and rewrites recognized commands to prefix `rtk`
   (`git status` → `rtk git status`). Transparent, zero token overhead, never typed by
   hand.
2. **Filtering (binary).** `rtk <cmd>` runs the *real* command, then compresses its
   stdout before it returns to the model:
   - command has a dedicated filter → strips ANSI/noise, groups by file, shows
     failures-only, dedups logs → 60–99% fewer tokens;
   - no filter → output passes through unchanged (always safe).

It is output compression, not interception of the work — the real command always runs in
full.

**Does RTK need a per-repo init? No.** The hook lives in `~/.claude/settings.json` and
applies to every repository. Nothing to run per project. (Contrast: CodeGraph *does*
need `codegraph init` per repo.)

Two *optional* per-repo extras, neither required to function:
- `./.rtk/filters.toml` at a repo root — project-local custom filters for your own
  tools. `setup/.rtk/filters.toml` is a commented template stub and is **inert** (RTK
  reads repo-root `./.rtk/`, not `setup/.rtk/`).
- A local `./CLAUDE.md` RTK doc block. `rtk init --show` reporting
  `[--] Local (./CLAUDE.md): rtk not configured` is cosmetic — the global `@RTK.md`
  reference already documents RTK for every session.

**Limits to keep in mind:**
- RTK intercepts the **Bash tool only**. Native Read / Grep / Glob bypass it.
- Issue #582: on file-heavy work RTK can *raise* total cost (model re-reads detail that
  was compressed away). `rtk gain` measures only the input side, not that rebound.
- Issue #1821: the hook can vanish after the first command — re-check with
  `rtk init --show`.

---

## Activate the stack

### Global (once per machine)

```bash
export PATH="$HOME/.local/bin:$PATH"
rtk init -g --auto-patch                  # RTK: hook + RTK.md + @RTK.md + settings.json
python3 ~/ops/merge-effort-hook.py        # effort-router -> UserPromptSubmit hook
# Caveman is installed as a plugin; only if reinstalling from scratch:
#   curl -fsSL https://raw.githubusercontent.com/JuliusBrussee/caveman/main/install.sh | bash
# CodeGraph MCP is registered with ABSOLUTE node+shim paths (handoff-runbook.md §2)
python3 -m json.tool ~/.claude/settings.json >/dev/null && echo VALID || echo INVALID
```

Always validate `settings.json` after a write — Claude Code silently discards the whole
file if it fails schema validation.

### Per-repo (CodeGraph only)

```bash
cd <repo> && codegraph init               # creates .codegraph/ — REQUIRED for MCP tools
# RTK, effort-router, caveman need nothing here.
```

Without `.codegraph/` in a repo the MCP server shows green but exposes zero tools.

### Per-session (inside Claude Code)

```text
/model claude-haiku-4-5-20251001   # fast tier for trivial edits
/model claude-sonnet-4-6           # long-context work
/effort xhigh                      # deep one-off (router auto-sets level otherwise)
/caveman lite|full|ultra           # terseness;  "stop caveman" turns it off
```

---

## Verify it is live

```bash
rtk init --show                                       # every line should read [ok]
rtk gain                                              # token savings accruing
echo '{"prompt":"fix bug"}' | ~/ops/effort-router.sh  # -> Task complexity: high
printf 'API_KEY:%s AUTH_TOKEN:%s\n' "${ANTHROPIC_API_KEY:-unset}" "${ANTHROPIC_AUTH_TOKEN:-unset}"
# expect both unset -> subscription OAuth intact (an API key silently overrides it)
# In a session: /mcp -> codegraph GREEN (tools appear only after codegraph init in that repo)
```

---

## Turn a piece off

RTK compression and Caveman terseness both fight this box's evidence-first operating
contract, and both are currently always-on. To dial back:

```bash
rtk init -g --uninstall            # remove RTK entirely
rtk init -g --hook-only            # keep RTK hook, drop the doc injection
# Caveman:       "stop caveman" per session, or remove its plugin hooks
# effort-router: delete its UserPromptSubmit entry from settings.json
```

## Rollback

A bad `settings.json` write → restore the latest backup:

```bash
cp ~/ops/setup/settings.json.backup-* ~/.claude/settings.json
```
