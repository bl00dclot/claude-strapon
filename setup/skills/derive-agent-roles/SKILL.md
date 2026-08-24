---
name: derive-agent-roles
description: Use when a repo needs Claude Code subagents or an agent team and the
  roles are not yet decided, when a proposed roster reads like job titles
  (developer, QA, marketing, analyst), or when existing .claude/agents definitions
  have grown generic, overlapping, or unused.
---

# Derive Agent Roles

Produce the subagent roster for THIS repo by deriving it from what the repo already
produces and already breaks. The value is the kill list — most candidates must die.
**Never write agent files before the human approves the roster.**

Job titles are not roles. A role exists only if it has one of: a distinct tool
allowlist, a distinct output artifact, or an adversarial stance toward another role.

## Procedure (in order — do not skip)

### 0. Preflight
Confirm a git repo. Check whether `.claude/agents/` already holds definitions — if so
this is a re-derivation, and step 7's kill list already exists. Read it first.

### 1. Inventory artifacts
What the repo already hands between steps. Read, do not guess:

```bash
ls scripts/ bin/ Makefile .github/workflows/ docs/ 2>/dev/null
jq -r '.scripts // {} | keys[]' package.json 2>/dev/null
```

Record each artifact, its producer, and whether that producer is **deterministic**
(a script) or **human**. This column decides most of step 5.

### 2. Inventory gates
Every place work is already checked before it counts: CI workflows, pre-commit hooks,
required review steps, `.claude/settings.json` allow/ask/deny, any tier or permission
doc. The allow/deny lists **are** tool allowlists that already exist — reuse them
verbatim in step 7. Do not invent new ones.

### 3. Inventory failures

```bash
git log --oneline -80 -i --grep='fix\|revert\|hotfix'
```

Plus open issues and post-mortems under `docs/`. Group into recurring classes. A class
that a *different context* would have caught is a candidate role. A one-off is not.

### 4. Generate candidates
One per artifact boundary from step 1. One per failure class from step 3. Name each by
the artifact it produces, never by a profession.

### 5. Kill
Apply in order. First match wins.

| Test | Verdict |
|---|---|
| A deterministic command already produces the artifact | KILL — a script beats an agent: zero tokens, cannot misreport |
| A human gate from step 2 already covers it | KILL — an agent in front of a working gate manufactures the appearance of review |
| Needs send, publish, deploy, or spend authority | KILL — replace with a role that writes the steps and stops |
| Needs write access to guardrail files (settings, sudoers, CI config) | KILL |
| Same tools AND same artifact as a surviving candidate | MERGE |
| No distinct allowlist, no distinct artifact, no adversarial stance | KILL — it is a prompt, not an agent |

Expect to kill most. A roster above ~5 for one repo means step 5 was not applied.

### 6. Curate with the human — REQUIRED GATE
Present two tables:

- **Survivors** — role, tools, model, output artifact, derived-from (which artifact or
  failure class produced it)
- **Kills** — candidate, and the rule that killed it

Ask which to keep, cut, rename, or merge. Wait for an explicit answer. Writing any file
before this point is a violation of the skill, not a shortcut.

### 7. Emit
Per approved role, `.claude/agents/<name>.md`:

```markdown
---
name: <name>
description: <when the lead should delegate here>
tools: <narrowest allowlist from step 2 that still works>
model: <haiku read-only | sonnet default | opus adversarial>
---
<one line: the job>

Your final message is the deliverable. It consists of, in order:
1. <part>
2. <part>
```

State the output as a recipe — what it **is**, in parts, in order. Never as a list of
prohibitions: a "don't" list invites negotiation, a shape does not.

Then one hook, wired to a gate that already exists from step 2:

```bash
#!/bin/bash
# TaskCompleted — not done until the repo's own check passes
<existing verify command> || exit 2
```

Then `.claude/agents/README.md` recording the kill list and the rule that killed each.
The next derivation reads it and does not resurrect them.

## Common mistakes

- Starting from a role taxonomy found online instead of from step 1
- Writing files before the step 6 gate
- Inventing a tool allowlist when `settings.json` already defines one
- Keeping a role because it "feels missing" — no artifact is the evidence it is not
- One role that both produces and checks the same artifact — split it or kill it
- Deriving once and never again; re-run when the failure log in step 3 changes
