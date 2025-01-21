default:
    @just --list

alias b := build
alias br := build-release
alias f := fmt
alias c := clean

# variables

elf-path := "./target/release/eq-program-keccak-inclusion"

# Private just helper recipe
_pre-build:
    {{ if path_exists(elf-path) == "false" { `cargo b -r -p eq-program-keccak-inclusion` } else { "" } }}

build: _pre-build
    cargo b

build-release: _pre-build
    cargo b -r

clean:
    #!/usr/bin/env bash
    set -euxo pipefail
    cargo clean

fmt:
    @cargo fmt
    @just --quiet --unstable --fmt > /dev/null
