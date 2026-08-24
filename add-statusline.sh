#!/usr/bin/env bash
# One-shot: add caveman statusline badge to ~/.claude/settings.json
set -euo pipefail

SETTINGS="$HOME/.claude/settings.json"
STATUSLINE_SCRIPT="$(ls -1 $HOME/.claude/plugins/cache/caveman/caveman/*/src/hooks/caveman-statusline.sh 2>/dev/null | head -1)"

if [[ -z "$STATUSLINE_SCRIPT" ]]; then
  echo "ERROR: caveman-statusline.sh not found under plugins cache" >&2
  exit 1
fi

cp "$SETTINGS" "$SETTINGS.bak-statusline"

python3 - "$SETTINGS" "$STATUSLINE_SCRIPT" <<'PY'
import json, sys
settings_path, script_path = sys.argv[1], sys.argv[2]
with open(settings_path) as f:
    data = json.load(f)
data["statusLine"] = {
    "type": "command",
    "command": f'bash "{script_path}"',
}
with open(settings_path, "w") as f:
    json.dump(data, f, indent=2)
    f.write("\n")
print("statusLine added.")
print("  script :", script_path)
print("  backup :", settings_path + ".bak-statusline")
PY
