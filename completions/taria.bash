_taria_completions() {
  local cur prev commands sources
  cur="${COMP_WORDS[COMP_CWORD]}"
  prev="${COMP_WORDS[COMP_CWORD-1]}"
  commands="search s install i remove r info list ls update u upgrade up doctor report self-update help"
  sources="apt snap pacman github all"
  case "$prev" in
    taria) COMPREPLY=( $(compgen -W "$commands" -- "$cur") ); return ;;
    --source) COMPREPLY=( $(compgen -W "$sources" -- "$cur") ); return ;;
  esac
  if [[ "$cur" == --* ]]; then
    COMPREPLY=( $(compgen -W "--source --purge --full --check --help --version" -- "$cur") )
  fi
}
complete -F _taria_completions taria
