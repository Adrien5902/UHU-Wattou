alias w := watch
alias r := run
alias b := build

watch:
    watchexec -r cargo run --bin bot

build:
    cargo b -r --bin bot

run:
    cargo run --bin bot
