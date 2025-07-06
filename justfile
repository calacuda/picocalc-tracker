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

tmux:
  tmux new -ds pico-dc -n "README"
  tmux send-keys -t pico-dc:README 'nv ./README.md "+set wrap"' ENTER
  # @just new-window "Reff" ""
  @just new-window "Edit" ""
  @just new-window "Run" ""
  @just new-window "Git" "git status"
  tmux a -t pico-dc

new-window NAME CMD:
  tmux new-w -t pico-dc -n "{{NAME}}"
  tmux send-keys -t pico-dc:"{{NAME}}" "{{CMD}}" ENTER

