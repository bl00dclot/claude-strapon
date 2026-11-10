# Claude Code model guide (subscription — speed is the differentiator, not cost)

> **Historical.** Model IDs are from mid-2026. The `model` component now pins `opus[1m]`; see [components.md](components.md#model). Index: [README.md](README.md).

| Task type                                      | Model                            | Why                           |
|------------------------------------------------|----------------------------------|-------------------------------|
| Renaming, typo fixes, simple file edits        | claude-haiku-4-5-20251001        | Fastest responses             |
| Normal coding, features, bug fixes             | claude-opus-4-8                  | Platform default, high effort |
| Architecture, root cause, complex debugging    | claude-opus-4-8 + /effort xhigh  | Deep thinking                 |
| Large file review, long context work           | claude-sonnet-4-6                | Long context quality          |

## How to switch
- Per session, inside Claude Code:  `/model claude-haiku-4-5-20251001`
- Per shell default (optional):     add `export ANTHROPIC_MODEL="claude-opus-4-8"` to ~/.bashrc
- One-off launch:                   `ANTHROPIC_MODEL="claude-sonnet-4-6" claude`

## Why no CCR (claude-code-router)
CCR sets `ANTHROPIC_AUTH_TOKEN` from its config on `ccr activate`/`ccr code`, which
overrides your subscription OAuth session and causes pay-per-token API billing. For a
pure Max-subscription user this is the wrong tradeoff, so it is excluded from this stack.
