# CAAC documentation

| Task | Guide |
| --- | --- |
| See current and proposed milestones | [Roadmap](ROADMAP.md) |
| Create and check a first contract | [Quickstart](quickstart.md) |
| Draft a contract from an existing project | [Scan to draft](scan-to-draft.md) |
| Revise a draft and trace a clause | [Candidate authoring](candidate-authoring.md) |
| Inspect past experiments and reviews | [Reports](../reports/) |

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

## Reusable project contracts

- [Reusable project contracts](design/reusable-contracts.md): proposed format for logging, interfaces, and APIs.

## Contract references

The [method contract](../CONTRACT.md) defines the authoring rules. The [tooling contract](../contracts/tooling.md) defines the CLI checks. The [batch division example](../examples/batch-division.md) shows clauses and cases together.
