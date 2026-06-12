alias w := watch
watch:
    watchexec cargo run

alias b := build
build:
    cargo b -r

alias r := run
run:
    cargo run

alias i := install
install:
    cargo install --locked --path .
