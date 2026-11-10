# Claude router — how both Claude instances reach the API
> **Historical.** Predates claude-strapon; paths and steps are from the original by-hand setup. Current procedure: [components.md](components.md). Index: [README.md](README.md).

Measured on the dev box, 2026-10-22. Snapshot of the live config: `setup/router/`.

## What it is

Every Claude Code session on this box — `claude` (account 1, `~/.claude`) and `claude2`
(account 2, `CLAUDE_CONFIG_DIR=~/.claude-account2`) — sends its API traffic through
**caveman-proxy**, a local compression proxy, instead of straight to Anthropic:

```
claude / claude2
   │  ANTHROPIC_BASE_URL=http://127.0.0.1:8787/w/claude   (per launch: files/claude-route.sh)
   ▼
caveman-proxy  127.0.0.1:8787   mode=compress, no inbound auth, no TLS (loopback only)
   │  /w/claude  ->  provider "anthropic"
   ▼
https://api.anthropic.com       (OAuth subscription token passes through unchanged)
```

`_CLAUDE_CODE_ASSUME_FIRST_PARTY_BASE_URL=1` keeps Claude Code treating the proxy as
first-party, so subscription OAuth and first-party features still work. This is not
claude-code-router (CCR): CCR sets `ANTHROPIC_AUTH_TOKEN` and breaks subscription auth
(see `setup/stack-status.md`). Caveman does not set a token.

The proxy also knows `gemini`, `openai`, `openai-codex` and `opencode-go` upstreams
(`setup/router/proxy-run.json`); only the `/w/claude` route is used here.

## Remote Control goes direct (2026-11-09)

`claude remote-control` refuses any `ANTHROPIC_BASE_URL` that is not api.anthropic.com,
and `_CLAUDE_CODE_ASSUME_FIRST_PARTY_BASE_URL` does not cover that check (Claude Code
2.1.319: "Remote Control is only available when using Claude via api.anthropic.com").
While the route sat in `settings.json` env it blocked Remote Control for both accounts:
settings env beats a shell `ANTHROPIC_BASE_URL=`/empty override, `--settings` before the
verb is refused, and a project `.claude/settings.local.json` override did not help (all
tested 2026-11-09).

So the route now comes from the shell. `files/claude-route.sh`, sourced from `~/.bashrc`,
exports it, so everything started from an interactive shell inherits it: `claude2` (an
alias that expands to `claude`), scripts, `sh -c` launchers, and `claude -p` workers.
Its `claude()` function strips the route only for launches that start Remote Control:

| Launch | Route |
|---|---|
| `claude`, `claude2`, `claude -p …` | proxy |
| `claude remote-control`, `claude rc`, `claude2 rc` | direct |
| `claude --remote-control [name]`, `--remote-control=name` | direct |

This uses the same rule caveman's own `wrap` door applies (`claudeStartsRemoteControl`
in `@caveman-ai/cli`). Sessions that Remote Control spawns inherit its env, so they run
direct too. Remote Control sessions are not compressed. That cannot be avoided.

The shell function does not cover these launches:
- `caveman claude` (the native door): it launches the binary straight from caveman, so
  the session runs direct. Use `claude`, or `caveman wrap claude` for a routed session.
- A launch with no interactive shell above it (cron, systemd, a GUI launcher that runs
  claude directly): runs direct.
- `sh -c 'claude rc'`: it skips the function and inherits the route, so it is refused.
  Use `env -u ANTHROPIC_BASE_URL claude rc` there.
- `/remote-control` typed inside a routed session: it is still refused. Start the session
  with `claude --remote-control` instead.

Tradeoff of exporting: every process started from an interactive shell sees the route,
not only Claude Code. Any Anthropic SDK client that honours `ANTHROPIC_BASE_URL` sends
its key through the proxy. Under the settings route only Claude sessions and their
children got it. As before, whoever holds 127.0.0.1:8787 sees the bearer tokens. The
systemd unit keeps the proxy restarting so the port is not left free, but on a host
where other local users can bind ports, such as CI runners, a squatter on :8787
would capture them. The fix is planned but not deployed: systemd holds :8787 and the
proxy moves to root-only :787. See ops `docs/caveman-gate-plan.md`.

Because the journaled route is no longer in settings, `caveman status` may report the
claude integration as drifted. `caveman doctor claude` or `caveman enable claude` can put
the route back into settings.json. claude-strapon flags that as
`route in settings.json blocks Remote Control`, and apply moves it again.

## Why both accounts use it

Account 2 does not have its own settings. `~/.claude-account2/{settings.json,
settings.local.json,CLAUDE.md,RTK.md,agents,skills,plugins}` are symlinks into
`~/.claude`, so the router env and hooks written to `~/.claude/settings.json` apply to
both. `caveman-profiles.json` lists only `~/.claude`; account 2 gets the router through
the symlink, not through caveman.

What cannot be symlinked:

| Per-account file | Why | Kept in sync by |
|---|---|---|
| `.claude.json` (MCP servers, login) | holds per-account state | `setup/router/install.sh` copies missing MCP servers |
| `.caveman-active`, `.ponytail-active` | caveman refuses symlinked flags (recorded at 2026-10-10 setup, not re-tested) | rewritten at SessionStart from `~/.config/caveman/config.json` |
| `.credentials.json` | the two subscriptions | never shared |

**Known divergence (2026-10-22):** account 2's `.claude.json` lacks the `caveman` MCP
server (recovery tools for compressed content; the proxy runs with `CAVEMAN_RECOVERY=mcp`).
`setup/router/install.sh` dry run reports it; `--apply` fixes it.

## Pieces on disk

| Path | Role | Repo copy |
|---|---|---|
| `~/.caveman/bin/caveman-proxy` (+ engine, mcp, shrink, mem, browse) | binaries, release `bin-v2.1.0` | `bin-manifest.json` (sha256s) |
| `@caveman-ai/cli@2.1.0` (npm global, Node 22.22.3) | `caveman` CLI, hook adapter | — |
| `~/.claude/settings.json` → `env` + caveman hooks | route + native hooks on every event | `settings.fragment.json` |
| `~/.claude.json` → `mcpServers.caveman` | recovery MCP | `claude.json.fragment.json` |
| `~/.caveman/integrations/claude.json` | what caveman wrote, with sha256 before/after and backups | `caveman-integration.json` |
| `~/.caveman/run/8787.json` | live proxy pid, port, upstreams | `proxy-run.json` (token redacted) |
| `~/.config/caveman/config.json` | default level `ultra` | `caveman-config.json` |
| `~/.caveman/{caveman,ccr}.db`, `proxy.log` | stats / compressed-content store (DBs ~200 MB each, log ~100 KB) | not copied — runtime data |

Hooks: one `caveman-proxy native-hook claude` entry on SessionStart, UserPromptSubmit,
PreToolUse, PostToolUse, PostToolUseFailure, PreCompact, SubagentStart/Stop, Stop,
SessionEnd, plus `caveman shrink-hook` on PreToolUse. The UserPromptSubmit "Likely
implementation path … ccr://" context comes from these. The separate
`~/ops/effort-router.sh` is a prompt classifier, not part of the API route —
see `setup/effort-map.md`.

## Lifecycle — the one way this breaks

The proxy is **not** a systemd unit. It runs detached (ppid 1) with
`CAVEMAN_PROXY_OWNER=wrap`, i.e. started by the caveman CLI on demand. Evidence: boot
2026-10-20 19:16 local, proxy start 22:13 local (`started_at` 18:13Z, TZ +04). Three
`caveman proxy listening` events in `proxy.log` (Oct 11, 17, 20). The Oct 20 start was
followed 12 ms later by a second launch that failed with `bind: address already in use`,
so two launchers raced and the loser exited. That is harmless, but it means more than one
caller starts the proxy.

If nothing listens on :8787, every routed claude and claude2 session fails to reach the
API. Remote Control launches still work, because they run direct. Check and recover:

```bash
ss -tlpn 'sport = :8787'          # expect caveman-proxy
caveman status                    # native integrations: "claude  installed · … provider_proxy, …"
caveman claude                    # owner=wrap suggests this launches it — not yet tested
```

To bypass the proxy for one launch, run `env -u ANTHROPIC_BASE_URL claude …`.
To bypass it for a whole shell, run `unset ANTHROPIC_BASE_URL`.

## Rebuild on a fresh box

1. `npm i -g @caveman-ai/cli@2.1.0` (Node 22), then `caveman claude` once. The integration
   on this box was installed 2026-10-11T16:08Z by pack 2.2.0; it wrote the env route,
   the hooks and the MCP entry shown in `setup/router/`. Compare the result against
   `settings.fragment.json` and `claude.json.fragment.json`.
2. Log in once as `claude2` so `~/.claude-account2/.claude.json` exists.
3. `setup/router/install.sh` (dry run), then `setup/router/install.sh --apply` with no
   claude2 session open.
4. Verify: `setup/router/install.sh` reports all `ok`; `caveman status` shows today's
   tokens rising after one prompt in each account.

You run steps 1–3, not the agent. The `~/.claude` targets are covered by the settings.json
`deny` list and the cmd_guard hook. The `~/.claude-account2/.claude.json` target is rewritten
by any live claude2 session.

## Rollback

`caveman-integration.json` lists the pre-install backups:
`~/.caveman/integrations/backups/claude/2149067c-…/0.bin` (settings.json) and `1.bin`
(`~/.claude.json`). Restoring them would also revert every settings change made since
2026-10-11 — prefer deleting the two `env` keys and the caveman hook entries by hand.
