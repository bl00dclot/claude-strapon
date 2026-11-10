# Handoff runbook — guardrail-touching steps (YOU run these)
> **Historical.** Predates claude-strapon; paths and steps are from the original by-hand setup. Current procedure: [components.md](components.md). Index: [README.md](README.md).

The ops agent is forbidden by its operating contract (`~/ops/CLAUDE.md`) and by the deny
rule `Edit(~/.claude/**)` from modifying `~/.claude/`. Everything below writes into
`~/.claude/` (hooks, settings.json, CLAUDE.md, MCP registration) or `~/.bashrc`, so you
run it yourself in a normal terminal. Run these in order. **Validate JSON after every
settings.json change** — Claude Code silently discards the whole file if it fails schema
validation.

Already done by the agent (no action needed):
- RTK binary installed → ~/.local/bin/rtk (v0.42.4), verified correct tool
- CodeGraph binary installed → Node 22.22.3 (now nvm default), v1.0.1
- ~/ops/effort-router.sh created, chmod +x, all tests pass
- ~/ops/merge-effort-hook.py created (NOT run)

---

## 0. Back up settings.json first (always)

```bash
cp ~/.claude/settings.json ~/ops/setup/settings.json.backup-$(date +%Y%m%d-%H%M%S)
python3 -m json.tool ~/.claude/settings.json >/dev/null && echo "✓ current settings.json valid" || echo "✗ already invalid — fix before proceeding"
```

---

## 1. RTK hook wiring

```bash
export PATH="$HOME/.local/bin:$PATH"
rtk init -g          # answer y to patch settings.json
```
In RTK v0.42.4 this registers the PreToolUse(Bash) hook as the **native binary command
`rtk hook claude`** directly in `~/.claude/settings.json` (NOT a `~/.claude/hooks/*.sh`
script — that path is obsolete), creates `~/.claude/RTK.md`, and adds `@RTK.md` to
`~/.claude/CLAUDE.md`.

Verify (the file-test for rtk-rewrite.sh is OBSOLETE — it always reports MISSING on v0.42.4):
```bash
rtk init --show     # the real check — every line should read [ok]
grep -q '"rtk hook claude"' ~/.claude/settings.json && echo "rtk hook: present" || echo "rtk hook: MISSING"
python3 -m json.tool ~/.claude/settings.json >/dev/null && echo "✓ valid" || echo "✗ INVALID — restore backup"
```

Caveats to revisit after a few sessions:
- Issue #1821: the RTK hook can vanish after the first command. Re-check with `rtk init --show`.
- Issue #582: RTK can RAISE total cost on file-heavy work (input savings outweighed by
  Claude re-reading compressed-away detail). `rtk gain` only measures the input side.
- RTK intercepts the **Bash** tool only — native Read/Grep/Glob bypass it.
- ⚠ This box's whole design is "read live, uncompressed output." RTK compressing tool
  output works against that. Consider `rtk init -g --hook-only` or leaving RTK off for
  the ops agent specifically.

---

## 2. CodeGraph MCP registration

Requires Node 22 active (the agent set it default, but confirm):
```bash
export NVM_DIR="$HOME/.config/nvm"; . "$NVM_DIR/nvm.sh"; nvm use 22
node --version            # expect v22.x
codegraph install         # registers MCP into Claude Code; answer yes to auto-allow
grep -n -i "code.\?graph" ~/.claude/CLAUDE.md 2>/dev/null || echo "not in CLAUDE.md (ok if MCP-registered)"
```

⚠ CRITICAL nvm trap (CONFIRMED on this box 2026-06-15): `codegraph install` writes
`"command": "codegraph"` (bare) into ~/.claude.json. Because BOTH codegraph and node are
nvm-managed, Claude Code can't find them when it spawns the MCP server (it does not load
nvm) → `/mcp` shows "Failed to reconnect to codegraph: ENOENT". Fix: re-register with
ABSOLUTE node + shim paths (zero PATH dependence), then RESTART Claude Code:

```bash
claude mcp remove codegraph -s user
claude mcp add codegraph -s user -- \
  $HOME/.config/nvm/versions/node/v22.22.3/bin/node \
  $HOME/.config/nvm/versions/node/v22.22.3/lib/node_modules/@colbymchenry/codegraph/npm-shim.js \
  serve --mcp
```
If you change Node versions, re-find the paths: `ls ~/.config/nvm/versions/node/*/bin/codegraph`

THE ONLY real confirmation (config presence ≠ working — issue #172):
```
Start a Claude Code session in a repo, run /mcp, confirm codegraph shows GREEN/connected.
Red or absent = not actually working, regardless of config files.
```

Per-repo (you run inside each project, one time):
```bash
cd <repo> && codegraph init
```
Note: v1.0.1 `status`/`--help` has no `Backend: native/wasm` line — the native/wasm check
from the original spec doesn't apply to this version.

---

## 3. Model default (optional)

```bash
# Only if you want a persistent per-shell default; /model switches per session anyway.
echo 'export ANTHROPIC_MODEL="claude-opus-4-8"' >> ~/.bashrc && source ~/.bashrc
```
See ~/ops/setup/model-guide.md for the which-model-when table. Do NOT install CCR.

---

## 4. Caveman

The official installer itself wires SessionStart + UserPromptSubmit hooks into
settings.json (and validates the schema):
```bash
curl -fsSL https://raw.githubusercontent.com/JuliusBrussee/caveman/main/install.sh | bash
# when prompted, select Claude Code; skip other agents
```
Verify:
```bash
grep -n "caveman" ~/.claude/settings.json | head
python3 -m json.tool ~/.claude/settings.json >/dev/null && echo "✓ valid" || echo "✗ INVALID — restore backup"
grep -A3 "caveman" ~/.claude/settings.json | head -20   # shows /caveman vs always-on mode
```
⚠ Same caution as RTK: Caveman makes output terse (~65% reduction). That fights the ops
agent's evidence-first / detailed-output design. Consider activating it only via `/caveman`
on demand rather than always-on.

---

## 5. effort-router hook registration

The script and merge tool are already created. Just run the merge (it appends an entry
without clobbering Caveman's UserPromptSubmit — multiple hooks on one event coexist):
```bash
python3 ~/ops/merge-effort-hook.py
python3 -m json.tool ~/.claude/settings.json >/dev/null && echo "✓ valid" || echo "✗ INVALID — restore backup"
grep -n "effort-router" ~/.claude/settings.json && echo "effort-router hook: present"
```

---

## 6. Final verification pass

```bash
echo "=== AUTH ==="; printf 'API_KEY:%s AUTH_TOKEN:%s\n' "${ANTHROPIC_API_KEY:-unset}" "${ANTHROPIC_AUTH_TOKEN:-unset}"
echo "=== HOOKS ==="; python3 -c "import json,os;s=json.load(open(os.path.expanduser('~/.claude/settings.json')));[print(e,[h.get('command','') for x in v for h in x.get('hooks',[])]) for e,v in s.get('hooks',{}).items()]"
echo "=== VALID JSON ==="; python3 -m json.tool ~/.claude/settings.json >/dev/null && echo VALID || echo INVALID
echo "=== effort-router live ==="; echo '{"prompt":"why does the JWT expire before the session cookie"}' | ~/ops/effort-router.sh
echo ">>> MANUAL: open a session, run /mcp, confirm codegraph GREEN <<<"
echo ">>> MANUAL: run /status, confirm subscription (not API) <<<"
```

Expected hooks after all steps: PreToolUse=[rtk], UserPromptSubmit=[caveman, effort-router],
SessionStart=[caveman].

## Rollback
Any step corrupts settings.json → `cp ~/ops/setup/settings.json.backup-* ~/.claude/settings.json`
Remove effort-router → delete its UserPromptSubmit entry. Remove RTK → `rtk init --uninstall` (or hand-remove). Remove CodeGraph → `codegraph uninstall`.
