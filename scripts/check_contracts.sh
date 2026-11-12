#!/bin/sh
# Run declared syntax checks on contract Markdown files only.
set -e
cargo run --quiet -- check \
    CONTRACT.md \
    contracts/tooling.md \
    examples/batch-division.md
