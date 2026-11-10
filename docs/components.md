# Components reference

What each component checks (detect, read-only) and what it changes (apply), in apply
order. Source: `src/comp/<name>.rs`. "Deps": a component is skipped when one of these
failed earlier in the same run.

States: `●` ok · `◐` partial · `○` missing · `✗` broken · `!` needs you (a component
whose state is `!` is listed in a separate "needs you" section under the board and is
never offered for apply).

---

## node22

Node 22 under nvm. caveman's CLI and codegraph are npm globals pinned to it. Their MCP
registrations use absolute paths into it, because Claude Code never loads nvm.

| | |
|---|---|
| Default | on · deps: none |
| Finds nvm at | `$NVM_DIR`, else `~/.config/nvm` if it exists, else `~/.nvm` |
| `●` | a `versions/node/v22.*` exists (newest by version number is used) |
| `○` | `nvm.sh` present, no Node 22 |
| `!` | no nvm at all: install nvm yourself, then re-run |
| Apply | `. $NVM_DIR/nvm.sh && nvm install 22` |

## claude-code

| | |
|---|---|
| Default | on · deps: none |
| `●` | `claude` on PATH (shows `claude --version`) |
| `○` | not installed |
| Apply | `curl -fsSL https://claude.ai/install.sh \| bash`, then `~/.local/bin` is put on PATH for the rest of the run. Log in once by hand. |

## rtk

[rtk](https://github.com/rtk-ai/rtk) (Rust Token Killer): a PreToolUse(Bash) hook that
compresses command output before Claude reads it.

| | |
|---|---|
| Default | **optional** · deps: claude-code |
| `●` | `rtk gain` works and `rtk init --show` reports `[ok] Hook` and `[ok] settings.json` |
| `◐` | installed, hook not wired |
| `✗` | `rtk gain` fails: the `rtk` on PATH is a different tool (Rust Type Kit) |
| `○` | not installed |
| Apply | if missing: rtk's install script (`curl … \| sh`); then `rtk init -g --auto-patch` (hook, `RTK.md`, `@RTK.md` in `~/.claude`) |

## caveman

The caveman compression proxy on 127.0.0.1:8787, its hooks, its recovery MCP server, and
the shell route. See the README section *The caveman route* for why the route lives in
the shell.

| | |
|---|---|
| Default | **optional** · deps: node22, claude-code |
| Measures | CLI (`caveman` on PATH or in Node 22's bin); proxy binary `~/.caveman/bin/caveman-proxy`; `systemctl --user is-enabled caveman-proxy.service`; who holds :8787 (`ss -tlnHe`); route in `settings.json` env; route line in `~/.bashrc`; caveman hooks in settings; `claude mcp get caveman` (account 1) |
| `●` | proxy up and yours, shell route, hooks, service, MCP |
| `◐` | something apply can fix: not a service, route still in `settings.json` (blocks Remote Control), shell route missing, MCP missing. If binaries or hooks are also missing, the detail adds "run `caveman claude` once for: …" |
| `✗` | a route is set and nothing listens on :8787 (every routed session fails), or :8787 belongs to another user (a route would hand them your credentials) |
| `!` | only proxy binaries or hooks are missing (nothing apply can fix): run `caveman claude` once, then re-run |
| `○` | CLI not installed |

Apply, in order:
1. `npm i -g @caveman-ai/cli@2.1.0` with Node 22 first on PATH, if the CLI is missing.
2. If the proxy binary exists and the unit is not enabled: install
   `files/caveman-proxy.service` to `~/.config/systemd/user/`, then `daemon-reload` and
   `enable --now`. The unit restarts every 15 s for ever, so it takes over the port if a
   proxy started by `caveman claude` exits.
Steps 2–4 run only when the proxy binary `~/.caveman/bin/caveman-proxy` exists.

3. If the shell route is missing: wait for :8787 (up to 5 s, only if step 2 just started
   the unit). If the port is yours or root's, append
   `. "<repo>/files/claude-route.sh"` to `~/.bashrc`. Otherwise stop with an error and
   write nothing.
4. If `settings.json` still holds the route: remove `env.ANTHROPIC_BASE_URL` and
   `env._CLAUDE_CODE_ASSUME_FIRST_PARTY_BASE_URL` (other env keys stay).
5. If the MCP server is missing: `caveman mcp install claude` (account 1; `accounts`
   copies it into the others).

Open shells keep their old environment. Open a new shell after apply.

## codegraph

The codegraph MCP server
([@colbymchenry/codegraph](https://www.npmjs.com/package/@colbymchenry/codegraph)). Its
tools appear only in repos where you ran `codegraph init`.

| | |
|---|---|
| Default | **optional** · deps: node22, claude-code |
| Measures | the package under Node 22 (`lib/node_modules/@colbymchenry/codegraph/npm-shim.js`), then `claude mcp get codegraph` in account 1 and every logged-in extra account, all at once |
| `●` | every account: Connected, and registered with the absolute shim path |
| `◐` | some account not registered, not connected, or registered as a bare `codegraph` command (fails with ENOENT because Claude Code never loads nvm) |
| `○` | package not installed |
| Apply | `npm i -g @colbymchenry/codegraph` if missing; then, per account: `claude mcp remove codegraph -s user`, then `claude mcp add codegraph -s user -- <node22>/node <shim> serve --mcp` |

## accounts

Every extra Claude account: `~/.claude-accountN`, N ≥ 2. See the README section
*Accounts*.

| | |
|---|---|
| Default | on, and always offered on a terminal (so accounts can be added) · deps: none |
| Links checked | `settings.json settings.local.json CLAUDE.md RTK.md agents skills plugins`, each must be a symlink to the same name in `~/.claude` |
| Launchers | `~/.bashrc` sources `files/claude-accounts.sh` (or `files/claude-route.sh`, which sources it) |

Per-account sub-rows:

| Row | Meaning |
|---|---|
| `● 7/7 links · MCP in sync` | done |
| `◐ k/7 links …` | links missing (apply creates them) |
| `◐ links ok · MCP missing: a b` | account 1 has MCP servers this login lacks (apply copies them) |
| `! log in once: claudeN` | no `.claude.json` yet; links are fine |
| `✗ not links, move aside by hand: f` | a real file or a link elsewhere sits where a shared link belongs; never touched |

The component row is `✗` if any account is, `◐` if any account is or the launcher line
is missing (only when extra accounts exist), else `●`. `!` rows never fail a run.

Apply:
1. `--add-accounts N` (or the number you typed): create `~/.claude-account<max+1>`… with
   mode 0700.
2. For every account: create the missing links, never replacing anything that exists.
3. For every logged-in account with missing MCP servers: merge account 1's
   `mcpServers` into its `.claude.json`. The account's own entries win, and all other
   keys (its login) pass through untouched. The write is a safe JSON edit with a backup.
4. If extra accounts exist and no launcher line is in `~/.bashrc`, append
   `. "<repo>/files/claude-accounts.sh"`.

## effort-router

A UserPromptSubmit hook that tags each prompt with a complexity level
(`files/effort-router.sh`). The hook runs the repo's copy by absolute path.

| | |
|---|---|
| Default | on · deps: claude-code |
| `●` | exactly one UserPromptSubmit hook mentions `effort-router`, and it is `<repo>/files/effort-router.sh` |
| `◐` | it points elsewhere (an old path, a worktree, a second copy) |
| `○` | no such hook |
| Apply | settings.json: drop every UserPromptSubmit entry that runs an `effort-router`, keep all other hooks, add `<repo>/files/effort-router.sh` with timeout 10 |

## plugins

| | |
|---|---|
| Default | on · deps: claude-code |
| Measures | `claude plugin list --json` (user scope) and `claude plugin marketplace list`, in parallel |
| `●` | every plugin in `files/plugins.txt` installed and enabled, none from `files/plugins-remove.txt`, every marketplace in `files/marketplaces.txt` added |
| `◐` | lists what is off: `add marketplace:`, `install:`, `remove:`, `enable:` |
| `✗` | `claude plugin list` failed |
| Apply | `claude plugin marketplace add <source>`, `install -s user`, `enable`, `uninstall -s user -y`, in that order. Plugins on neither list are left alone. |

List formats: `plugins.txt` / `plugins-remove.txt` hold one `plugin@marketplace` per line;
`marketplaces.txt` holds `name source` per line; `#` starts a comment. Extra accounts get
the same plugins through their `plugins/` and `settings.json` links.

## statusline

caveman's mode badge in the Claude Code status line.

| | |
|---|---|
| Default | **optional** (cosmetic) · deps: plugins |
| `●` | the self-updating command (looks up the newest caveman plugin copy each time it runs), or any custom command you set (left alone) |
| `◐` | pinned as `bash "<path>"` to one plugin version: works until the next plugin update |
| `✗` | pinned to a script that no longer exists |
| `○` | not configured |
| Apply | settings.json `statusLine` = the self-updating command; refuses if the caveman plugin is not installed |

## agents-skills

| | |
|---|---|
| Default | on · deps: claude-code |
| Measures | each file under `files/agents/` and `files/skills/` against the same path under `~/.claude`, byte for byte |
| `●` / `◐` / `○` | all current / some missing or different (named) / none installed |
| Apply | copy the missing or different files (mode 0644). Files in `~/.claude` that are not from this repo are left alone; nothing is deleted. |

## model

| | |
|---|---|
| Default | on · deps: claude-code |
| `●` | `settings.json` `model` is `opus[1m]` |
| `◐` | another model is pinned (untick it if that is deliberate) |
| `○` | not pinned |
| Apply | settings.json `model = "opus[1m]"` |

The wanted model is the constant `MODEL` in `src/comp/model.rs`.
