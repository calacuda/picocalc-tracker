_:
  @just -l

check:
  @just cargo check

build:
  @just cargo build

cargo CMD:
  # cargo +stable {{CMD}}
  cargo {{CMD}}

flash:
  elf2uf2-rs -d target/thumbv6m-none-eabi/debug/test-1

run:
  # DEFMT_LOG=trace cargo +stable run
  DEFMT_LOG=trace cargo run

# tmux:
#   tmux new -ds pico-dc -n "README"
#   tmux send-keys -t pico-dc:README 'nv ./README.md "+set wrap"' ENTER
#   # @just new-window "Reff" ""
#   @just new-window "Edit" ""
#   @just new-window "Run" ""
#   @just new-window "Git" "git status"
#   tmux a -t pico-dc
#
# new-window NAME CMD:
#   tmux new-w -t pico-dc -n "{{NAME}}"
#   tmux send-keys -t pico-dc:"{{NAME}}" "{{CMD}}" ENTER

test:
  cargo test --target x86_64-unknown-linux-gnu --lib

_new-tmux-dev-session SESSION:
  tmux new -ds "{{SESSION}}" -n "README"
  tmux send-keys -t "{{SESSION}}":README 'nv ./README.md "+set wrap"' ENTER
  @just _new-window "{{SESSION}}" "Edit" "nv src/{bin/pico-tracker.rs,lib.rs,**/*.rs}"
  @just _new-window "{{SESSION}}" "Run" "cargo check"
  @just _new-window "{{SESSION}}" "Git" "git status"
  @just _new-window "{{SESSION}}" "Misc" ""

_new-window SESSION NAME CMD:
  tmux new-w -t "{{SESSION}}" -n "{{NAME}}"
  # tmux send-keys -t "{{SESSION}}":"{{NAME}}" ". ./.venv/bin/activate" ENTER
  [[ "{{CMD}}" != "" ]] && tmux send-keys -t "{{SESSION}}":"{{NAME}}" "{{CMD}}" ENTER || true

tmux:
  tmux has-session -t '=pico-tracker' || just _new-tmux-dev-session pico-tracker
  tmux a -t '=pico-tracker'

