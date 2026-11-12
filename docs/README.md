# Design and evidence index

The documents below specify the proposed contract-completion product. They are conceptual designs, not implemented feature claims.

Independent content review, owner acceptance, runtime evidence, and publication remain separate results.

## Conceptual deliverables

| Task | Document | Responsibility |
| --- | --- | --- |
| T01 | [Product concept](concept.md) | Product roles, foundations, authority, and conceptual boundary. |
| T02 | [Contract model](design/contract-model.md) | Canonical meaning, typed slots, revisions, decisions, and evidence. |
| T03 | [Prepared pattern library](design/pattern-library.md) | Thirty-four proposed counterparts across ten linked family catalogs. |
| T04 | [Composition semantics](design/composition-semantics.md) | Typed relations, scope, exits, cleanup, substitution, and edge obligations. |
| T05 | [Language profiles](design/language-profiles.md) | Profile schema, scoped source-backed mappings, coverage, and explicit gaps. |
| T06 | [Tool protocol](design/tool-protocol.md) | Proposed catalog, requests, results, receipts, limits, and evidence. |
| T07 | [LLM skill](design/llm-skill.md) | Authoring instructions, host context, questions, routing, and repairs. |
| T08 | [Scratch workflow](design/scratch-workflow.md) | Contract creation from intent without an implementation baseline. |
| T09 | [Plugin adoption](design/plugin-adoption.md) | Bounded existing-source observation, responsibility coverage, and proposed requirements. |
| T10 | [Migration workflow](design/migration-workflow.md) | Current/target/transition contracts, state, effects, cutover, and recovery. |
| T11 | [Completion controller](design/completion-controller.md) | Revision-bound authoring states, gates, budgets, checkpoints, and resumption. |
| T12 | [Implementation boundary](design/implementation-boundary.md) | Action bindings, interception coverage, separate runtime authority, and nonconformance. |
| T13 | [Rust architecture](design/rust-architecture.md) | Core/host boundaries, prototype transition, distribution, and evidence. |
| T14 | [Conceptual acceptance](design/conceptual-acceptance.md) | Integrated baseline, scenarios, review outcomes, and owner gate. |

## Sources and review evidence

- [Method contract](../CONTRACT.md) supplies authoring and review rules.
- [CONTRACT.md](../CONTRACT.md) is the canonical method contract.
- [examples/batch-division.md](../examples/batch-division.md) is the reference example.

Retained artifact hashes identify the reviewed bytes. They do not supply owner acceptance or executed semantic evidence automatically.

## Implemented prototype

The [root README](../README.md) documents the Rust scaffold and check commands.
The [tooling contract](../contracts/tooling.md) bounds the prototype.

The designs above specify additional responsibilities. They do not claim those
components run today.
