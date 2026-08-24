# Subagent roster — derived 2026-08-24

Derived with the `derive-agent-roles` skill from this repo's artifacts, gates, and
failure log. Re-run the derivation when the failure classes in `git log` change.

## Install

`.claude/**` is denied to the agent (settings.json `deny`, and rule 3 of `CLAUDE.md`),
so these are staged here the same way `setup/skills/` is. A human installs them:

```sh
install -d ~/.claude/agents          # or .claude/agents for repo scope
install -m0644 setup/agents/evidence-auditor.md setup/agents/runbook-author.md \
  ~/.claude/agents/
```

The hook is `scripts/agent-taskcompleted.sh`. Wiring it needs an edit to
`.claude/settings.json`, which the agent also cannot make:

```json
"hooks": {
  "TaskCompleted": [
    { "hooks": [ { "type": "command", "command": "./scripts/agent-taskcompleted.sh" } ] }
  ]
}
```

It shells out to `shellcheck`, which is not installed on this box yet
(`apt install shellcheck`). Without it the hook still runs `verify.sh` and warns.

## The roster

| Role | Tools | Model | Output artifact | Derived from |
|---|---|---|---|---|
| `evidence-auditor` | Read, Grep, Glob, Bash (Tier 0 allow list) | sonnet | per-claim PROVEN / DISPROVEN / UNPROVEN table with commands and output | failure class F1 |
| `runbook-author` | Read, Grep, Glob, Bash (Tier 0 allow list) | sonnet | `docs/<thing>-runbook.md`, steps with verify + revert | Tier 3 rule in `CLAUDE.md` and `docs/tiers.md` |

`evidence-auditor` is adversarial toward the lead's own `state/` and `docs/` edits.
`runbook-author` has no mutation tools by construction — it is the "writes the steps and
stops" replacement for anything needing execute authority.

## Failure classes the roster answers

- **F1 — unmeasured assertion (7 commits).** `1feb621` LAN values, `53707f2` NIC/MAC/
  chipset, `dbeb7cb` router model, `4f319ad` no ethernet port, `7f7206f` backup
  direction, `a6ac565` fabricated SMART, `c864d19` RTK binary name. State and docs
  written from assumption, corrected later by measurement. Dominant class.
- **F2 — script dies in a non-interactive shell (`1ba4c4f`).** Five `#!/bin/sh` scripts
  under `state/*/usr/local/sbin/` run as root under systemd timers, checked by nothing.
  Answered by the hook, not by a role.
- **F3 — unrecoverable step before its proof (`d93c07a`).** One commit. Folded into
  `runbook-author`'s output shape rather than given its own role.

## Kill list

Do not resurrect these without a new failure class to justify them.

| Candidate | Rule that killed it |
|---|---|
| baseline-reporter | deterministic command exists — `scripts/discover.sh` |
| plan-differ | deterministic command exists — `scripts/plan.sh` |
| reconciler | deterministic command exists — `scripts/apply.sh`; also needs Tier 2 authority |
| verifier | deterministic command exists — `scripts/verify.sh`, `scripts/nas-verify.sh` |
| shell-script-hardener | deterministic command exists — `shellcheck`, wired into the hook. Its residue (a script assuming a tty for sudo, which shellcheck does not model) folds into `evidence-auditor` |
| firewall-lockout-checker | existing gate covers it — `claude-fw-apply`'s 60s auto-revert plus `nft -c -f` in `plan.sh`. An agent in front of a working gate manufactures the appearance of review |
| tier-classifier | existing gate covers it — the `ask` list in settings.json and `sudoers.d/claude-ops` are the enforcement. The verdict is one word: no distinct artifact |
| secrets-scanner | existing gate covers it — `deny` already blocks `.env`, `~/.ssh`, `~/.aws`. No distinct artifact against `evidence-auditor` |
| state-file-author | no distinct allow list, artifact, or stance against the lead — it is a prompt |
| commit-writer | no distinct artifact |
| irreversibility-sequencer | one-off, not a recurring class (F3 is a single commit). Its concern lives in `runbook-author`'s step shape |

Eleven candidates, two survivors.
