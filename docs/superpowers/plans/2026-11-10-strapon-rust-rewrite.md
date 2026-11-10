# claude-strapon Rust rewrite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the bash claude-strapon with one Rust binary (ratatui picker) that supports N accounts and makes rtk/caveman/codegraph optional.

**Architecture:** `src/main.rs` runs scan → board → pick → preview → confirm → apply → rescan. Each component is a `Component` value with `detect` (read-only, run in parallel threads, returns state + detail + plan lines) and `apply` (serial, re-measures). Shared helpers: `ctx.rs` (paths, subprocesses, account discovery), `json.rs` (safe JSON edit), `ui.rs` (colour, board, prompts, ratatui picker). Shell launchers stay bash: `files/claude-accounts.sh`, `files/claude-route.sh`.

**Tech Stack:** Rust 2024 (cargo 1.99), ratatui 0.30 (crossterm backend), serde_json with `preserve_order`. Nothing else.

**Spec:** `docs/superpowers/specs/2026-11-10-strapon-rust-rewrite-design.md`

**Workspace:** worktree `$HOME/claude-strapon-rust`, branch `rust-rewrite`. The main checkout `~/claude-strapon` is live (every shell sources its `files/claude-route.sh`) and is not touched.

## Global Constraints

- Same flags: `--dry-run`, `--yes/-y`, `--only a,b`, `--redo a,b`, `-h`; new `--add-accounts N`. Unknown component name: exit 2, list valid names; `claude2` hints `now: accounts`.
- Marks: `●` ok, `◐` partial, `○` missing, `✗` broken, `!` needs. Needs never in the pick-list, never fails the exit code.
- No tty and no `--yes`: print preview, `No terminal: stopping after the preview. Re-run with --yes to apply.`, exit 0. `--dry-run`: `Dry run: nothing changed.`
- Exit 1 if any picked component is not ok after the rescan (`Some picked components still do not read ok (see above).`), else `All picked components read ok.`; empty candidates: `Nothing to fix.`
- Optional (unticked unless named in `--only`): rtk, caveman, codegraph, statusline.
- JSON writes: resolve symlinks, refuse if the existing file does not parse, backup `<file>.bak-YYYYmmdd-HHMMSS[-2…]` (UTC), tmp in same dir, re-parse, rename, mode 0600, key order preserved.
- Root = `$STRAPON_ROOT` or compile-time `CARGO_MANIFEST_DIR`. `CLAUDE_DIR` env overrides `~/.claude`. `NVM_DIR` as today.
- Caveman route `http://127.0.0.1:8787/w/claude`; CLI pin `@caveman-ai/cli@2.1.0`; codegraph pkg `@colbymchenry/codegraph`; model `opus[1m]`.
- Account links: `settings.json settings.local.json CLAUDE.md RTK.md agents skills plugins`. Extra accounts are `~/.claude-accountN`, N ≥ 2, digits only, no leading zero; new dirs 0700; new N = max+1.
- `.bashrc` is only appended to, never edited.

## Review Focus

1. An existing `settings.json` that does not parse: no write, no `{}` overwrite; the edit fails loudly. (json unit test, Task 1)
2. `$HOME` with a space or quote: the eval'd launcher still sets the right `CLAUDE_CONFIG_DIR`. (shell test, Task 2)
3. A detector that panics or a missing external tool: board shows `✗ detector failed`, the run goes on. (Task 3 test with no `rtk` stub)
4. Junk dirs `~/.claude-account`, `~/.claude-account02`, `~/.claude-accountx`, `~/.claude-account1`: ignored by both the binary and the shell file. (Tasks 2 and 4)
5. Running from a git worktree writes the worktree's path into hooks and `.bashrc`: the preview warns when `<root>/.git` is a file. (Task 3)

---

### Task 1: Crate, context, safe JSON edit

**Files:**
- Create: `Cargo.toml`, `src/main.rs` (stub `fn main` for now), `src/ctx.rs`, `src/json.rs`
- Modify: `.gitignore` (add `/target`)

**Interfaces:**
- Produces (`ctx.rs`):
  - `pub struct Ctx { pub root, pub home, pub claude_dir, pub claude_json, pub settings, pub nvm_dir, pub bashrc: PathBuf }`, `Ctx::from_env() -> Ctx`
  - `Ctx::settings_json(&self) -> serde_json::Value` (`{}` when missing or unparseable)
  - `Ctx::node22_bin(&self) -> Option<PathBuf>` (newest `versions/node/v22.*` by numeric compare, `/bin` appended)
  - `Ctx::accounts(&self) -> Vec<Account>` (extras only, sorted by n)
  - `pub struct Account { pub n: u32, pub dir: PathBuf }` with `name() -> String` (`claudeN`), `logged_in() -> bool` (`.claude.json` exists)
  - `pub fn claude_cmd(ctx: &Ctx, dir: Option<&Path>) -> Command` (None = account 1: `env_remove("CLAUDE_CONFIG_DIR")`; Some = sets it)
  - `pub fn out(cmd: &mut Command) -> Option<String>` (stdout when exit 0, stdin null, stderr null)
  - `pub fn ok(cmd: &mut Command) -> bool` (exit 0, quiet)
  - `pub fn run(cmd: &mut Command) -> bool` (inherit stdout/stderr, for apply)
  - `pub fn which(name: &str) -> Option<PathBuf>`
- Produces (`json.rs`): `pub fn read(p: &Path) -> Option<Value>`, `pub fn edit(p: &Path, f: impl FnOnce(&mut Value)) -> Result<(), String>`, `fn backup_path(p: &Path, secs: u64) -> PathBuf`, `fn stamp(secs: u64) -> String`

- [ ] **Step 1: Write failing unit tests in `src/json.rs`**: `stamp(0) == "19700101-000000"`, `stamp(1762768330) == "20251110-095210"`; `edit_preserves_key_order` (`{"z":1,"a":2}` + set `m` gives keys `z,a,m`); `edit_refuses_unparseable` (file `{bad` stays byte-identical, `Err`, no `.tmp` left); `edit_writes_through_symlink` (link stays a link, target changed); `edit_mode_0600`; `backups_never_collide` (two edits in one second leave two distinct `.bak-*` files, the first holds the original).
- [ ] **Step 2:** `cargo test` → fails (no impl).
- [ ] **Step 3:** Implement `ctx.rs` and `json.rs`. Civil-from-days for `stamp` (no date crate). Tmp via `OpenOptions::create_new` + mode 0600 in the target dir.
- [ ] **Step 4:** `cargo test` → pass. `cargo clippy -- -D warnings` clean.
- [ ] **Step 5:** Commit `feat(rust): crate skeleton, context, safe JSON edit`.

### Task 2: Shell launchers for N accounts

**Files:**
- Create: `files/claude-accounts.sh`, `tests/shell.sh` (from section 9 of `tests/test-claude-strapon.sh`), `tests/shell.rs` (runs `bash tests/shell.sh`, asserts exit 0 + `PASS`)
- Modify: `files/claude-route.sh` (only the two exports, then `. "$(dirname "${BASH_SOURCE[0]}")/claude-accounts.sh"`)

**Interfaces:** Produces shell functions `_claude_launch`, `claude`, `claude1`, `claudeN` (see spec "Shell file split").

- [ ] **Step 1: Write `tests/shell.sh`** keeping every existing `want` line, except `type -t claude1` now expects `function`. Add, with `HOME=$T/home` holding `.claude-account3`, `.claude-account02`, `.claude-accountx`, and a second HOME `"$T/h o'me"`:
  - `claude3` → `url=$R fp=1 dir=$HOME/.claude-account3 args=`; `claude3 rc` → `url= fp= dir=$HOME/.claude-account3 args=rc`
  - `CLAUDE_CONFIG_DIR=/x claude1` → `dir=` (account 1)
  - `type -t claude02 claudex || echo none` → `none`
  - accounts file alone (no route): `claude3` → `url=inherited …`, `claude3 rc` → `url= …`
  - spaced/quoted HOME: `claude3` prints that dir exactly
  - alias `claude2` survives sourcing (existing check)
- [ ] **Step 2:** `cargo test --test shell` → fails.
- [ ] **Step 3:** Write `files/claude-accounts.sh`: glob `"$HOME"/.claude-account[1-9]*`, keep `^[1-9][0-9]*$` and N ≥ 2, skip names that are aliases, wrap an executable on PATH, else `eval` a function with `printf %q` of the dir.
- [ ] **Step 4:** `cargo test --test shell` → pass.
- [ ] **Step 5:** Commit `feat(shell): claudeN launchers for every ~/.claude-accountN; route file sources them`.

### Task 3: Component model, main flow, file-only components, test harness

**Files:**
- Create: `src/comp/mod.rs`, `src/comp/{claude_code,node22,effort_router,model,agents_skills,statusline}.rs`, `src/ui.rs`, `tests/cli.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Produces (`comp/mod.rs`):
  - `#[derive(Clone, Copy, PartialEq, Debug)] pub enum State { Ok, Partial, Missing, Broken, Needs }`
  - `pub struct Report { pub state: State, pub detail: String, pub rows: Vec<Row>, pub plan: Vec<String> }`, `Report::new(state, detail) -> Report`
  - `pub struct Row { pub state: State, pub name: String, pub detail: String }`
  - `pub struct Opts { pub add_accounts: u32 }`
  - `pub struct Component { pub name: &'static str, pub deps: &'static [&'static str], pub optional: bool, pub always_offer: bool, pub detect: fn(&Ctx) -> Report, pub apply: fn(&Ctx, &Opts) -> Result<(), String> }`
  - `pub fn all() -> Vec<Component>` in apply order: node22, claude-code, rtk, caveman, codegraph, accounts, effort-router, plugins, statusline, agents-skills, model
  - `pub fn pick_defaults(comps: &[Component], reports: &[Report], only: &[String], redo: &[String], interactive: bool, add: u32) -> (Vec<usize>, Vec<bool>)` → candidate indices + pre-ticked flags
- Produces (`ui.rs`): `mark(State) -> String`, `head(&str)`, `board(title, &[Component], &[Report])`, `interactive() -> bool`, `confirm(&str) -> bool`, `ask_number(&str) -> u32`, `choose_multi(header, &[&str], &[bool]) -> Vec<usize>` (plain numbered for now; ratatui in Task 6)
- `tests/cli.rs` harness: `Sandbox::new()` builds temp HOME as in the bash test, a stub-only PATH dir (stubs `claude`, `rtk`, `ss`, `systemctl`, `pgrep`, `npm`, `curl`, `sudo`, `node`, plus links to `bash`, `jq`, `env`, `cat` for stubs to use), `run(args) -> (code, stdout)` with stdin null, `snapshot()` of files newer than a stamp.

- [ ] **Step 1: Unit tests for `pick_defaults`**: optional component not pre-ticked; same component pre-ticked when in `--only`; ok component offered only when in `--redo`; `needs` never offered; `accounts` (always_offer) offered unticked when interactive and ok, not offered when non-interactive; `add > 0` offers and ticks `accounts`.
- [ ] **Step 2: Integration tests** (ported from bash tests 1–4 and 7, restricted to the components of this task): `dry_run_changes_nothing` (also no-tty run stops after preview, `snapshot()` empty); `apply_file_components` (`--yes --only effort-router,model,agents-skills`: model `opus[1m]`, hooks `other-hook,<root>/files/effort-router.sh`, agent file copied, a `.bak-*` exists, `All picked components read ok`); `rerun_is_noop` (`Nothing to fix`, no new backup); `statusline_follows_updates`; `unknown_component_exits_2`; `missing_tool_is_broken_not_fatal` (Review Focus 3); `worktree_warning` (Review Focus 5: root with `.git` file → preview line containing `worktree`).
- [ ] **Step 3:** `cargo test` → new tests fail.
- [ ] **Step 4:** Implement `comp/mod.rs`, the six components (ports of the bash ones; rtk/caveman/codegraph/accounts/plugins registered as stubs returning `Missing` until Tasks 4–5), `ui.rs`, `main.rs` flow. Scan with `std::thread::scope`, one thread per component, `catch_unwind` → `Report::new(Broken, "detector failed")`. Sessions warning when caveman, codegraph or accounts is picked and `pgrep -x claude` finds any.
- [ ] **Step 5:** `cargo test` → pass; clippy clean.
- [ ] **Step 6:** Commit `feat(rust): component model, flow, file-only components`.

### Task 4: accounts component

**Files:** Create `src/comp/accounts.rs`; Modify `src/main.rs` (`--add-accounts N`), `tests/cli.rs`.

**Interfaces:** Consumes `Ctx::accounts`, `json::edit`. Produces `accounts::add_plan(ctx: &Ctx, n: u32) -> Vec<String>` (preview lines for new accounts, called by main).

- [ ] **Step 1: Integration tests:** `accounts_merge_mcp` (bash test 2's claude2 asserts: links kept, MCP `codegraph,dune`, `oauthAccount.x == 1`, 0600); `three_accounts` (add `.claude-account3` logged in, MCP gap → board row `claude3` partial, apply fixes it, links created); `not_logged_in_is_needs` (dir without `.claude.json` → `!` row `log in once: claude4`, picked accounts still exits 0); `add_accounts` (`--yes --add-accounts 2` with 2 and 3 present → creates 4 and 5, mode 700, 7 links each, `.bashrc` gains `. "<root>/files/claude-accounts.sh"` once; re-run adds nothing); `add_accounts_dry_run` (lists `create … .claude-account4`, writes nothing); `junk_dirs_ignored` (Review Focus 4); `real_file_blocks_link` (regular `settings.json` in an account → `✗ … move aside by hand`, untouched).
- [ ] **Step 2:** `cargo test` → fail.
- [ ] **Step 3:** Implement. Launcher line is satisfied by either the accounts line or the existing caveman route line (the route file sources the accounts file). Required only when extras exist.
- [ ] **Step 4:** `cargo test` → pass.
- [ ] **Step 5:** Commit `feat(rust): accounts component, --add-accounts`.

### Task 5: rtk, caveman, codegraph, plugins

**Files:** Create `src/comp/{rtk,caveman,codegraph,plugins}.rs`; Modify `tests/cli.rs`.

- [ ] **Step 1: Integration tests:** `optional_unticked` (no-tty `--dry-run` with rtk/caveman/codegraph not ok: their plan lines absent from the preview; present with `--only caveman`); `dead_route_broken` (bash test 5); `route_moves_to_shell` (bash test 8: env keeps only `ENABLE_TOOL_SEARCH`, `.bashrc` route line exactly once, re-run `Nothing to fix`); `route_not_written_while_down` (`ss` stub empty, bins present → apply does not append the route line, reports failure); `plugins_diff` (stripe flagged `remove: stripe`, apply calls the uninstall stub); `codegraph_every_logged_in_account` (stub records `mcp add` per `CLAUDE_CONFIG_DIR`; accounts 1, 2, 3 registered, unlogged 4 not).
- [ ] **Step 2:** `cargo test` → fail.
- [ ] **Step 3:** Implement as ports of `components/40,50,60,90`. Caveman apply: after enabling the unit, poll `ss` up to 5 × 1 s before appending the route line. Plugins: `plugin list --json` and `marketplace list` in parallel threads. Codegraph: one thread per account for `mcp get`.
- [ ] **Step 4:** `cargo test` → pass.
- [ ] **Step 5:** Commit `feat(rust): rtk, caveman, codegraph, plugins (rtk/caveman/codegraph optional)`.

### Task 6: ratatui picker

**Files:** Modify `src/ui.rs`.

- [ ] **Step 1:** Unit test the pure key handler `fn step(sel: &mut Picker, key: KeyCode) -> Option<Vec<usize>>`: Down/Up/j/k move with clamping; Space toggles; `a` toggles all; Enter returns ticked indices; Esc/`q` returns empty.
- [ ] **Step 2:** Implement `choose_multi` with `ratatui::init_with_options(TerminalOptions { viewport: Viewport::Inline(items + 3) })`, restore on every exit path; fall back to the numbered prompt if init fails. Number and y/N prompts stay plain lines.
- [ ] **Step 3:** `cargo test` → pass. Manual check left to the owner (needs a real terminal).
- [ ] **Step 4:** Commit `feat(rust): ratatui pick-list`.

### Task 7: Cut over

**Files:** Modify `claude-strapon` (becomes `exec cargo run --release --quiet --manifest-path "$ROOT/Cargo.toml" -- "$@"`), `README.md`; Delete `lib/`, `components/`, `tests/test-claude-strapon.sh`.

- [ ] **Step 1:** `cargo build --release`; `time ./claude-strapon --dry-run </dev/null` against the real machine (read-only). Expect ≤ 4 s and a board matching the bash tool's run from the main checkout except: no gum row; rtk/caveman/codegraph unticked; `accounts` replaces `claude2`.
- [ ] **Step 2:** Delete the bash files, update README (layout, `--add-accounts`, optional components, Rust build, checks: `cargo test`, `cargo clippy -- -D warnings`, `shellcheck files/*.sh`).
- [ ] **Step 3:** `cargo test && cargo clippy -- -D warnings && shellcheck -x files/claude-accounts.sh files/claude-route.sh` → clean.
- [ ] **Step 4:** Commit `feat!: claude-strapon is a Rust binary; bash implementation removed`.
