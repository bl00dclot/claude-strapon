# docs

## Current

| Doc | What it is |
|---|---|
| [../README.md](../README.md) | user guide: quick start, flags, accounts, safety, troubleshooting, undo |
| [components.md](components.md) | exact checks and actions for every component |
| [superpowers/specs/2026-11-10-strapon-rust-rewrite-design.md](superpowers/specs/2026-11-10-strapon-rust-rewrite-design.md) | design of the Rust rewrite: why, goals, decisions |
| [superpowers/plans/2026-11-10-strapon-rust-rewrite.md](superpowers/plans/2026-11-10-strapon-rust-rewrite.md) | the build plan it was implemented from |
| [effort-map.md](effort-map.md) | effort levels the effort-router hook assigns |
| [agents/README.md](agents/README.md) | how the subagent roster in `files/agents/` was derived |

## Historical

These predate claude-strapon. They describe how the stack was first built by hand, from
an ops repo (`~/ops/...` paths, `setup/` scripts that no longer exist), and they are kept
for the reasoning behind each piece. Where they disagree with the code, the code and
[components.md](components.md) are right.

| Doc | Background on |
|---|---|
| [claude-router.md](claude-router.md) | how both accounts reached the API through the caveman proxy (Oct 2026 snapshot; the route has since moved from settings.json to the shell) |
| [activation-guide.md](activation-guide.md) | what scope each piece of the stack lives at |
| [handoff-runbook.md](handoff-runbook.md) | the original by-hand install steps |
| [stack-status.md](stack-status.md) | stack status as verified in mid-2026 |
| [ops-setup-prompt.md](ops-setup-prompt.md) | the prompt the stack was first set up from |
| [model-guide.md](model-guide.md) | which model for which task (mid-2026 model IDs; the `model` component pins `opus[1m]`) |
