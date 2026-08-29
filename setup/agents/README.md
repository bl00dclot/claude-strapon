# Subagent roster — derived 2026-08-24, re-derived 2026-08-25 and 2026-08-26

Derived with the `derive-agent-roles` skill from this repo's artifacts, gates, and
failure log. Re-run the derivation when the failure classes in `git log` change.

The 2026-08-25 pass ran against 14 new commits, ten of them fixes. It added one role and
widened one; the eleven kills from the first pass all stand. The structural change that
justified the new role: at the first derivation this repo configured one machine, and
six files now shell out over ssh to a second.

## Install

`.claude/**` is denied to the agent (settings.json `deny`, and rule 3 of `CLAUDE.md`),
so these are staged here the same way `setup/skills/` is. A human installs them:

```sh
install -d ~/.claude/agents          # or .claude/agents for repo scope
setup/agents/install.sh          # copies every *.md here except this README
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
| `evidence-auditor` | Read, Grep, Glob, Bash (Tier 0 allow list) | sonnet | per-claim PROVEN / DISPROVEN / UNPROVEN table with commands and output | failure classes F1, F4 |
| `runbook-author` | Read, Grep, Glob, Bash (Tier 0 allow list) | sonnet | `docs/<thing>-runbook.md`, steps with verify + revert | Tier 3 rule in `CLAUDE.md` and `docs/tiers.md` |
| `ssh-hazard-table` | Read, Grep, Glob, Bash (Tier 0 allow list) | sonnet | per-invocation table: remote tool present? stdin free? survives peer death? fresh connection? | failure class F5 |
| `state-author` | Read, Grep, Glob, Edit, Write, Bash | sonnet | files under `state/`/`scripts/` plus the commit, gate output pasted | resurrected 2026-08-26 — see below |

`evidence-auditor` is adversarial toward the lead's own `state/` and `docs/` edits.
`runbook-author` has no mutation tools by construction — it is the "writes the steps and
stops" replacement for anything needing execute authority. `ssh-hazard-table` is
adversarial toward any script the lead writes that crosses a host boundary.

The three do not overlap by input: `evidence-auditor` audits claims in a diff,
`ssh-hazard-table` audits invocations in a script, `runbook-author` produces rather than
checks.

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
- **F4 — the operation returns 0 and achieves nothing (5 commits, added 2026-08-25).**
  `0a2c0cd` reload without re-bind, `4b6dd03` udev change event that applies nothing,
  `d700546` apply silently no-op on rerun, `a9af27b` `network-online.target` active with
  no address on the interface, `8b2d8ff` the inverse — a false FAIL from an empty
  variable. Joint-dominant with F1. Widened `evidence-auditor` rather than adding a role:
  an `effect-prover` would carry the same tools and the same artifact shape.
- **F5 — remote execution assumption (3 commits, added 2026-08-25).** `7c699a9` rsync
  assumed present on the box, `21c46e4` heredoc occupying the stdin sudo needed to
  prompt on, `735bb27` a connection held across the peer's own poweroff. None are
  reachable by shellcheck, which does not model ssh, tty or stdin semantics. Answered by
  `ssh-hazard-table`.

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

### Added 2026-08-25

| Candidate | Rule that killed it |
|---|---|
| nas-baseline-reporter | deterministic command exists — `scripts/nas-discover.sh` |
| nas-verifier | deterministic command exists — `scripts/nas-verify.sh` |
| nas-reconciler | deterministic command exists — `scripts/nas-apply.sh`; also needs Tier 2 authority |
| effect-prover | same tools **and** same artifact as `evidence-auditor` — MERGE. F4 widened that role instead |
| idempotence-tester | needs apply authority to run the thing twice. Belongs as a script that runs `apply.sh` twice and diffs, not as an agent |
| wake-path-validator | deterministic command exists — `scripts/nas-verify.sh` |
| cross-host-config-coupler | no failure class. The `TimeoutIdleSec=600` ↔ `IDLE_TICKS=3` coupling is documented in both unit files and has never broken — no artifact is the evidence it is not a role |
| doc-citation-checker | one-off (a single drifted `file:line` reference, fixed in `53a840c`) and mechanical |

Eight candidates, one survivor. Running total: nineteen candidates, three survivors.

---

# Re-derivation — 2026-08-25

Re-run because the failure log changed (eleven new `fix(nas)` commits). **The roster
did not change.** Three new candidates, three kills.

Step 0 finding worth more than the re-derivation itself: nothing from 2026-08-24 was
ever installed. No `~/.claude/agents/`, no `TaskCompleted` in any settings file, no
`shellcheck`. Two roles written, zero in use. `setup/agents/install.sh` now does the
whole install in one command, since every target is under `~/.claude` and the agent is
denied there.

## New failure classes

- **F4 — the gate itself returns a wrong verdict.** `8b2d8ff` verify reported a false
  FAIL against a correctly armed NIC; `d700546` apply was idempotent-broken and blocked
  its own reruns. Distinct from F1: here a deterministic script, not a human claim, is
  what lies.
- **F5 — a mechanism behaves unlike its name at boot or reload.** `a9af27b`
  network-online.target vacuous so smbd bound loopback-only; `0a2c0cd` smbd reload never
  re-binds sockets; `4b6dd03` `.link` needs an add event, not a change event;
  `735bb27` sleep-nas froze the terminal.
- **F6 — blanket staging sweeps unrelated files.** `7c699a9` fixed exactly this, then
  `4c38283` swept `state/otel/*` into a NAS commit anyway. A prompt rule did not survive
  across sessions. Recurrence after a fix is what promotes it to a class.

## Kill list — 2026-08-25

| Candidate | Rule that killed it |
|---|---|
| gate-adversary (F4) | cannot produce its artifact within its tool ceiling — exposing a false verdict means inducing the state that triggers it, which is mutation, not Tier 0. The repo's own answer is a test script (`scripts/tests/test-nas-idle.sh` is the precedent), and a deterministic command beats an agent |
| boot-lifecycle-auditor (F5) | its proof requires a reboot — Tier 2+, and no read-only command substitutes. Residue folds into `evidence-auditor`, whose UNPROVEN verdict is the correct output for a claim nothing can currently measure |
| commit-scope-checker (F6) | deterministic command exists once written — a script beats an agent. Emitted as `scripts/git-add-scope-guard.sh`, a PreToolUse hook that refuses `git add -A` / `git add .` and names the two commits that justify it |

## Hooks

| Hook | Script | Gate it reuses |
|---|---|---|
| `TaskCompleted` | `scripts/agent-taskcompleted.sh` | `shellcheck` + `scripts/verify.sh` |
| `PreToolUse` (Bash) | `scripts/git-add-scope-guard.sh` | F6; no prior gate existed |


---

# Re-derivation — 2026-08-26

Run with the argument "to plan, research and write a sincere plan". Five commits since
`fbccaa2`. **One kill resurrected, six candidates killed, roster now four.**

## Step 0 — what the last two passes produced, measured

```console
$ ls ~/.claude/agents/
ls: cannot access '$HOME/.claude/agents/': No such file or directory
$ grep -c TaskCompleted .claude/settings.json
1
$ grep -n 'PreToolUse|git-add-scope' .claude/settings.json .claude/settings.local.json
0 matches
$ command -v shellcheck
/usr/bin/shellcheck
```

Fixed since the last pass: `shellcheck` installed, `TaskCompleted` wired (`ac96998`).
Still inert: all three roles, and `scripts/git-add-scope-guard.sh` — written, widened in
`871b956` to also refuse `git add <dir>`, and never wired as `PreToolUse`. A guard
nothing invokes is not a gate.

`install.sh` carried its own instance of the same disease: it named
`evidence-auditor.md` and `runbook-author.md` literally, so `ssh-hazard-table` — the
whole product of the 2026-08-25 pass — was staged and could never have been installed
even by someone who ran the installer. It now discovers role files rather than listing
them.

## New failure log

| Commit | Class |
|---|---|
| `c72c66f` verify's port-22 check reported FAIL on a host with no ssh server | F4 recurrence |
| `76e478d` the shellcheck gate was green about the wrong files | F4 recurrence |
| `871b956` the guard refused `-A` and `.` but not `<dir>` | F6 recurrence |
| `b152195` `tar -C / -x` as root chmod'd `/`, `/etc`, `/usr` to 0775 | F6 in a new medium |
| `ac96998` wire the hook | install gap, not a failure |

`b152195` was filed as a candidate class of its own — unstated blast radius — and is
not one. It is `4c38283` (`git add` staging more than named) in a different tool: a
bulk operation whose target set is wider than the one named. F6 widens; F7 is not born.
Note the discovery path, which is the durable lesson: a group-writable `/` was found by
an unrelated tool refusing to trust it, never by anything in this repo's verify path.

## The resurrection

`state-file-author` was killed on 2026-08-24 by *no distinct allow list, artifact, or
stance against the lead — it is a prompt*. That verdict was correct while the lead did
all the writing. Three facts have changed it:

- Every surviving role is Tier 0 read-only. `Edit(state/**)` is the only mutation right
  in `settings.json`'s allow list, so a role that holds it is maximally distinct — no
  other role can produce its artifact.
- Its artifact is a reviewable commit, not a verdict. `runbook-author` produces text for
  a human to run; this produces files the machine will run.
- It completes the pair the other three exist to be. `evidence-auditor` and
  `ssh-hazard-table` were both written adversarial toward *the lead's* output; that
  output now has an author with its own tool ceiling, which is the "split the role that
  both produces and checks" rule satisfied rather than violated.

Renamed `state-author`, matching `runbook-author`. Its ceiling is the commit:
`apply.sh` and `nas-apply.sh` are absent from its shape by design, so reconciliation
stays with the human reading the diff.

## Kill list — 2026-08-26

| Candidate | Rule that killed it |
|---|---|
| `plan-author` | MERGE — same tools **and** same artifact as `runbook-author`. A plan whose steps carry verify and revert *is* the runbook shape |
| `research-scout` | A command already produces the artifact — the `deep-research` workflow, which returned a 21-source verified report on 2026-08-25 |
| `premise-prover` | MERGE — same tools and artifact as `evidence-auditor`; UNPROVEN is already the verdict for a claim nothing can yet measure |
| `plan-completeness-critic` | No distinct allow list, and its artifact overlaps `evidence-auditor`'s UNPROVEN column. Evidence is one commit (`d93c07a`) — one-off, not a class |
| `blast-radius-auditor` | A deterministic command exists once written, and it is already written: `nas-build-runbook.md:406` holds the `find / -xdev -maxdepth 2 -type d -perm -g+w` check, unwired |
| `install-drift-checker` | Deterministic — an assertion in `verify.sh`, not an agent |

Six candidates, zero survivors. Running total across three passes: **28 candidates,
4 roles**, one of them resurrected.

## Hooks

No new hook. `state-author`'s gate is the one that already exists: `TaskCompleted` runs
`shellcheck` over `state/*/usr/local/sbin/`, `state/*/etc/profile.d/` and `scripts/*.sh`,
then `./scripts/verify.sh`. Those are exactly the paths it writes.

Two gates remain unwired and are not roles: `git-add-scope-guard.sh` as `PreToolUse`
(the installer does it; nobody has run the installer), and the group-writable-`/` check
in `verify.sh`.

---

# Re-derivation — 2026-08-28

Fourth pass, and the first run against `ops` itself. The 2026-08-26 entry closed
with "No second repo has been derived. `ops` itself was analysed by hand and
yields three roles, none written." This writes two of them.

Measured against `main` at `f38a802`, clean, 11 ahead of origin. Inputs: 19
scripts, 5 Tier 2 wrappers, no CI, 15 docs, `.claude/settings.json` with 24
allow / 7 ask / 13 deny, one wired `TaskCompleted` hook, and **31 fix commits
of 71 total — a 44% fix rate.**

## New failure classes

**A check reported a result that was not true — 4 commits.** The strongest class
in the repo and the one no deterministic command covers, because the script that
would catch it *is* the defect:

- `c72c66f` port-22 probe FAILed against a host legitimately running no sshd
- `8b2d8ff` false FAIL against a correctly armed NIC
- `1a98b53` passed while apply had left targets group-writable
- `76e478d` the shellcheck gate was green while checking the wrong files

`scripts/agent-taskcompleted.sh` ends with `./scripts/verify.sh || exit 2`. The
gate that decides whether a subagent is finished trusts a script that has been
wrong three times, and nothing validates the validator.

**An apply that is correct once and destructive twice — 5 commits.**
`nas-apply.sh` and `nas-apply-remote.sh` carry 13 fix-touches between them, the
largest cluster here: `d700546`, `b152195` (tar restored umask-002 modes and
chmod'd `/` to 0775 on rerun), `7c699a9`, `21c46e4`, `1ba4c4f`.

**A wrapped binary answered instead of the real one — 2 instances, 2026-08-27.**
A `find / -xdev` sweep returned zero rows where `/usr/bin/find` returned one;
`ls -l` dropped the owner column an ownership audit needed. Both read as
answers, not errors. This produced no new role — it is an amendment to
`evidence-auditor`, whose whole value is not being fooled and which held the
same blind spot.

## Kept

| Role | Artifact | Derived from |
|---|---|---|
| `check-auditor` | Per-assertion verdict: can this check report something untrue? | the 4 false-verdict commits, plus the gate's own dependency on `verify.sh` |
| `idempotence-auditor` | Per-step second-run verdict for mutating scripts | the 5 rerun defects; 13 fix-touches on `nas-apply*.sh` |

`check-auditor` is `opus` and adversarial toward `state-author` **and** toward
the repo's own TaskCompleted gate — the first role in this roster pointed at a
gate rather than at a diff.

## Kill list — 2026-08-28

| Candidate | Rule |
|---|---|
| `discover-agent`, `plan-agent`, `verify-runner` | deterministic producer — `discover.sh`, `plan.sh`, `verify.sh` |
| `shellcheck-agent`, `test-runner` | existing gate — `agent-taskcompleted.sh` runs shellcheck over `scripts/*.sh` and `state/`; `test-teleport.sh`, `test-nas-idle.sh` exist |
| `state-author`, `runbook-author`, `ssh-hazard-table`, `evidence-auditor` | duplicate — already in this roster |
| `tier2-wrapper-author`, `settings-editor` | guardrail write — `deny Edit(bin/**)`, `Edit(//usr/local/sbin/claude-*)`, `Edit(.claude/**)` |
| `transfer-operator` | needs execute authority on a destructive path (`nas-media.sh --reclaim`) |
| `catalog-builder` | deterministic producer — `docs/ux-plan.md` phase 1 is a script |
| `convention-auditor` | one-off — n=2 (a missing `flock`, a missing local manifest), both in one session by one author; folds into `check-auditor` |
| `doc-drift` | no failure class — no commit in 71 fixes a stale doc |
| `reviewer` | no distinct allowlist, artifact or stance |

## Hooks

None added. `TaskCompleted` → `scripts/agent-taskcompleted.sh` already runs
shellcheck plus `verify.sh` and is already wired in `settings.json`. Adding a
second gate in front of a working one is the theatre step 5 exists to kill —
and `check-auditor` exists precisely because that gate is not always working.

## Known ceiling

Same as every prior pass, undischarged: **these roles have never run.** Their
output contracts are unproven against a real diff. The cheapest test for
`check-auditor` is `c72c66f` — check out its parent, point the role at
`scripts/verify.sh`, and see whether it finds the port-22 probe that FAILs on a
host with no sshd. One known-bad input, one known-good answer.
