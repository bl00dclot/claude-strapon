# plugins: user-scope plugins from files/plugins.txt, removing files/plugins-remove.txt.
# Marketplaces in files/marketplaces.txt are added first. Plugins not on either list are
# left alone. Account 2 shares them through its plugins/ + settings.json links.
plugins_deps="claude-code"

plugins_list() { sed 's/#.*//; /^[[:space:]]*$/d' "$ROOT/files/$1"; }
plugins_installed() {
  claude plugin list --json 2>/dev/null | jq -r '.[] | select(.scope == "user") | "\(.id) \(.enabled)"' | sort -u
}
# want/remove/disabled -> one id per line, computed against one `plugin list` call
plugins_diff() {
  local inst; inst=$(plugins_installed)
  PLUG_ADD=$(comm -23 <(plugins_list plugins.txt | sort -u) <(awk '{print $1}' <<<"$inst" | sort -u))
  PLUG_DEL=$(comm -12 <(plugins_list plugins-remove.txt | sort -u) <(awk '{print $1}' <<<"$inst" | sort -u))
  PLUG_OFF=$(comm -12 <(plugins_list plugins.txt | sort -u) <(awk '$2 == "false" {print $1}' <<<"$inst" | sort -u))
  PLUG_MKT=$(comm -23 <(plugins_list marketplaces.txt | awk '{print $1}' | sort -u) \
    <(claude plugin marketplace list 2>/dev/null | sed -n 's/^ *❯ \([^ ]*\)$/\1/p' | sort -u))
}
plugins_detect() {
  plugins_diff
  local n; n=$(plugins_list plugins.txt | wc -l)
  local parts=()
  [ -z "$PLUG_ADD" ] || parts+=("install: $(echo $PLUG_ADD | sed 's/@[^ ]*//g')")
  [ -z "$PLUG_DEL" ] || parts+=("remove: $(echo $PLUG_DEL | sed 's/@[^ ]*//g')")
  [ -z "$PLUG_OFF" ] || parts+=("enable: $(echo $PLUG_OFF | sed 's/@[^ ]*//g')")
  if [ ${#parts[@]} -eq 0 ]; then echo "ok	$n wanted, all enabled"
  else echo "partial	$(IFS=';'; echo "${parts[*]}" | sed 's/;/ · /g')"; fi
}
plugins_plan() {
  plugins_diff
  local m p
  for m in $PLUG_MKT; do echo "claude plugin marketplace add $(plugins_list marketplaces.txt | awk -v m="$m" '$1 == m {print $2}')"; done
  for p in $PLUG_ADD; do echo "claude plugin install -s user $p"; done
  for p in $PLUG_OFF; do echo "claude plugin enable $p"; done
  for p in $PLUG_DEL; do echo "claude plugin uninstall -s user -y $p"; done
}
plugins_apply() {
  plugins_diff
  local m p rc=0
  for m in $PLUG_MKT; do claude plugin marketplace add "$(plugins_list marketplaces.txt | awk -v m="$m" '$1 == m {print $2}')" || rc=1; done
  for p in $PLUG_ADD; do claude plugin install -s user "$p" || rc=1; done
  for p in $PLUG_OFF; do claude plugin enable "$p" || rc=1; done
  for p in $PLUG_DEL; do claude plugin uninstall -s user -y "$p" || rc=1; done
  return $rc
}
