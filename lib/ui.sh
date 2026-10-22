# Prompts and colour. gum when it is installed and we own a terminal; numbered plain
# prompts otherwise. Callers check ui_interactive before prompting at all, so nothing
# here ever blocks on a closed stdin.

ui_interactive() { [ -t 0 ] && [ -t 1 ]; }
# gum draws on stderr and prints the answer on stdout, so stdout may be a capture.
ui_has_gum() { command -v gum >/dev/null 2>&1 && [ -t 0 ] && [ -t 2 ]; }

if [ -t 1 ]; then
  C_OK=$'\e[32m' C_PART=$'\e[33m' C_BAD=$'\e[31m' C_DIM=$'\e[2m' C_ACC=$'\e[35m' C_OFF=$'\e[0m'
else
  C_OK='' C_PART='' C_BAD='' C_DIM='' C_ACC='' C_OFF=''
fi

# state -> coloured mark
ui_mark() {
  case $1 in
    ok)      printf '%s●%s' "$C_OK" "$C_OFF" ;;
    partial) printf '%s◐%s' "$C_PART" "$C_OFF" ;;
    missing) printf '%s○%s' "$C_DIM" "$C_OFF" ;;
    broken)  printf '%s✗%s' "$C_BAD" "$C_OFF" ;;
    needs)   printf '%s!%s' "$C_PART" "$C_OFF" ;;
    *)       printf '?' ;;
  esac
}

ui_head() { printf '\n%s%s%s\n' "$C_ACC" "$*" "$C_OFF"; }

# ui_choose_multi HEADER PRESELECTED_CSV ITEM... -> chosen items, one per line
ui_choose_multi() {
  local header=$1 pre=$2; shift 2
  if ui_has_gum; then
    gum choose --no-limit --header "$header" --selected "$pre" "$@" || true
    return
  fi
  local -a items=("$@") on=()
  local i n line
  for i in "${!items[@]}"; do
    [[ ",$pre," == *",${items[$i]},"* ]] && on[i]=1 || on[i]=0
  done
  while :; do
    printf '\n%s\n' "$header" >&2
    for i in "${!items[@]}"; do
      printf '  %2d [%s] %s\n' $((i + 1)) "$([ "${on[i]}" = 1 ] && echo x || echo ' ')" "${items[$i]}" >&2
    done
    printf 'numbers to toggle (e.g. "1 3"), enter to accept: ' >&2
    IFS= read -r line || line=''
    [ -z "$line" ] && break
    for n in $line; do
      [[ $n =~ ^[0-9]+$ ]] && [ "$n" -ge 1 ] && [ "$n" -le "${#items[@]}" ] || continue
      on[n - 1]=$((1 - on[n - 1]))
    done
  done
  for i in "${!items[@]}"; do [ "${on[i]}" = 1 ] && printf '%s\n' "${items[$i]}"; done
  return 0
}

ui_confirm() {
  if ui_has_gum; then gum confirm "$1"; return; fi
  local a
  printf '%s [y/N] ' "$1" >&2
  IFS= read -r a || a=''
  [[ $a =~ ^[Yy] ]]
}
