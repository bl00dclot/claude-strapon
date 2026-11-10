# claude-strapon

One command that measures the Claude Code stack on this machine, shows what is there,
and fixes what you pick: Node 22, Claude Code, rtk, the caveman proxy route, codegraph,
any number of accounts (claude2, claude3, …), effort-router, plugins, the statusline,
agents + skills and the default model.

```bash
./claude-strapon                     # scan, board, pick, preview, confirm, apply, rescan
./claude-strapon --dry-run           # board + preview only; changes nothing
./claude-strapon --yes --only plugins,model    # unattended, limited
./claude-strapon --redo codegraph              # offer it even though it reads ok
./claude-strapon --add-accounts 2              # create claude3 + claude4 (next free numbers)
```

The first run builds the binary (`cargo build --release`, about a minute); later runs
rebuild only when the source changed. Needs `cargo` (rustup).

## The board
```
  ●  wired and verified       ◐  installed, not wired     ○  missing
  ✗  wired but broken         !  needs you (a login or a by-hand step; never in the pick-list)
```
"Verified" means a live check, not a config entry: `claude mcp get` says Connected,
your own process listens on :8787, `rtk init --show` reads `[ok]`. The rescan after
apply is the proof. A picked component that still does not read ● makes the run exit 1.
Checks run in parallel; the slowest single one (`claude mcp get codegraph`, a few
seconds) sets the pace.

## Optional components
rtk, caveman and codegraph (and the statusline badge) start unticked. Tick them in the
pick-list, or name them: `--only caveman`.

## Accounts
Account 1 is `~/.claude`. Every `~/.claude-accountN` (N ≥ 2) is another subscription that
shares account 1's settings, CLAUDE.md, RTK.md, agents, skills and plugins through
symlinks; only its `.claude.json` (login + MCP servers) is its own, and MCP servers it lacks
are copied in from account 1. The board shows one sub-row per account.

Add accounts from the pick-list (`accounts` is always offered on a terminal; it asks how
many) or with `--add-accounts N`. Each new account then needs one login by hand:
open a new shell and run `claudeN`.

Launchers come from `files/claude-accounts.sh`, sourced from `~/.bashrc`: `claude`,
`claude1` and a `claudeN` for every account dir found when the shell starts. Every
launcher sends Remote Control launches direct to api.anthropic.com. An existing alias of
the same name is left alone; an executable of the same name on PATH is wrapped.

## Safety
- Read-only until you confirm the preview. Without a terminal and without `--yes`, it
  stops after the preview.
- Every `settings.json` / `.claude.json` write is backed up (`*.bak-<UTC timestamp>`,
  `-2`, `-3`… if several land in one second), written through a tmp file, re-parsed, then
  moved into place. A file that does not parse is never overwritten: Claude Code silently
  drops a settings file it cannot parse, which would switch off every hook at once.
- Symlinked account files are written through to their target, never replaced.
- `~/.bashrc` is only ever appended to.
- It never writes the proxy route while nothing listens on :8787, or while something
  other than you or root does (it would receive every session's credentials). The scan
  flags both cases ✗.
- The preview warns when claude sessions are running (they rewrite `.claude.json` and can
  undo MCP changes) and when this checkout is a git worktree (hooks and `.bashrc` lines
  would point into it).

## Layout
```
claude-strapon            entry point: cargo run --release
src/main.rs               flags, scan -> board -> pick -> preview -> apply -> rescan
src/ctx.rs                paths, account discovery, subprocess helpers
src/json.rs               the safe JSON edit
src/ui.rs                 colour, board, ratatui pick-list, plain prompts
src/comp/<name>.rs        one component: detect (read-only, parallel) + apply
files/                    what gets installed: claude-accounts.sh, claude-route.sh,
                          effort-router.sh, agents/, skills/, plugins.txt,
                          plugins-remove.txt, marketplaces.txt, caveman-proxy.service,
                          router/ (config snapshots)
docs/                     background: router, runbooks, model + effort guides
tests/cli.rs              the binary against a sandbox HOME with stub tools
tests/shell.sh            the launchers, in clean bash shells
```
Adding a component means one file in `src/comp/` with a `C: Component` (name, deps,
optional, detect, apply) and one line in `all()`.

## Checks
```bash
cargo test
cargo clippy --all-targets -- -D warnings
shellcheck -S warning -x -s bash claude-strapon files/claude-accounts.sh files/claude-route.sh tests/shell.sh
```

`docs/` predates this tool. Its `~/ops/...` paths and by-hand steps describe how the
stack was first built and are kept for background. The components are the current
procedure.
