#!/usr/bin/env python3
import json, os, sys

settings_path = os.path.expanduser("~/.claude/settings.json")
hook_path = os.path.join(os.path.expanduser("~"), "ops", "effort-router.sh")

# Load existing settings
with open(settings_path) as f:
    settings = json.load(f)

# Ensure hooks key exists
if "hooks" not in settings:
    settings["hooks"] = {}

# Ensure UserPromptSubmit key exists
if "UserPromptSubmit" not in settings["hooks"]:
    settings["hooks"]["UserPromptSubmit"] = []

# Check if effort-router is already registered
already_registered = any(
    any(
        "effort-router" in h.get("command", "")
        for h in entry.get("hooks", [])
    )
    for entry in settings["hooks"]["UserPromptSubmit"]
)

if already_registered:
    print("effort-router already registered in UserPromptSubmit — skipping")
else:
    settings["hooks"]["UserPromptSubmit"].append({
        "hooks": [
            {
                "type": "command",
                "command": hook_path,
                "timeout": 10
            }
        ]
    })
    print(f"Added effort-router hook: {hook_path}")

# Write back
with open(settings_path, "w") as f:
    json.dump(settings, f, indent=2)
print("settings.json written")
