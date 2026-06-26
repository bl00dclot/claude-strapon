---
name: switch-context
description: Use when a work session is ending or about to roll over — context
  window filling toward compaction/clear, switching to a different task, or the
  user says wrap up, hand off, checkpoint, save state, or switch context.
  Surfaces and routes everything unsaved before it is lost.
---

# Switch Context

Deliberate session handoff before context rolls over. Auto-compaction is lossy
and drops early instructions; a manual checkpoint is not. Goal: **leave nothing
that lives only in context** — route every loose thread to its durable sink,
then emit one paste-ready resume prompt.

**A handoff's job is to STOP cleanly, not to do more.** Do NOT start substantial
new work to "finish" it as context dies — capture it and hand it off. Cramming
new work into a dying context is the classic failure this skill exists to prevent.

## Procedure (in order — do not skip)

### 1. Snapshot live state — read-only
Never hand off from memory. Read actual state first:
```bash
git status && git diff --stat
```
Also check: background jobs still running, half-applied changes, open todos.
Re-read the top of any work-in-progress file so the summary describes live state,
not your recollection of it.

### 2. Stabilize the tree (the bounded "finish")
Make the session one reviewable diff (repo commit discipline). But carefully:
- Classify each loose path first — **commit** (real work) / **gitignore or
  leave** (caches, backups) / **delete** (scratch). Do not reflexively
  `git add -A`.
- Scan the staged diff for secrets/tokens before committing.
- On the main branch with non-trivial work → branch first. Do not push unless
  asked.
- Only *complete* in-flight work that is near-done AND finishable in a few safe
  steps. Anything larger is forward work → step 3, not now.
- Note anything left dirty on purpose so the next window does not panic about it.

### 3. File forward work as issues
Every distinct bug, enhancement, or feature surfaced this session that outlives
the current task → a GitHub issue. **REQUIRED SUB-SKILL: use file-github-issue**
— it investigates, dedups, gates on human approval, and posts. Do not reimplement
`gh`.
List the candidates first; file the ones the human approves. Reference the filed
`#numbers` in the kick-off prompt.

### 4. Save durable memory
Persist non-obvious facts that outlive this session AND every session: decisions
made, dead-ends ruled out, constraints discovered, user feedback. Follow the
project's memory rules (one fact per file + a MEMORY.md pointer; update an
existing file rather than duplicate). Do NOT save what git, issues, or the repo
already record — that is tracking, not memory.

### 5. Emit the kick-off prompt
One paste-ready block for the next window, matching the `the next planned`
resume format:
```
Committed: <sha + one-line>   |   Left dirty on purpose: <files + why>
Filed: <#issues>              |   Memory: <notes added>

Last completed: …
In progress:   …
Next:          …
Blockers:      …

Kick-off prompt:
<imperative, spec-first prompt naming the exact files/commands to resume on>
```

## Sink routing — where each thread goes (no overlap)

| Thread | Sink |
|---|---|
| Uncommitted real work | git commit (step 2) |
| Cache / backup / scratch file | gitignore or delete — never commit |
| Bug / enhancement / feature for later | GitHub issue via file-github-issue |
| Durable decision / constraint / feedback | memory file + MEMORY.md pointer |
| Current task's next concrete step | kick-off prompt only |
| Live planning doc (only if the project keeps one) | update it; else skip — do not invent one |

## Red flags — STOP

- About to start a big new change "to finish before the handoff" → don't. File
  it (step 3).
- `git add -A` without reading the diff → you will commit a cache or a secret.
- Summarizing from memory instead of `git status` → stale handoff.
- The same fact written to an issue AND memory AND the kick-off → pick one sink.

## Scope (YAGNI)
No standalone `HANDOFF.md` — git + issues + memory + the kick-off already carry
state. No auto-push, no auto-filing without approval, no mandated planning-doc
update.
