---
title: "CAAC roadmap"
doc_type: "guide"
audience: "contributors"
status: "maintained"
publication: "review-required"
---

# CAAC roadmap

## Completed

| Version | Key deliverable | Status |
| --- | --- | --- |
| 0.1.0-dev | Syntax checker, batch example, CLI scaffold | Done |
| 0.2.0 | Community docs, git hooks, CI workflow | Done |
| 0.3.0 | Repair watch loop (`caac watch`) | Done |
| 0.4.0 | Interview and resume (`caac interview`, `caac resume`) | Done |
| 0.5.0 | Contract schema (JSON), code generation (`caac gen`, `caac md`, `caac parse`) | Done |
| 0.6.0 | Reverse extraction (`caac extract`) + bidirectional sync | Done |

## Planned

| Version | Key deliverable | Scope |
| --- | --- | --- |
| 0.7.0 | LLM skill execution (T07) | Interview with suggested answers |
| 0.8.0 | Contract harness (T11) | Upper controller: loop, repair, owner questions |
| 0.9.0 | Pattern library execution (T03) | Prepared counterparts with code generation | Done |
| 1.0.0 | Conceptual acceptance (T14) | Integrated baseline, owner acceptance |

## References

- [CONTRACT.md](../CONTRACT.md): method rules.
- [contracts/tooling.md](../contracts/tooling.md): CLI and checker rules.
- [examples/batch-division.md](../examples/batch-division.md): complete example.

## Principles

- Each version adds one verifiable capability.
- No version claims completeness without evidence.
- T-shirt sizing determines contract scope before writing.
- Contract is the source of truth. Code is derived.

## Current state

Version 0.6.0. Next: 0.7.0 (LLM skill execution with suggested answers).
