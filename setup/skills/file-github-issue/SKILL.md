---
name: file-github-issue
description: Use when the user wants to file, open, or write up a GitHub issue
  for the current repo — a larger bug, an architecture enhancement, a feature
  improvement, or a general issue. Investigates the code first, drafts a
  grounded issue, and posts via gh only after approval.
---

# File a GitHub Issue

File ONE high-quality, code-grounded GitHub issue against the current repo. The
value is investigation + dedup + a human approval gate — not running `gh`.
**Never create the issue before the human explicitly approves the draft.**

## Procedure (in order — do not skip)

### 0. Preflight
Confirm all of the following. If any fails, STOP and explain; do not proceed.

```bash
git rev-parse --is-inside-work-tree           # inside a git repo?
gh auth status                                # authenticated?
gh repo view --json nameWithOwner,hasIssuesEnabled
```
- A GitHub remote must resolve (the `gh repo view` call succeeds).
- `hasIssuesEnabled` must be `true`. If false, stop — the repo can't take issues.

Echo the resolved `nameWithOwner` so the human sees where the issue will land.

### 1. Grasp the ask
Take the user's description. Ask ONE clarifying question only if intent is
genuinely ambiguous. Do not interrogate.

### 2. Investigate (the quality lever)
Read the relevant code, confirm current behavior, and collect concrete
`file:line` evidence.
- If `.codegraph/` exists at the repo root, use CodeGraph first
  (`codegraph explore "<question>"`, `codegraph node <symbol|file>`).
- Otherwise grep/read.
- Large repo: delegate the investigation to an `Explore` subagent to preserve
  context, then draft from its findings.
Do not write claims you have not grounded in the code.

### 3. Dedup
```bash
gh issue list --search "<keywords>" --state open
```
- Strong duplicate → STOP. Link the existing issue. Do not file.
- Weak/uncertain overlap → surface it in the draft for the human to judge.

### 4. Classify
Pick exactly one type: `bug` | `architecture` | `feature` | `general`.

### 5. Map labels
```bash
gh label list --json name
```
Apply only labels that already exist. Match the type to the closest existing
label by meaning, not exact name (e.g. `feature` → `enhancement`, `bug` →
`bug`). If nothing reasonable matches, apply none. Note any skipped. **Never
create labels.**

### 6. Draft
- **Title:** concise, imperative, optionally type-tagged (`Bug: …`, `Arch: …`).
- **Body:** use the template for the chosen type (below). Omit empty sections.

### 7. Present + approval gate
Show the human: target repo, title, labels to apply, full rendered body. WAIT
for explicit approval. The human may edit. **No GitHub write before an explicit
yes.**

### 8. Post
Feed the body over stdin — never inline `--body` (it corrupts multi-line
markdown):

```bash
gh issue create \
  --title "<title>" \
  --label "<label>" \
  --body-file - <<'EOF'
<rendered body>
EOF
```

### 9. Report
Echo the created issue URL. If `gh` errored, show the raw error verbatim and
claim nothing succeeded.

## Body templates

Omit any section that has no content.

**Bug**
```
## Summary
## Current behavior
<with file:line evidence>
## Expected behavior
## Trigger / reproduction
## Impact
## Proposed direction (optional)
```

**Architecture**
```
## Current design
<with file:line references>
## Limitation / problem
## Proposed change
## Tradeoffs / alternatives considered
## Migration / impact
## Open questions
```

**Feature**
```
## Motivation / value
## Current state
## Proposal
## Acceptance criteria
## Out of scope
```

**General**
```
## Summary
## Details
## Why it matters
## Next step
```

## Error handling

| Condition | Behavior |
|-----------|----------|
| Not inside a git repo (`git rev-parse` fails) | Stop at preflight; name this cause; do not proceed. |
| No GitHub remote resolves (`gh repo view` fails) | Stop at preflight; name this cause; do not proceed. |
| `gh` not authenticated (`gh auth status` fails) | Stop at preflight; name this cause; do not proceed. |
| Repo has Issues disabled (`hasIssuesEnabled` false) | Stop at preflight, explain; do not draft. |
| Strong duplicate found | Stop, link the existing issue, do not file. |
| Label does not exist in repo | Skip it, note which were skipped. Never create. |
| `gh issue create` fails | Surface the raw error verbatim; do not report success. |
| Multi-line body | Always `--body-file -` (stdin heredoc), never inline `--body`. |

## Scope (YAGNI)
One issue per invocation. No batch, no label creation, no editing/closing
existing issues, no cross-repo/fixed-target filing, no provenance footer.
