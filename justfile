_:
  @just -l

_new-tmux-dev-session SESSION:
  tmux new -ds "{{SESSION}}" -n "README"
  tmux send-keys -t "{{SESSION}}":README 'nv ./README.md "+set wrap"' ENTER
  @just _new-window "{{SESSION}}" "Edit" "nv src/{bin/pico-tracker.rs,lib.rs,**/*.rs}"
  @just _new-window "{{SESSION}}" "Run" "cargo check"
  @just _new-window "{{SESSION}}" "Git" "git status"
  @just _new-window "{{SESSION}}" "Misc" ""

_new-window SESSION NAME CMD:
  tmux new-w -t "{{SESSION}}" -n "{{NAME}}"
  tmux send-keys -t "{{SESSION}}":"{{NAME}}" "cd ./pico-tracker/" ENTER
  [[ "{{CMD}}" != "" ]] && tmux send-keys -t "{{SESSION}}":"{{NAME}}" "{{CMD}}" ENTER || true

tmux:
  tmux has-session -t '=pico-tracker' || just _new-tmux-dev-session pico-tracker
  tmux a -t '=pico-tracker'

