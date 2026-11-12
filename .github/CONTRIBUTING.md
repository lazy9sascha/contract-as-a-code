# Contributing Guide

Thank you for your interest in contributing to CAAC.

## How to contribute

1. Fork the repository.
2. Create a branch: `git checkout -b feat/your-change`.
3. Make your changes.
4. Run checks: `cargo test --offline && cargo clippy --all-targets --offline --locked -- -D warnings`.
5. Run syntax check: `cargo run -- check <files>`.
6. Commit with a conventional message: `feat: ...`, `fix: ...`, `docs: ...`.
7. Open a pull request.

## Commit message format

Use conventional commits:

- `feat:` new feature or capability.
- `fix:` bug fix.
- `docs:` documentation change.
- `chore:` maintenance.
- `refactor:` code restructuring without behavior change.

## What to contribute

- Method rule improvements (CONTRACT.md).
- Pattern definitions (docs/design/patterns/).
- Tool implementations (src/).
- Documentation (docs/).
- Test cases (tests/).

## Decision records

Record decisions in docs/concept.md using the D01/D02 format.
Do not add ADR files. The concept document is the single home for decisions.

## Code of Conduct

This project follows the [Code of Conduct](CODE_OF_CONDUCT.md).

## Contact

Email: lazy9sascha@agentmail.to

## References

- [README.md](../README.md)
- [CONTRACT.md](../CONTRACT.md)
 - [ROADMAP.md](../docs/ROADMAP.md)

## License

By contributing, you agree that your contributions are licensed under the
MIT License.
