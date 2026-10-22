# agents + skills: the subagent roster (files/agents/*.md) and user skills
# (files/skills/<name>/) copied into ~/.claude. Files there that are not ours stay.
agents_skills_deps="claude-code"

agents_skills_stale() {
  local f rel
  ( cd "$ROOT/files" && find agents skills -type f ) | while read -r rel; do
    f=$CLAUDE_DIR/$rel
    cmp -s "$ROOT/files/$rel" "$f" || echo "$rel"
  done
}
agents_skills_detect() {
  local total stale
  total=$( (cd "$ROOT/files" && find agents skills -type f) | wc -l)
  stale=$(agents_skills_stale | wc -l)
  if [ "$stale" -eq 0 ]; then echo "ok	$total files current"
  elif [ "$stale" -eq "$total" ]; then echo "missing	none installed ($total files)"
  else echo "partial	$stale of $total missing or outdated: $(agents_skills_stale | xargs -n1 dirname | sort -u | xargs -n1 basename | tr '\n' ' ')"; fi
}
agents_skills_plan() { agents_skills_stale | sed "s|^|install files/& -> $CLAUDE_DIR/|"; }
agents_skills_apply() {
  local rel
  agents_skills_stale | while read -r rel; do install -D -m0644 "$ROOT/files/$rel" "$CLAUDE_DIR/$rel"; done
}
