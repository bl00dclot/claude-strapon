# model: default model in settings.json. Per session, /model still overrides it.
model_deps="claude-code"
MODEL_DEFAULT='opus[1m]'

model_detect() {
  local m; m=$(settings_get '.model // empty')
  if [ "$m" = "$MODEL_DEFAULT" ]; then echo "ok	$m"
  elif [ -n "$m" ]; then echo "partial	$m (wanted $MODEL_DEFAULT)"
  else echo "missing	not pinned"; fi
}
model_plan() { echo "settings.json: model = $MODEL_DEFAULT"; }
model_apply() { json_edit "$SETTINGS" '.model = $m' --arg m "$MODEL_DEFAULT"; }
