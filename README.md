# claude-strapon

One command that measures the Claude Code stack on this machine, shows what is there,
and fixes what you pick: gum, Node 22, Claude Code, rtk, the caveman proxy route,
codegraph, the second account, effort-router, plugins, the statusline, agents + skills
and the default model.

```bash
./claude-strapon              # scan, board, pick, preview, confirm, apply, rescan
./claude-strapon --dry-run    # board + preview only; changes nothing
./claude-strapon --yes --only plugins,model    # unattended, limited
./claude-strapon --redo codegraph              # offer it even though it reads ok
```

## The board
```
  ●  wired and verified       ◐  installed, not wired     ○  missing
  ✗  wired but broken         !  needs you (a login or a by-hand step; never in the pick-list)
```
"Verified" means a live check, not a config entry: `claude mcp get` says Connected,
something listens on :8787, `rtk init --show` reads `[ok]`. The rescan after apply is
the proof. A component that still does not read ● makes the run exit 1.

## Safety
- Read-only until you confirm the preview. Without a terminal and without `--yes`, it
  stops after the preview.
- Every `settings.json` / `.claude.json` write is backed up (`*.bak-<timestamp>`),
  written through a tmp file, re-parsed, then moved into place. Claude Code silently
  drops a settings file it cannot parse, which would switch off every hook at once.
- Symlinked account-2 files are written through to their target, never replaced.
- It never writes the proxy route while nothing listens on :8787. A route with nothing
  behind it fails every session in both accounts; the scan flags that case ✗.
- The preview warns when claude sessions are running: they rewrite `.claude.json` and
  can undo MCP changes.

## Layout
```
claude-strapon            entry point
lib/ui.sh               gum prompts, or numbered plain prompts without gum
lib/claude.sh           account paths, json_edit, claude_as, node22_bin
components/NN-name.sh   <name>_detect / _plan / _apply, run in NN order
files/                  what gets installed: effort-router.sh, agents/, skills/,
                        plugins.txt, plugins-remove.txt, marketplaces.txt,
                        caveman-proxy.service, router/ (config snapshots)
docs/                   background: router, runbooks, model + effort guides
tests/test-claude-strapon.sh   sandbox HOME + stubs; touches nothing real
```
Adding a component means one file in `components/` with the three functions, plus
`<name>_deps` and `<name>_optional=1` if they apply.

## Checks
```bash
shellcheck -S warning -x -s bash claude-strapon lib/*.sh components/*.sh tests/*.sh
tests/test-claude-strapon.sh
```

`docs/` predates this tool. Its `~/ops/...` paths and by-hand steps describe how the
stack was first built and are kept for background. The components are the current
procedure.
