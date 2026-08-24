---
name: issue-board
description: Use when the user wants to see the open GitHub issues for a repo — "list the issues", "what's open", "gh issue list", "show me the backlog", "group the issues" — and wants them organised rather than dumped as a flat list. Fetches once with gh, sorts every issue into exactly one work-state block, and reconciles the count.
---

# Issue Board

Turn `gh issue list` into a board: every open issue in exactly one block,
ordered by what a human would pick up next. One fetch, no per-issue calls.

**The value is the sort, not the fetch.** A flat list is already available with
one command — do not just reprint it.

## 1. Fetch (two calls, that's all)

```bash
gh issue list --state open --limit 500 \
  --json number,title,labels,milestone,assignees,updatedAt,createdAt
gh pr list --state open --limit 200 --json number,title,body,headRefName,isDraft
```

- Second call is only to find which issues are already in flight: an open PR
  whose body says `Closes #N` / `Fixes #N` / `Resolves #N`, or whose branch or
  title cites `#N`.
- No GitHub remote, or `gh auth status` fails → say so and stop. Do not invent
  a list from docs.
- Args the user passes (`--label x`, `--assignee @me`, `--repo o/r`, a search
  term) go straight through to the first call.

Record the total: `gh issue list --state open --limit 500 --json number -q length`.

## 2. Sort into blocks — first match wins

Walk the ladder top to bottom. An issue lands in the **first** block it
matches and appears nowhere else.

1. **Blocked / needs a decision** — labels `blocked`, `needs-decision`,
   `question`; or the title/body says it waits on another issue or an external
   answer. These gate other work, so they lead.
2. **In flight** — an open non-draft PR closes it, or it has an assignee.
   Cite the PR: `#986 … (PR #991)`.
3. **Next up** — milestone with the nearest due date, or labels `next`,
   `ready`, `planned`, `priority`, `p0`, `p1`. Nearest milestone first.
4. **Themed blocks** — everything else, clustered by *subsystem*, not by
   label. Derive the theme from the title prefix (`[hq]`, `Arch:`, `Treasury:`)
   and recurring nouns (payments, offers, CI, docs, i18n). Name each block for
   the subsystem, e.g. `### Payments & settlement`. Aim for 3–7 blocks of 3+
   issues; anything that clusters with nothing goes to **Loose ends**.
   Only fall back to label-family blocks (`bug` / `tech-debt` / `enhancement`)
   when titles carry no subsystem signal at all.
5. **Cold** — label `backlog`, or untouched for >90 days. Collapse to a count
   plus numbers unless the user asked for the whole thing.

Inside every block: most recently updated first.

## 3. Render

```
## Blocked (2)
- #979 A POI-blocked Railgun note holds stock reserved forever — tech-debt · 4d
- …

## In flight (1)
- #983 hq brand site serves invented company history — PR #985 · 2h
```

- One line per issue: `#N` · title (trim to ~80 chars, keep the meaning) ·
  labels that add information · age since last update.
- Drop labels that every issue in the block shares — say it in the heading.
- Skip empty blocks silently. Never print a block with a "none" placeholder.
- Close with the reconciliation line: `74 open · 2 blocked · 1 in flight · 5
  next · 52 themed · 14 cold`. **The block counts must sum to the total from
  step 1.** If they don't, you lost or duplicated an issue — fix it before
  answering, and never round or hand-wave the number.

## 4. Then shut up

No recommendations, no "I suggest starting with…", unless asked. The board is
the deliverable. If the user wants a pick, they will ask for one.

## Notes

- 74 issues is a wall of text — the blocks are what make it readable, so never
  emit more than ~40 lines without collapsing the Cold block to counts.
- Milestones and labels differ per repo. Read the real vocabulary with
  `gh label list --json name -q '.[].name'` before assuming `next` or `p1`
  exists; match by meaning, not by exact name.
- Multi-label issues are common. The ladder, not the label set, decides
  placement — resist listing an issue twice because it is both a bug and
  blocked.
