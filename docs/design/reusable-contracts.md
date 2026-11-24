# Reusable project contracts

Status: design proposal. It does not change the method contract or the executable schema.

## Format

Use the same format for logging, software interfaces, and APIs. The subject changes; the contract format does not.

| Part | Content |
| --- | --- |
| Identity | Stable ID, revision, source, and status. |
| Scope | Responsible actor, covered operations, and exclusions. |
| Common behavior | Terms, typed inputs and results, and clauses with IDs. Define failures and observable effects. |
| Project binding | Exact base revision and selected clauses. Bind each required slot to a value or an admitted supplier. |
| Cases and evidence | Inputs, preconditions, and expected observations. Record what was checked separately from what is required. |

Use the existing [contract model](contract-model.md#entity-payloads) for clauses, decisions, and cases. Use a [pattern definition and instance](pattern-library.md#definition-interface) where the reusable structure has typed slots. Use a [composition relation](composition-semantics.md) only for dependencies or inheritance that the contract explicitly declares. A Markdown heading is not evidence of applicability. This proposal adds no syntax or parser feature.

## Common glossary

These terms describe project contracts. They add no schema entities. They do not replace the terms in the
[method contract](../../CONTRACT.md#meaning-of-terms).

| Term | Meaning |
| --- | --- |
| Component | A named part of a system with a defined responsibility. A component does not imply a source module or a separate process. |
| Interface | A defined boundary through which actors or components interact. |
| Operation | A named action at a defined boundary. Its contract identifies the actor, inputs, preconditions, results, failures, effects, and completion condition. |
| API | An interface that software uses to call operations. The interface can be local or remote. |
| Diagnostic event | An observation of an occurrence at a named boundary. |
| Log record | A recorded description of a diagnostic event. |
| Health check | A check that reports the observed state of a defined target. |
| Liveness | Whether the target continues to run. This does not show readiness or progress. |
| Readiness | Whether the target can accept its assigned work now. This does not show that the target completed work. |

The project defines its components, operations, and concrete checks.
PigeonMQ defines mailbox and acknowledgment terms in its own contract.

References: [SEI architecture views](https://www.sei.cmu.edu/documents/2072/2005_004_001_14498.pdf),
[OpenTelemetry glossary](https://opentelemetry.io/docs/concepts/glossary/), and
[Kubernetes probes](https://kubernetes.io/docs/concepts/workloads/pods/probes/).
These references do not set implementation requirements.

## What belongs where

| Subject | Common part | Project binding |
| --- | --- | --- |
| Logging | A diagnostic event identifies its time, component, stage, and actual outcome. Use a permitted correlation ID when the contract calls for one. Exclude message bodies and secrets. | Name the events, evidence for each result, diagnostic destination, and any separate protocol stream. |
| Interface | Each operation defines actor, typed input, output, failure, and effects at its boundary. | Name operations, types, caller obligations, and externally observable changes. |
| API transport profile | A transport profile maps operations to requests, results, and failures. Define authentication and retry behavior when applicable. | Bind the actual method, route, fields, authentication, and retry rules. |

The common part does not choose libraries, protocols, storage, or timers for every project. Add these choices only where a project guarantee needs them. Record an unbound required value as an open decision; do not invent a default.

## Example: PigeonMQ logging

The project binds its diagnostic events to queue acceptance, native processing, reply storage, original ACK, and terminal confirmation. A successful reply-and-ACK operation can produce two distinct facts from one commit. A queue acceptance does not prove native processing. An empty successful queue read can prove an observation without proving delivery.

A check supplies an accepted queue request without native admission. The expected observations include queue acceptance and exclude native admission. A second check supplies a credential and message body in the input. Neither appears in the diagnostic output.

A logging check reads the record for a named diagnostic event at the declared destination. It also checks that the protocol consumer receives unchanged protocol output. The check fails if the record does not reach that destination.

Rust `tracing` with JSON on stderr is a PigeonMQ implementation choice. It is not a CAAC requirement.

## Example: PigeonMQ operations

MCP is an interface. A tool on that interface maps to an operation with its own effects and completion condition.
A mailbox read observes a message. An RPC receive claims a request.
An ordinary reply creates a separate answer message and records the original ACK in the same transaction.
An RPC response stores the result on the original request and acknowledges it in the same transaction.
A native handoff submits work to a bound runtime. It does not prove execution.
