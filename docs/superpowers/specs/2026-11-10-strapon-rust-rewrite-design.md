# claude-strapon in Rust: N accounts, optional caveman/rtk/codegraph

Date: 2026-11-10 · Status: draft for owner review

## Why

claude-strapon is the one command that measures and fixes the Claude Code stack on a
machine. Today it is ~730 lines of bash (`claude-strapon`, `lib/`, 12 `components/`)
with one sandbox test. Three owner asks:

1. **Any number of accounts.** Beyond claude1/claude2, the owner wants to attach
   claude3, claude4, … as many as they like during setup. All accounts share one unified
   global config (owner: "unified global settings"). Divergence between accounts is a bug.
2. **caveman proxy, rtk and codegraph are optional features**: they start unticked on
   the board (owner ruling). Picking them is opt-in, same as statusline today.
3. **Rewrite in Rust with ratatui** (owner ruling, after web research: Rust/Go/Zig startup
   differences are milliseconds; Rust chosen for ratatui, single static binary, cargo
   1.99 already on the dev box).

Measured baseline (dev box, 2026-11-10): `./claude-strapon --dry-run </dev/null` takes
**11.5 s** wall, 3.4 s user. Nearly all of it is serial subprocesses: `claude mcp get`
(health-checks the server) per account per component, `claude plugin list`,
`rtk init --show`. The speed win comes from design, not the language: run detectors
concurrently and read JSON files directly where the CLI is not the only source of truth.

## Goals

- Same flow and same guarantees as the bash tool: scan → board → pick → preview →
  confirm → apply → rescan; the rescan is the proof; exit 1 if a picked component does
  not read ok.
- Same CLI surface: `--dry-run`, `--yes/-y`, `--only a,b`, `--redo a,b`, `-h`. Without a
  terminal and without `--yes` it stops after the preview.
- Same board states and marks: `●` ok, `◐` partial, `○` missing, `✗` broken, `!` needs
  you (never in the pick-list).
- N accounts (below).
- caveman, rtk, codegraph, statusline: optional, unticked by default. `--only` naming
  one ticks it (current `--only` rule).
- Dry run noticeably faster than 11.5 s: detectors run in parallel (thread per
  component, or a small pool); accounts checked in parallel inside a component.
- A ratatui interface for the board and the pick-list on a terminal; numbered plain
  prompts only if the TUI cannot start. No terminal and no `--yes`: stop after preview.

## Non-goals

- Rewriting what the tool *installs*: `files/effort-router.sh`, `files/claude-route.sh`,
  `files/caveman-proxy.service`, agents, skills, plugin lists stay as files. (effort-router
  runs on every prompt; porting it is a possible later win, not this project.)
- Per-account settings. Every account links to `~/.claude`.
- Remote servers: their own copy of `claude-route.sh` is
  managed from ops, not here.
- Logging accounts in. Login stays by hand (`claudeN`, then `/login`).
- The unmerged `origin/seat-profile` branch (bash component `96-seat-profile`). Left as is;
  porting it is a follow-up once this lands.

## Anti-goals (would make it a failure even if it "works")

- A `settings.json` / `.claude.json` write that is not backed up, not atomic, or not
  re-parsed before it lands. Claude Code silently drops an unparseable settings file,
  switching off every hook in every account at once.
- Replacing a symlink in an account dir instead of writing through it.
- Exporting the caveman route while nothing listens on :8787, or while caveman is not
  picked/installed. A route with nothing behind it fails every session in every account.
- Remote Control launches routed through the proxy (RC refuses a non-api.anthropic.com base URL).
- Touching anything for real in tests.

## Accounts

- Account 1: `CLAUDE_DIR` (default `~/.claude`) + `~/.claude.json`. Source of truth.
- Account N ≥ 2: `~/.claude-accountN`, launched as `claudeN` with
  `CLAUDE_CONFIG_DIR=~/.claude-accountN`. Links into `~/.claude`:
  `settings.json settings.local.json CLAUDE.md RTK.md agents skills plugins`.
  Its own `.claude.json` (login + MCP servers) cannot be a link; MCP servers missing
  from it are merged in from `~/.claude.json` (existing entries win, as today).
- Discovery: scan `~/.claude-account*` for `^\.claude-account([0-9]+)$`. Existing
  accounts appear without any config.
- Board: one row per extra account (`claude2`, `claude3`, …) with the same states the
  `claude2` component has today (links N/7, MCP in sync, `!` log in once, `✗` a real
  file where a link should be: move aside by hand).
- Adding: an **"add account"** entry in the pick step (interactive) and `--add-accounts N`
  (non-interactive, creates the next N free numbers). Creates the dir + links + launcher;
  board shows `! log in once: claudeN`.
- Launchers: see the shell file split below. Existing `alias claudeN=` lines are left
  alone: an alias `claude2='CLAUDE_CONFIG_DIR=… claude'` expands to the `claude`
  function, so it already gets the Remote Control rule.
- codegraph registers in every logged-in account. caveman installs its MCP into
  account 1 (as today); `accounts` copies missing MCP servers into the others.

## Shell file split (caveman optional)

Today `~/.bashrc` sources `files/claude-route.sh`, which always exports the route. With
caveman optional, launchers must work without it. Both files are static, in the repo,
and sourced from `~/.bashrc` by absolute path (no generated file, no `.bashrc` rewrite,
apply only appends a missing line):

- `files/claude-accounts.sh` (owned by `accounts`): defines `_claude_launch` (the Remote
  Control strip), `claude`, `claude1`, and a `claudeN` function for each
  `~/.claude-accountN` (N ≥ 2) found **when the shell starts**, so an account added
  later needs only a new shell. A name that is already an alias is skipped (the alias
  wins anyway); a name that is an executable on PATH is wrapped (`command claudeN`).
- `files/claude-route.sh` (owned by `caveman`): only the two route exports, then sources
  `claude-accounts.sh` itself, so a machine whose `.bashrc` only has the old route line
  keeps the Remote Control rule after a pull.
- Unticking caveman never auto-removes an existing route (no destructive surprise); the
  detector still reports `✗` if the route is set and :8787 is down, as today.

## Components (ported, same behaviour unless noted)

| Order | Name | Default | Notes |
|---|---|---|---|
| 20 | node22 | on | nvm `versions/node/v22.*`, newest by version sort |
| 30 | claude-code | on | native installer |
| 40 | rtk | **optional** | was pre-ticked |
| 50 | caveman | **optional** | was pre-ticked; see shell split |
| 60 | codegraph | **optional** | was pre-ticked; registers in every account |
| 70 | accounts | on | replaces `claude2`; per-account rows |
| 80 | effort-router | on | |
| 90 | plugins | on | |
| 92 | statusline | optional | |
| 95 | agents-skills | on | |
| 99 | model | on | `opus[1m]` |

`gum` (10) is dropped: ratatui replaces it. The bash tool keeps working until the Rust one
passes its tests; then the bash files are deleted in the same branch.

Each component keeps the detect / plan / apply split and `deps`. Detect is read-only and
safe to run concurrently; apply runs serially in order.

## Constraints

- Linux only (dev box, Debian 13). Single binary built with `cargo build --release`;
  the repo's `claude-strapon` path stays the entry (wrapper or the binary itself, builder's call).
- External tools stay external: `claude`, `npm`, `systemctl --user`, `ss`, `rtk`,
  `caveman`, `curl … | bash` installers. JSON handling moves in-process (serde_json); `jq`
  is no longer required.
- JSON writes: backup `*.bak-<timestamp>`, write tmp in same dir, re-parse, rename;
  resolve symlinks first; mode 0600 on new files.
- `claude mcp get` is the only "Connected" proof for MCP; keep it, run in parallel.
- The agent working on this host may run `--dry-run` only; the owner runs apply.

## Tests

Port `tests/test-claude-strapon.sh` scenarios (sandbox HOME, stub binaries on a
stub-only PATH, stdin closed) to Rust integration tests that exec the built binary.
Must cover: dry run and no-tty change nothing; JSON write safety; symlink write-through;
optional components unticked unless `--only`; accounts discovery with 2 and 3 accounts;
`--add-accounts`; route never written while :8787 is down; RC strip for `claudeN`.

## Settled after builder review (designer's calls, owner to check)

1. **Launchers.** Static `files/claude-accounts.sh`, discovery at shell start (see the
   shell file split). Revised from a generated file during the final review: nothing to
   regenerate, nothing in `.bashrc` to rewrite.
2. **Accounts are one component** (`accounts`, order 70). The board shows a sub-row per
   account; picking, `--only` and `--redo` work on `accounts` as a whole. "Add accounts"
   is a number prompt (default 0) that appears after the pick when `accounts` is ticked.
   On a terminal, `accounts` is always in the pick-list (unticked when it reads ok) so
   accounts can be added on any run. `--add-accounts N` ticks `accounts`; without
   `--yes` it still stops after the preview.
3. **Accounts not logged in yet.** Shown as `!` sub-rows. They do not count against
   ok/exit code (same as `needs` today). A later run picks up newly logged-in ones.
4. **Speed bar.** Dry run on the dev box ≤ 4 s with the same stack, through parallelism.
   `claude mcp get`, `claude plugin list` and `rtk init --show` stay as the sources of
   truth: no reading their private files.
5. **UI tests.** Integration tests cover the non-interactive paths (dry run, no tty,
   `--yes`, `--only`, `--add-accounts`). Selection logic is a pure function with unit
   tests. No PTY harness.
6. **No alias migration** (see 1): aliases are equivalent and left alone.
7. **Numbering.** Discovery takes N ≥ 2 only. New accounts get max(N)+1; gaps stay gaps.
   New account dirs are mode 0700 (they will hold credentials).
8. **The `.bashrc` lines.** Apply only appends a missing `. "<root>/files/…"` line,
   never edits or removes one. The settings.json route is still stripped by caveman apply
   only. Nothing removes a route the owner already had.
9. **Backups never overwrite each other.** `*.bak-YYYYmmdd-HHMMSS`, plus `-2`, `-3`… if that
   name exists. (The bash tool lost the pre-run state when several edits landed in one
   second.)
10. **JSON key order is preserved** (serde_json `preserve_order`), as jq did.
11. **`--only`/`--redo` with an unknown name is an error** (exit 2), naming the valid
    components; `claude2` gets the hint "now: accounts".
12. **Plan lines come from the scan.** Detect returns state, detail and the plan lines
    for the facts it just measured, so the preview does not re-run slow calls. Apply
    measures again before it acts.

## Left to the builder

Crate layout, async vs threads, exact ratatui widgets, colour scheme, whether launchers
are a static file or generated.
