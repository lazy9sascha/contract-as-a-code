# Contract as a Code (CAAC)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org/)

**The contract is the code. The code is the contract.**

CAAC is a bidirectional framework: a contract schema generates code, and code
generates a contract. They are the same artifact in two forms.

## Table of contents

1. [Project description](#project-description)
2. [Who this project is for](#who-this-project-is-for)
3. [The contract](#the-contract)
4. [Quickstart](#quickstart)
5. [Run](#run)
6. [Checks](#checks)
7. [Roadmap](#roadmap)
8. [Contributing](#contributing)
9. [License](#license)

## Project description

With CAAC you can write contracts as programs with explicit programming
semantics. A contract schema defines operations, clauses, and cases. From that
schema, CAAC generates Rust traits and Markdown contracts. From existing Rust
code, CAAC extracts a contract schema. The two directions are reversible.

CAAC does not write contracts for you. It checks syntax, generates scaffolds,
guides you through open decisions, and converts between contract and code. The
contract content comes from you or your LLM.

CAAC combines prepared programming patterns, an LLM skill, and tools. The
controller retains decisions and evidence. A completed contract constrains
downstream implementation work.

## Who this project is for

This project is intended for software engineers and technical writers who want
contracts with explicit meaning, machine-checkable syntax, evidence-based
completion, and bidirectional code generation.

## The contract

The method rules live in [CONTRACT.md](CONTRACT.md). It defines permitted
behavior, clause IDs, and acceptance cases. Read it before changing any code.

## Quickstart

See [docs/quickstart.md](docs/quickstart.md) for a five-minute guide.

## Run

```sh
cargo run -- check CONTRACT.md contracts/tooling.md examples/batch-division.md
cargo run -- check --json CONTRACT.md contracts/tooling.md examples/batch-division.md
```

A successful check reports declared syntax only. Semantic review remains
pending. Exit codes: 0 for success, 1 for findings, 2 for input errors.

Create a new draft in an absent directory:

```sh
cargo run -- scratch /tmp/caac-scratch --title "Recipient normalization"
cargo run -- plugin /tmp/caac-plugin --title "Batch division" --source src/batch.rs
cargo run -- migration /tmp/caac-migration --title "Batch transition" --source src/batch.rs
```

Generate code from a contract schema:

```sh
cargo run -- gen schema.json --output src/trait.rs
cargo run -- md schema.json --output docs/contract.md
cargo run -- parse contract.md --output schema.json
cargo run -- extract src/trait.rs --output schema.json
```

Interview and repair loop:

```sh
cargo run -- interview contract.md
cargo run -- interview --suggest --source src/batch.rs contract.md
cargo run -- resume contract.md
cargo run -- watch contract.md
cargo run -- run --source src/batch.rs contract.md
```

Pattern library:

```sh
cargo run -- pattern list
cargo run -- pattern gen FLOW-SEQUENCE --output src/flow.rs
```

Repeat `--source` for additional files. Existing output paths cause rejection.
Generated drafts contain unresolved decisions and are expected to fail `check`.

## Checks

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

Integration tests exercise the executable, temporary source baselines, malformed
contracts, and batch behavior. A separate arithmetic oracle checks selected
division inputs.

These checks establish only their tested behavior. They do not establish
semantic completeness or conceptual 0.1.0 acceptance.

## Roadmap

See [ROADMAP.md](docs/ROADMAP.md).

## Contributing

See [.github/CONTRIBUTING.md](.github/CONTRIBUTING.md). Use conventional commits.

Install git hooks: `git config core.hooksPath .githooks`.

## License

This project is licensed under the [MIT License](LICENSE).
