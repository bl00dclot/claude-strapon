# claude-strapon

One command that measures the Claude Code stack on a Linux machine, shows what is there,
and fixes what you pick. It covers Node 22, Claude Code, rtk, the caveman compression
proxy, codegraph, any number of Claude accounts (`claude2`, `claude3`, …) sharing one
global config, the effort-router hook, plugins, the statusline badge, agents + skills
and the default model.

```
claude-strapon · my-box · scan
  ● node22          v22.22.3 under ~/.config/nvm
  ● claude-code     2.1.322 (Claude Code)
  ● rtk             0.42.4 · hook ok
  ● caveman         :8787 up · shell route (Remote Control direct) · hooks · service · MCP
  ● codegraph       1.0.1 · MCP connected · per repo: codegraph init
  ● accounts        claude2 claude3 · launchers in ~/.bashrc
      ● claude2     7/7 links · MCP in sync
      ! claude3     log in once: claude3
  ● effort-router   hook -> ~/claude-strapon/files/effort-router.sh
  ● plugins         19 wanted, all enabled
  ● statusline      caveman badge (follows plugin updates)
  ◐ agents-skills   1 of 10 missing or outdated: issue-board
  ◐ model           sonnet (wanted opus[1m])
```

Contents: [Quick start](#quick-start) · [How a run works](#how-a-run-works) ·
[Flags](#flags) · [Components](#components) · [Accounts](#accounts) ·
[The caveman route](#the-caveman-route) · [Safety](#safety) ·
[Troubleshooting](#troubleshooting) · [Undo and uninstall](#undo-and-uninstall) ·
[Development](#development) · [Security](#security) · [More docs](#more-docs) ·
[License](#license)

## Quick start

Requirements: Linux with bash, `cargo` ([rustup.rs](https://rustup.rs)), and
[nvm](https://github.com/nvm-sh/nvm) for the Node 22 pieces. The tool can install
everything else.

```bash
git clone git@github.com:bl00dclot/claude-strapon.git ~/claude-strapon
cd ~/claude-strapon
./claude-strapon --dry-run     # look first: board + what it would do, changes nothing
./claude-strapon               # pick, confirm, apply, rescan
```

The first run compiles the binary (`cargo build --release`). That takes about a minute,
longer on a busy machine, and prints nothing while it builds. Later runs start
immediately and rebuild only when the source changed.

Keep the clone where it is. The hooks and `~/.bashrc` lines it installs point into it by
absolute path (`files/effort-router.sh`, `files/claude-accounts.sh`,
`files/claude-route.sh`). If you move the clone, run it again from the new place.

## How a run works

```
scan -> board -> pick -> preview -> confirm -> apply -> rescan
```

1. **Scan.** Every component checks the machine at the same time, read-only. The slowest
   single check sets the pace: `claude mcp get codegraph`, which starts the server to
   health-check it. That takes a few seconds on an idle machine.
2. **Board.** One row per component, and a sub-row per extra account:

   | Mark | Meaning |
   |---|---|
   | `●` | wired and verified by a live check |
   | `◐` | installed, not (fully) wired |
   | `○` | missing |
   | `✗` | wired but broken: fix this first |
   | `!` | needs you: a login or a by-hand step. Listed under "needs you" below the board, never in the pick-list, never fails the run |

   "Verified" means a live check, not a config entry: `claude mcp get` says Connected,
   your own process listens on :8787, `rtk init --show` reads `[ok]`.
3. **Pick.** On a terminal you get a list:
   - `↑`/`↓` (or `j`/`k`) move, `space` toggles, `a` toggles all, `enter` confirms,
     `esc`/`q` cancels.
   - Anything not `●` and not `!` is offered. Optional components start unticked.
   - `accounts` is always offered so you can add accounts on any run. Ticking it asks
     how many to add.
4. **Preview.** The exact commands and file edits for each picked component, plus two
   possible warnings:
   - claude sessions are running, shown when caveman, codegraph or accounts is picked
     (sessions rewrite `.claude.json` and can undo MCP changes);
   - the checkout is a git worktree.
5. **Confirm**, then **apply** in a fixed order. If a component's dependency failed in
   this run, that component is skipped.
6. **Rescan.** The second board is the proof. If any picked component does not read `●`,
   the run exits 1.

Without a terminal (a script, a pipe, `ssh host cmd`) and without `--yes`, the run stops
after the preview and changes nothing.

## Flags

| Flag | Effect |
|---|---|
| *(none)* | the full run above |
| `--dry-run` | stop after the preview; change nothing |
| `--yes`, `-y` | take the pre-ticked fixes and apply without prompting |
| `--only a,b` | limit the run to these components; ticks optional ones too |
| `--redo a,b` | offer these even when they read `●` (re-apply) |
| `--add-accounts N` | create N more accounts; implies `accounts` |
| `-h`, `--help` | usage and the component list |

Exit codes:
- `0`: done, or nothing to do.
- `1`: a picked component still does not read `●` after apply.
- `2`: bad arguments or an unknown component name.

```bash
./claude-strapon --yes --only plugins,model     # unattended, limited
./claude-strapon --redo codegraph               # re-register codegraph everywhere
./claude-strapon --yes --add-accounts 2         # claude3 + claude4 (next free numbers)
```

## Components

Components are applied in this order. Optional ones start unticked; tick them in the list
or name them with `--only`. The exact checks and actions for each component are in
[docs/components.md](docs/components.md).

| Component | Default | What it sets up |
|---|---|---|
| `node22` | on | Node 22 under nvm (`nvm install 22`); the npm-based tools are pinned to it |
| `claude-code` | on | Claude Code, native installer (`~/.local/bin/claude`) |
| `rtk` | optional | [rtk](https://github.com/rtk-ai/rtk), a hook that compresses shell output (`rtk init -g`) |
| `caveman` | optional | the caveman compression proxy on 127.0.0.1:8787, its systemd user unit, the shell route and its recovery MCP server |
| `codegraph` | optional | the codegraph MCP server, registered in every logged-in account |
| `accounts` | on | every `~/.claude-accountN`: links into `~/.claude`, MCP servers copied from account 1, `claudeN` launchers |
| `effort-router` | on | a UserPromptSubmit hook that tags each prompt with a complexity level |
| `plugins` | on | user-scope plugins from `files/plugins.txt`, minus `files/plugins-remove.txt`; marketplaces from `files/marketplaces.txt` |
| `statusline` | optional | caveman's mode badge, following plugin updates |
| `agents-skills` | on | subagents (`files/agents/`) and skills (`files/skills/`) copied into `~/.claude` |
| `model` | on | `model: opus[1m]` in settings (per session, `/model` still overrides it) |

To change what gets installed, edit the lists in `files/` (one entry per line, `#`
comments allowed) and re-run.

## Accounts

Each Claude subscription on the machine is an account:

- **Account 1** is the default: `~/.claude` plus `~/.claude.json`.
- **Account N** (N ≥ 2) is `~/.claude-accountN`, run as `CLAUDE_CONFIG_DIR=~/.claude-accountN claude`.

All accounts share one global config. In each extra account, `settings.json`,
`settings.local.json`, `CLAUDE.md`, `RTK.md`, `agents/`, `skills/` and `plugins/` are
symlinks into `~/.claude`, so a change made in any account applies to all of them.

Only `.claude.json` (that account's login and MCP servers) cannot be shared. Any MCP
server that account 1 has and the account lacks is copied in. Where both have an entry
with the same name, the account's own entry is kept.

**Adding accounts.** Tick `accounts` in the list (it asks how many), or use
`--add-accounts N`. New accounts take the next free numbers after the highest existing
one. Each gets a mode 0700 directory and all seven links. Then, once per account:

```bash
exec bash        # or open a new terminal: launchers are picked up when a shell starts
claude3          # log in
./claude-strapon # copies MCP servers into the new login, registers codegraph there
```

**Launchers.** `files/claude-accounts.sh` is sourced from `~/.bashrc`. When a shell starts,
it defines:
- `claude`;
- `claude1`, which always runs account 1, even when `CLAUDE_CONFIG_DIR` is set;
- `claudeN` for every `~/.claude-accountN` that exists at that moment.

Name clashes are respected:
- If you already have an alias with that name (say `alias claude2=...`), the alias is
  left alone.
- If an executable with that name is on PATH (a wrapper script), the launcher wraps it.

Only directories named `.claude-account` plus a number of 2 or more, with no leading
zero, count as accounts. Anything else is ignored.

## The caveman route

caveman compresses the traffic between Claude Code and the API through a local proxy on
127.0.0.1:8787. Claude Code reaches it through `ANTHROPIC_BASE_URL`.

**Why the shell, not settings.json.** The route is set in the shell
(`files/claude-route.sh`, sourced from `~/.bashrc`). Remote Control refuses any base URL
that is not api.anthropic.com, and `settings.json` env beats the shell, so a route in
settings made Remote Control impossible for every account.

**What gets routed.** The launchers drop the route for launches that start Remote
Control and keep it for everything else, including scripts and `claude -p` workers
started from that shell:
- direct: `claude remote-control`, `claude rc`, `claude --remote-control[=name]`;
- to bypass the proxy once: `unset ANTHROPIC_BASE_URL`.

**Moving an existing route.** `caveman claude` writes the route into `settings.json`.
When apply finds it there, it adds the shell route first and then removes the settings
one, so no step leaves claude unrouted.

Some parts can only be installed by `caveman claude` itself: the proxy binaries and its
hooks. The board marks those `!`.

## Safety

- **Read-only until you confirm.** `--dry-run` never writes, and neither does a run
  without a terminal unless you pass `--yes`.
- **JSON writes cannot break Claude Code.** Claude Code silently ignores a
  `settings.json` it cannot parse, which would switch off every hook in every account at
  once. Every write to `settings.json` or a `.claude.json`:
  - is refused if the file does not parse now (it is never replaced with `{}`);
  - is backed up next to the file as `<file>.bak-YYYYmmdd-HHMMSS` (UTC), with `-2`, `-3`…
    when several land in one second;
  - goes to a tmp file in the same directory (mode 0600), is parsed again, then renamed
    over the original;
  - keeps the key order.
- **Symlinks are written through**, never replaced, so the shared config stays shared.
- **`~/.bashrc` is only appended to**, never edited.
- **The route is never pointed at the wrong process.** Apply writes it only while
  something listens on :8787 *and* that listener belongs to you or root (checked with
  `ss -tlnHe`). Otherwise another local user could bind the port while the proxy is down
  and receive every session's credentials. The scan marks a route with nothing behind it,
  or with someone else behind it, `✗`.
- **It removes only what its preview names.** There are exactly three removals:
  - the caveman route in `settings.json` env (after the shell route is in place);
  - UserPromptSubmit hooks that run an `effort-router` (replaced by this repo's);
  - plugins listed in `files/plugins-remove.txt`.

  It never deletes files in `~/.claude`, and never removes a shell route or `.bashrc` line.

## Troubleshooting

| Board says | What it means / what to do |
|---|---|
| `caveman ✗ route set but nothing on :8787` | Every routed session fails. Start the proxy (`systemctl --user start caveman-proxy.service`), or `unset ANTHROPIC_BASE_URL` in the shell. |
| `caveman ✗ :8787 held by uid N, not you or root` | Someone else owns the port. Find it with `ss -tlnpe 'sport = :8787'`, stop it, then start your proxy. Do not route until it is yours. |
| `caveman ! run caveman claude once` | Proxy binaries or hooks are missing; only caveman's own installer adds them. Run `caveman claude`, then re-run this tool to move its route out of settings. |
| `codegraph ◐ … not the absolute-path form` | It was registered as a bare `codegraph` command, which fails because Claude Code never loads nvm. Re-apply with `--redo codegraph`. |
| `accounts ✗ not links, move aside by hand: …` | A real file sits where a shared link belongs. Move it aside (`mv file file.mine`), re-run, then merge anything you need from it. |
| `accounts ! log in once: claudeN` | Open a new shell and run `claudeN` to log in, then re-run. |
| `rtk ✗ rtk on PATH is not Rust Token Killer` | A different program named `rtk` (Rust Type Kit) comes first on PATH. |
| `statusline ✗ missing script` | The statusline was pinned to one plugin version that has since been replaced. Re-apply `statusline`. |
| `✗ detector failed` | That component's check crashed. Run `--only <name> --dry-run` to see it alone, and file an issue. |
| `… is a git worktree` warning | You are running from a worktree. Hooks and `.bashrc` lines would point into it and break when it is removed. Run from the main clone. |
| `claude session(s) running` warning | Open sessions rewrite `.claude.json` and can undo MCP changes. Close them first, or re-run afterwards and check the board. |
| A JSON edit fails with `<file> does not parse (…); not touching it` | Fix that file by hand (or restore its newest `.bak-*`), then re-run. |

## Undo and uninstall

- **Restore a JSON file.** Every edit left the previous version next to it.
  ```bash
  ls -t ~/.claude/settings.json.bak-* | head -1          # newest backup
  cp ~/.claude/settings.json.bak-20261110-115740 ~/.claude/settings.json
  ```
- **Remove the shell pieces.** Delete the lines that source `files/claude-accounts.sh`
  and `files/claude-route.sh` from `~/.bashrc`, then open a new shell.
- **Stop the proxy unit.** Run `systemctl --user disable --now caveman-proxy.service`,
  then `rm ~/.config/systemd/user/caveman-proxy.service`.
- **Detach an extra account.** Remove its directory, `~/.claude-accountN`. Its login goes
  with it. The shared files in `~/.claude` are untouched, because the account only held
  links to them.
- **Everything else** (plugins, MCP servers, rtk, codegraph) is removed with its own tool:
  `claude plugin uninstall`, `claude mcp remove`, `npm rm -g`.

## Development

```
claude-strapon            entry point: builds if needed, then runs the binary
Cargo.toml                Rust 2024; dependencies: ratatui, serde_json (preserve_order)
src/main.rs               flags, scan -> board -> pick -> preview -> apply -> rescan
src/ctx.rs                paths, account discovery, subprocess helpers
src/json.rs               the safe JSON edit (backup, tmp, re-parse, rename)
src/ui.rs                 colour, board, ratatui pick-list, plain prompts
src/comp/mod.rs           Component, Report, State, the apply order, pick defaults
src/comp/<name>.rs        one component each
files/                    what gets installed (shell files, hook, unit, lists, agents, skills)
files/router/             reference snapshots of the caveman proxy setup (not used by the code)
tests/cli.rs              the binary against a throwaway HOME with stub tools
tests/shell.sh            the launchers, in clean bash shells
docs/                     background and design records (see docs/README.md)
```

**Adding a component.** Write one file in `src/comp/` that defines
`pub const C: Component` (name, deps, optional, detect, apply), and add one line to
`all()`.
- `detect` must stay read-only, because it runs in parallel with the others. It returns
  the state, a one-line detail, and the plan lines for what it measured.
- `apply` measures again before it acts. On failure it returns `Err` with a message the
  board can show.

**Checks.** All must pass before a commit:
```bash
cargo test                                     # unit, CLI and shell tests
cargo clippy --all-targets -- -D warnings
shellcheck -S warning -x -s bash claude-strapon files/claude-accounts.sh files/claude-route.sh tests/shell.sh
```
The CLI tests run the real binary with `HOME` set to a temp dir and `PATH` set to a stub
directory only. A missing stub is "command not found", never the real `claude` or
`systemctl`, and nothing can reach your real `~/.claude`. While developing, run the
binary on your real HOME only with `--dry-run`.

## Security

The repository holds no credentials, tokens, keys, or machine-specific identifiers. It
was audited on 2026-11-10 in two passes:
- gitleaks v8.28.0 over every commit on every branch, and over the working tree;
- a grep of every file version ever committed for key and token patterns, emails, IP
  addresses, and host and user names.

Nothing was found besides loopback addresses, a fake login object used by the tests, and
placeholders. Snapshots under `files/router/` use `$HOME` for paths, and the one token in
them is stored as `<redacted>`.

Keep it that way:
- Never commit a real `~/.claude.json`, `.credentials.json`, a settings backup
  (`*.bak-*` is git-ignored), or proxy run files with live tokens. Redact before
  committing a snapshot.
- Write paths as `~` or `$HOME`, and name machines by role ("the dev box"), not hostname.
- Before pushing, scan: `gitleaks git . --log-opts=--all` and `gitleaks dir .`.

What the tool itself protects is listed under [Safety](#safety). Credentials stay where
Claude Code keeps them. The tool never reads or copies a login; when it merges MCP
servers into an account's `.claude.json`, the other keys pass through untouched.

## More docs

- [docs/components.md](docs/components.md): exact checks and actions for each component.
- [docs/README.md](docs/README.md): index of the background docs. Most of them predate
  this tool and cite the `~/ops` paths it was first built from. The components are the
  current procedure.
- [docs/superpowers/](docs/superpowers/): the design and plan of the Rust rewrite
  (2026-11-10).

## License

[MIT](LICENSE) © 2026 bl00dclot
