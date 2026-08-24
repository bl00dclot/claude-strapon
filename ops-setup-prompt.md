# Claude Code Optimization Stack — Setup Prompt
# Paste this entire prompt into Claude Code running inside ~/ops

---

You are setting up a permanent token-optimization stack for Claude Code on this Linux machine.
Work entirely inside `~/ops/`. Create a `setup/` subdirectory to store logs and configs you generate.
Run every install and verification step yourself — do not ask me to run anything manually.
After each major step, print a one-line status: ✓ done or ✗ failed + reason.

## Goals

1. Install **RTK** (Rust Token Killer) and wire its PreToolUse hook into Claude Code
2. Install **CodeGraph** and register it as a global MCP server in Claude Code
3. Install **claude-code-router** and configure smart Anthropic-only model routing
4. Install the **Caveman** skill globally so it auto-activates every session
5. Install the **effort-router** hook that auto-classifies every prompt and sets effort level
6. Write a summary file `~/ops/setup/stack-status.md` confirming what is running

---

## Step 1 — RTK

Install RTK via the official install script:

```
curl -fsSL https://raw.githubusercontent.com/rtk-ai/rtk/refs/heads/master/install.sh | sh
```

After install:
- Run `rtk --version` to confirm it is present
- Run `rtk init -g` to register the PreToolUse hook in Claude Code's settings.json
  - When prompted whether to patch settings.json, answer y
- Run `rtk gain` — it will show "no data yet", that is expected
- Confirm the hook appears in `~/.claude/settings.json` under `hooks.PreToolUse`

If RTK binary lands somewhere not on PATH, add the directory to PATH in `~/.bashrc` and source it before continuing.

---

## Step 2 — CodeGraph

Verify Node version is 22.x (required — Node 25 has a known crash bug with CodeGraph):

```
node --version
```

Install globally:

```
npm install -g @colbymchenry/codegraph
```

After install:
- Run `codegraph --version` to confirm
- Run `codegraph install` — this auto-detects Claude Code and writes the MCP server entry into
  `~/.claude/mcp_servers.json`. When asked about auto-allow, answer yes.
- Run `codegraph status` and confirm the output shows `Backend: native` not `Backend: wasm`.
  If it shows wasm, uninstall and reinstall from scratch.
- Do NOT run `codegraph init` yet — that is per-project and will be done manually per repo later.

---

## Step 3 — claude-code-router (Anthropic-only smart routing)

Install globally:

```
npm install -g @musistudio/claude-code-router
```

After install confirm with `ccr --version`.

Create the config file at `~/.claude-code-router/config.json` with this exact content:

```json
{
  "HOST": "127.0.0.1",
  "PORT": 3456,
  "Providers": [
    {
      "name": "anthropic",
      "api_base_url": "https://api.anthropic.com/v1/chat/completions",
      "api_key": "ANTHROPIC_API_KEY_PLACEHOLDER",
      "models": [
        "claude-haiku-4-5-20251001",
        "claude-sonnet-4-6",
        "claude-opus-4-6"
      ]
    }
  ],
  "Router": {
    "default": "anthropic,claude-sonnet-4-6",
    "background": "anthropic,claude-haiku-4-5-20251001",
    "longContext": "anthropic,claude-sonnet-4-6",
    "reasoning": "anthropic,claude-opus-4-6"
  }
}
```

Routing logic:
- `default`     → Sonnet 4.6  — all normal coding tasks
- `background`  → Haiku 4.5   — summaries, doc rewrites, grep/search sub-tasks
- `longContext` → Sonnet 4.6  — large file work (Haiku degrades on long context)
- `reasoning`   → Opus 4.6    — architecture decisions, deep debugging

Note for stack-status.md:
```
MANUAL STEP REQUIRED — CCR API KEY
Edit ~/.claude-code-router/config.json
Replace ANTHROPIC_API_KEY_PLACEHOLDER with your actual sk-ant-... key.
Then run: ccr start
Launch Claude Code through the router with: ccr code  (not: claude)
```

---

## Step 4 — Caveman (global auto-activation)

Clone the Caveman skill into the Claude Code global skills directory:

```
mkdir -p ~/.claude/skills
cd ~/.claude/skills
git clone https://github.com/JuliusBrussee/caveman.git
```

Then add Caveman auto-activation to the global CLAUDE.md at `~/.claude/CLAUDE.md`.
If the file does not exist, create it. Append (do not overwrite) the following block:

```markdown
## Output compression — Caveman

@~/.claude/skills/caveman/caveman.md

Always respond in caveman style: terse, no filler, no hedging, no preamble.
Drop pleasantries. Lead with the answer. Code speaks. Words cost tokens.
```

Confirm with:
```
grep -n "caveman" ~/.claude/CLAUDE.md
```

---

## Step 5 — Effort router (auto effort-level classification)

This is a UserPromptSubmit hook. It fires before Claude sees each prompt, classifies the
task using keyword and pattern rules, then injects an effort directive so the right thinking
budget is used automatically — no manual /effort typing needed.

### 5a — Create the classifier script

Create the file `~/ops/effort-router.sh` with this exact content, then `chmod +x` it:

```bash
#!/usr/bin/env bash
# effort-router.sh — auto-classify prompt and inject effort directive
# Receives Claude Code hook JSON on stdin via UserPromptSubmit event.
# Outputs JSON with injected effort level on stdout.
# Effort levels: low | medium | high | xhigh
# (max is reserved for explicit /effort max — do not auto-assign it)

set -euo pipefail

input=$(cat)

# Extract the raw prompt text from the hook payload
prompt=$(echo "$input" | python3 -c "
import json, sys
data = json.load(sys.stdin)
# UserPromptSubmit payload: data['prompt'] contains the user message
print(data.get('prompt', ''))
" 2>/dev/null || echo "")

prompt_lower=$(echo "$prompt" | tr '[:upper:]' '[:lower:]')

# ── Classifier ────────────────────────────────────────────────────────────────
# Rules are evaluated top-down; first match wins.

effort="medium"  # sensible default — handles the majority of everyday tasks

# LOW — mechanical, no reasoning needed
# Renames, typo fixes, value substitutions, formatting, status checks
if echo "$prompt_lower" | grep -qE \
  'rename|typo|whitespace|indent|format|prettier|lint fix|variable name|constant|placeholder|replace .* with|what is|whats|check if|does .* exist|list (files|dirs|packages)|show me|print|echo|git (status|log|diff|add|push|pull)|ls |cat |pwd|which |version'; then
  effort="low"

# XHIGH — complex multi-file reasoning, architecture, security, deep bugs
# These warrant extended thinking but not the full max budget
elif echo "$prompt_lower" | grep -qE \
  'architect|refactor (entire|whole|all)|design (pattern|system|schema)|race condition|deadlock|memory leak|security (vuln|audit|issue|review)|auth (flow|system|logic)|migration|database schema|api design|breaking change|performance (issue|regression|bottleneck)|why (is|does|isn.t|doesn.t)|root cause|investigate|explain (how|why)|trade.?off|compare (approach|option|solution)'; then
  effort="xhigh"

# HIGH — multi-step implementation, non-trivial bugs, new features
elif echo "$prompt_lower" | grep -qE \
  'implement|build|create (a |the |new )|add (feature|support|endpoint|route|component|function|class)|fix (bug|issue|error|crash|test)|debug|failing test|write (test|spec|function|class|module)|integrate|set up|configure|deploy|ci.?cd|dockerfile|parse|serialize|async|concurrent|hook|middleware|plugin'; then
  effort="high"
fi

# ── Output ────────────────────────────────────────────────────────────────────
# UserPromptSubmit hook output format:
# Return the original input with an added system injection that sets effort.
# The hookSpecificOutput.suppressOutput key hides this from the transcript.

python3 -c "
import json, sys

data = json.loads('''$(echo "$input" | python3 -c "import json,sys; print(json.dumps(sys.stdin.read()))")''')
effort = '$effort'

# Inject effort instruction as a system-level prefix Claude will obey
injection = f'<effort>{effort}</effort>'

existing = data.get('prompt', '')
data['prompt'] = injection + '\n' + existing

out = {
    'hookSpecificOutput': {
        'hookOutputToClaudeCode': json.dumps(data)
    }
}
print(json.dumps(out))
"
```

If the python3 JSON injection block causes any shell quoting issues, rewrite the output
section as a simpler jq command:

```bash
# Alternative output section using jq (use this if python3 block fails):
effort_tag="<effort>${effort}</effort>"
echo "$input" | jq --arg tag "$effort_tag" \
  '.prompt = ($tag + "\n" + .prompt)' | \
  jq -c '{hookSpecificOutput: {hookOutputToClaudeCode: (. | tostring)}}'
```

Test the script manually before wiring it:
```
echo '{"prompt":"rename the variable foo to bar"}' | ~/ops/effort-router.sh
# expected: effort=low in the injected output

echo '{"prompt":"investigate the race condition in the websocket handler"}' | ~/ops/effort-router.sh
# expected: effort=xhigh
```

### 5b — Register the hook in Claude Code settings

Open `~/.claude/settings.json`. Add the UserPromptSubmit hook entry.
If a `hooks` key already exists, merge carefully — do not overwrite existing hooks.

The entry to add:

```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "~/ops/effort-router.sh"
          }
        ]
      }
    ]
  }
}
```

After writing, validate the JSON is still valid:
```
python3 -m json.tool ~/.claude/settings.json > /dev/null && echo "valid JSON" || echo "INVALID — fix before continuing"
```

### 5c — Also write the effort map to ~/ops/setup/effort-map.md for reference

```markdown
# Effort routing map

| Effort | Thinking budget | Triggers (examples) |
|--------|----------------|---------------------|
| low    | Minimal — pattern recall only | rename, typo, git status, cat, ls, what is, version |
| medium | Default — handles most tasks | anything not matched by other rules |
| high   | Extended — multi-step impl   | implement, build, fix bug, write test, add feature, configure |
| xhigh  | Deep — architecture & hard bugs | architect, refactor entire, race condition, security audit, why does X, root cause |

Note: `max` is never auto-assigned. Use `/effort max` manually for the hardest
one-off problems (e.g. debugging a heisenbug you've spent days on).
The classifier is additive — tune ~/ops/effort-router.sh patterns any time.
```

---

## Step 6 — Verification pass

Run each of these and report all output:

```
rtk --version
codegraph --version
ccr --version
grep -c "rtk" ~/.claude/settings.json
grep -c "effort-router" ~/.claude/settings.json
grep -c "codegraph" ~/.claude/mcp_servers.json 2>/dev/null || grep -rn "codegraph" ~/.claude/ | head -3
grep -n "caveman" ~/.claude/CLAUDE.md
echo '{"prompt":"fix the failing auth test"}' | ~/ops/effort-router.sh
echo '{"prompt":"ls"}' | ~/ops/effort-router.sh
```

---

## Step 7 — Write ~/ops/setup/stack-status.md

Create this file populated from actual command output:

```markdown
# Claude Code optimization stack — status

Generated: <date>

## RTK
- Version: <output>
- Hook registered: <yes/no>
- Savings tracking: accumulates with usage — check with: rtk gain

## CodeGraph
- Version: <output>
- MCP registered: <yes/no>
- Backend: <native — must NOT be wasm>
- Per-project init: cd into repo, run: codegraph init

## claude-code-router
- Version: <output>
- Config: ~/.claude-code-router/config.json
- ⚠ NEEDS API KEY: replace ANTHROPIC_API_KEY_PLACEHOLDER in config
- Launch: ccr code  (replaces: claude)
- default    → claude-sonnet-4-6
- background → claude-haiku-4-5-20251001
- reasoning  → claude-opus-4-6

## Caveman
- Location: ~/.claude/skills/caveman/
- Activation: global CLAUDE.md @import — always-on

## Effort router
- Script: ~/ops/effort-router.sh
- Hook: UserPromptSubmit in ~/.claude/settings.json
- Levels assigned automatically: low / medium / high / xhigh
- Reference map: ~/ops/setup/effort-map.md
- Override any time with: /effort <level>

## Full session workflow
1. cd into project
2. codegraph init          (first time only per repo)
3. ccr code                (launches Claude Code through router)
4. Type prompts normally — effort level is set automatically
5. Override with /effort max for the hardest problems
6. rtk gain                (check accumulated token savings)
```

---

## Failure handling

- PATH issues: check ~/.local/bin and ~/.npm-global/bin; add to ~/.bashrc and re-source
- CodeGraph wasm: npm uninstall -g @colbymchenry/codegraph then reinstall
- effort-router.sh JSON errors: use the jq fallback output section
- settings.json merge conflicts: show me the current file content and I will write the merged version
- Log all errors to: ~/ops/setup/install-errors.log

Do not silently skip a failed step. Report it explicitly.
