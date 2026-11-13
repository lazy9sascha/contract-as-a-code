# Changelog

## [0.9.1] - 2026-11-13

### Fixed
- Reject unsupported or unsafe pattern bindings rather than generating misleading output.
- Preserve complete schema data in verified generated artifacts; reject altered metadata, lossy Markdown admission, ambiguous merges and malformed signatures.
- Protect existing files and aliases during artifact publication and sync.
- Preserve interview checkpoints and contract clauses, recheck answers immediately, and share checker decisions with the harness.
- Parse Markdown structure with Comrak to distinguish code, comments, headings and normative text.

### Compatibility
- Rust library callers must handle `Result<String, String>` from `generate_rust_trait` and `generate_markdown`, and `Result<ContractSchema, String>` from `merge_schemas`. This is a source-incompatible API correction despite the patch version.
- Unsupported inputs that previously produced partial output now fail explicitly. Generated-artifact recovery requires matching metadata and content.

### Scope
- This patch contains bounded integrity repairs, not source-intake baselines, candidate authoring/interviews, revision manifests or legal-style rendering. Those features remain on the separate 0.10 development line.
- Syntax checks and regression tests do not establish semantic completion, executed contract evidence or owner acceptance. Existing authoring-role and review-gate gaps remain open.

## [0.9.0] - 2026-11-13

### Added
- `caac pattern list`: list available prepared patterns by family.
- `caac pattern gen <ID> --output <file>`: generate Rust code from a pattern template.
- `src/patterns.rs`: 5 prepared patterns (FLOW-SEQUENCE, VAL-LITERAL, FLOW-BRANCH, FAIL-RETRY, STATE-MACHINE).
- TOOL-066 through TOOL-068: pattern rules.

### Changed
- Version bumped from 0.8.0 to 0.9.0.

## [0.8.0] - 2026-11-13

### Added
- `caac run` command: contract completion loop (check → interview → repair → re-check).
- `--max-iterations` and `--source` flags for the harness.
- `src/harness.rs`: upper controller combining check, interview, and watch.
- TOOL-061 through TOOL-064: harness rules.

### Changed
- Version bumped from 0.7.0 to 0.8.0.

## [0.7.0] - 2026-11-13

### Added
- `caac interview --suggest [--source FILE]`: heuristic suggestions for open decisions.
- `src/suggest.rs`: keyword-based suggestion engine reading source code.
- TOOL-057 through TOOL-060: suggest rules.

### Changed
- Version bumped from 0.6.0 to 0.7.0.

## [0.6.0] - 2026-11-12

### Added
- `caac extract <source.rs> --output <schema.json>`: extract contract schema from Rust source.
- `caac parse` improvements: round-trip with `caac md` verified.
- Bidirectional conversion: schema ↔ Markdown ↔ Rust trait.
- TOOL-051 through TOOL-056: parse and extract rules.

### Changed
- Version bumped from 0.5.0 to 0.6.0.

### Known limitations
- Cases and terms are contract-only; not recoverable from code extraction.
- Type mapping between contract and Rust types is basic.

## [0.5.0] - 2026-11-12

### Added
- Contract schema (JSON) as machine-readable contract format.
- `caac gen <schema.json> --output <file.rs>`: generate Rust trait from schema.
- `caac md <schema.json> --output <file.md>`: generate Markdown contract from schema.
- Multi-file input support for gen and md (modular contracts).
- TOOL-045 through TOOL-050: schema, gen, and md rules.

### Changed
- Version bumped from 0.4.0 to 0.5.0.

## [0.4.0] - 2026-11-12

### Added
- `caac interview` command: walk through open decisions, prompt owner for answers,
  record as `DECISION:` lines, save state for resumption.
- `caac resume` command: continue a previous interview.
- `--auto` flag for non-interactive mode.
- TOOL-039 through TOOL-044: interview/resume rules.

### Changed
- Version bumped from 0.3.0 to 0.4.0.

## [0.3.0] - 2026-11-12

### Added
- `caac watch` command: repair loop that re-runs syntax checks on file changes.
- `--interval` flag for watch polling (default 2 seconds).

### Changed
- Version bumped from 0.2.0 to 0.3.0.

## [0.2.0] - 2026-11-12

### Added
- Community documentation: CONTRIBUTING.md, CODE_OF_CONDUCT.md, SECURITY.md,
  SUPPORT.md, GOVERNANCE.md, LICENSE.
- Git hooks for pre-commit checks and commit message validation.
- Release script (`scripts/release.sh`).
- CAAC-style contract for PigeonMQ common-queue.md (Q-001 through Q-082).
- T15 task for public release preparation.

### Changed
- Version bumped from 0.1.0-dev to 0.2.0.
- Pattern library expanded (control, values, state, abstraction, expressions,
  concurrency, failure, functions, effects, organization).
- Concept document revised with CONCEPT-001 through CONCEPT-020.
- Task lifecycle expanded (LIFE-001 through LIFE-079).

### Fixed
- (none yet)

## [0.1.0-dev] - 2026-11-11

### Added
- Initial prototype: author, batch, checker, CLI.
- Batch division example contract and Rust implementation.
- Method contract (CONTRACT.md) with CAAC-001 through CAAC-039.
- Tooling contract with TOOL-001 through TOOL-033.
- Task system with T01 through T14.
- Pattern library (provisional).
