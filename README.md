# Contract as a Code (CAAC)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org/)

CAAC is an experimental contract-authoring toolkit. A JSON schema generates
Rust trait scaffolds and Markdown. Unchanged generated artifacts retain the
complete schema in versioned metadata; they do not enforce its obligations.

## Roadmap

- **[0.10.0 · Released](https://github.com/lazy9sascha/contract-as-a-code/releases/tag/v0.10.0).** Generate a reviewable contract draft from a selected project. [Verified scope and follow-up work](docs/ROADMAP.md#0100-contract-drafts)
- **0.11.0 · Planned after 0.10.0.** Enforce selected CAAC rules with contract lint and an execution harness. Trace a result to its clause, code and test revisions and rerun it when a bound revision changes. [Example and completion criteria](docs/ROADMAP.md#0110-execution-evidence)

[Browse the backlog by milestone](docs/ROADMAP.md#backlog)

## Table of contents

1. [Project description](#project-description)
2. [Who this project is for](#who-this-project-is-for)
3. [The contract](#the-contract)
4. [Quickstart](#quickstart)
5. [Run](#run)
6. [Checks](#checks)
7. [Contributing](#contributing)
8. [License](#license)

## Project description

With CAAC you can write contracts as programs with explicit programming
semantics. A contract schema defines operations, clauses, and cases. From that
schema, CAAC generates Rust traits and Markdown contracts. Extraction recovers
the schema only from unchanged CAAC-generated Rust. Arbitrary or legacy Rust
is rejected rather than silently producing an incomplete contract.

CAAC does not write contracts for you. It checks syntax, generates scaffolds,
guides you through open decisions, and converts between contract and code. The
contract content comes from you or your LLM.

CAAC combines prepared programming patterns, an LLM skill, and local tools.
The interview records decisions. The harness performs a pre-review check, not
semantic completion or owner acceptance. Downstream enforcement remains a
provisional design, not an implemented security boundary.

## Who this project is for

This project is intended for software engineers and technical writers who want
contracts with explicit meaning, machine-checkable syntax, evidence-based
completion, and bidirectional code generation.

## The contract

The method rules live in [CONTRACT.md](CONTRACT.md). It defines permitted
behavior, clause IDs, and acceptance cases. Read it before changing any code.

## Quickstart

See [docs/quickstart.md](docs/quickstart.md) for a five-minute CLI guide.

Use a coding agent to draft or review a contract. Choose the skill in the
[feature map and flowchart](docs/skill-workflows.md).

For a local MCP client, start the installed binary with `caac mcp`. The
[`caac_lint` tool](docs/skill-workflows.md#local-mcp-review) checks selected Markdown.

## Run

```sh
cargo run -- check CONTRACT.md contracts/tooling.md examples/batch-division.md
cargo run -- check --json CONTRACT.md contracts/tooling.md examples/batch-division.md
```

A successful check reports declared syntax only. Semantic review remains
pending. Exit codes: 0 for success, 1 for findings, 2 for input errors.

Run `cargo run -- lint CONTRACT.md` to validate case inputs, results, and clause
references. See [contract lint](docs/contract-lint.md).

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
cargo run -- parse docs/contract.md --output parsed-schema.json
cargo run -- extract src/trait.rs --output extracted-schema.json
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

Repeat `--source` for additional files. Existing output paths, including
symlinks, cause rejection. Generation never replaces an existing schema or file.
Generated drafts contain unresolved decisions and are expected to fail `check`.

Conversion boundaries:
- Preserve the original JSON schema as the source of truth. Edit it and generate
  new artifacts instead of editing generated Rust or Markdown in place.
- Metadata-backed extraction verifies the complete rendered artifact. Edited,
  unsupported, or unmarked Rust fails instead of losing requirements.
- Handwritten Markdown parsing supports declared clauses across sections and
  excludes fences, HTML comments, and non-normative sections. It is not a
  natural-language compiler. Unrepresented prose is not recovered as semantics.
- Rust generation accepts a restricted signature grammar. Unsupported syntax,
  ambiguous merges, and unknown schema fields produce errors.
- `sync schema.json --code new.rs --docs new.md` requires absent destinations.
  `--watch` updates only outputs created by that invocation and stops when it
  detects external output edits. This detects edits between polls; it is not a
  filesystem lock against simultaneous writers. Multiple outputs are not one
  filesystem transaction; an I/O failure can leave earlier outputs published.
- The pre-review check does not validate execution results, independent review,
  owner acceptance, or revision-bound completion evidence.

## Checks

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

Integration tests exercise authoring, checking, lossless generated-artifact
round trips, malformed conversion inputs, output safety, interview recovery,
and harness progress. Generated Rust fixtures are compiled. A separate
arithmetic oracle checks selected division inputs.

These checks establish only their tested behavior. They do not establish
semantic completeness or conceptual 0.1.0 acceptance.

## Contributing

See [.github/CONTRIBUTING.md](.github/CONTRIBUTING.md). Use conventional commits.

Install git hooks: `git config core.hooksPath .githooks`.

## License

This project is licensed under the [MIT License](LICENSE).
