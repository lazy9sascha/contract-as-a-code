# Tool protocol

Revision: `tool-protocol-draft-3`. Status: provisional design. Designer: `the reviewer`. Ticket: T06 revision 2.

This document proposes D09 and D10. Neither decision has owner acceptance. No protocol implementation or executed fixture is claimed.

## Purpose and authority

Tools supply bounded evidence to the contract-completion controller. The LLM selects prepared patterns, fills slots, interprets findings, and proposes repairs.

A tool pass covers one declared method and scope. The controller combines required evidence, reviews, capability coverage, and owner decisions.

| Source | Use and revision |
| --- | --- |
| T06 specification | Retained r2, SHA-256 `6c6be5052df2bbfc93792e028c9eae869cb02458dfe05be294f6629926208d5f`. Protocol requirements and cases. |
| RUN-001 | S01 authorizes design. S02 supplies the contract-completion loop. Neither accepts D09/D10. |
| Input manifest | Exact observed method, prototype, and T02–T05 requirements. |
| [Method contract](../../CONTRACT.md) | Draft clauses CAAC-001–CAAC-039 govern precision, authority, and evidence separation. |
| [Prototype contract](../../contracts/tooling.md) | Draft-1 defines the existing syntax checks. It does not define this protocol. |
| [Checker](../../src/checker.rs), [CLI](../../src/cli.rs), [tests](../../tests/cli.rs) | Source inspection only. Test files establish intended cases, not observed test success. |

T02–T05 have no accepted outputs at this drafting baseline. Their acceptance remains a close gate.

Provisional assumptions: T02 supplies stable entity references and exact revision identities. T03 supplies check obligations for prepared patterns.

T04 supplies relation meanings. T05 supplies language profiles and capability coverage. This protocol preserves their identities without deciding their semantics.

## Terms and scope

| Term | Meaning |
| --- | --- |
| Caller | The contract controller that requests a tool operation. |
| Dispatcher | The boundary that validates a request and selects a registered tool build. |
| Tool build | One implementation identified by tool ID, version, and retained executable digest. |
| Descriptor | A versioned declaration of one build's capability, method, limits, and evidence meaning. |
| Revision reference | Contract ID, revision ID, manifest digest, and retained manifest location. |
| Manifest | An immutable inventory of all inputs that determine the requested meaning, including dependencies and profiles. |
| Target | A stable entity reference with revision and optional slot path. Relations and guarantees are entities. |
| Check unit | One declared check ID applied to one target or declared target group. |
| Assessment | One check unit's result, method, findings, evidence, and limits. |
| Finding | A located defect, capability gap, uncertain obligation, or operational failure. |
| Evidence reference | A retained artifact's locator, SHA-256 digest, byte count, media type, and bounded covered scope. |
| Receipt | The caller's record of whether a response is valid, current, and eligible as evidence. |

Included operations inspect immutable contract inputs. They do not execute implementation code, contact services, or decide requirements.

Excluded operations include general proof, automatic owner acceptance, implementation-harness actions, and unrestricted patch application.

TOOLPROTO-001 | The caller must separate tool assessments, LLM interpretations, and owner decisions.

TOOLPROTO-002 | The caller must bind each invocation to an exact revision reference and explicit check units.

TOOLPROTO-003 | The caller must retain every input that determines an invocation's result.

TOOLPROTO-004 | The dispatcher must reject unresolved revision references before invoking a checker.

## Catalog and capability discovery

The following IDs are abstract design IDs. They are not installed commands or implemented pattern tools.

Every proposed entry shares these properties: immutable inputs, no contract changes, no external effects, explicit method scope, and D10 bounds below.

| Design ID | Owning requirements | Inputs | Outputs and evidence meaning | Supported scope and assumptions |
| --- | --- | --- | --- | --- |
| `structure.inspect` | T02-001, T03-001 | Exact package and selected pattern entries | Containment, missing children, unknown fields. Evidence covers declared structure rules only. | Registered entry schemas. No inferred pattern inventory. |
| `slots.check` | T02-002, T03-002 | Instances, slot declarations, explicit conditional predicates | Missing, mistyped, or invalid slot findings. Evidence covers decidable supplied constraints. | Registered slot types and supported predicates. Unknown predicate methods remain gaps. |
| `names.resolve` | T04-001, T04-002 | Entities, bindings, explicit scope rules | Resolved references or exact unresolved/ambiguous targets. Evidence covers registered scope rules. | Declared names, visibility, and target rules. No guessed target. |
| `composition.check` | T04-003–T04-009 | Nodes, edges, base revisions, composition rules | Edge obligations, incompatible flow, substitution conflicts, unresolved constraints. | Registered relation methods. Arbitrary semantic equivalence remains outside scope. |
| `profile.check` | T05-001–T05-007 | Exact profile, feature mappings, selected guarantees | Compatibility results or affected unsupported guarantees. | Registered profile/version/settings tuples only. No profile substitution. |
| `cases.evaluate` | CAAC-031–CAAC-034, T06-010 | Versioned finite cases and declared evaluation method | Per-case expected/derived observable comparison and limits. | Finite supported contract methods. No implementation execution or exhaustive proof. |
| `revision.impact` | T02-007, T02-008, T05-007 | Before/after manifests, lineage, evidence dependency graph | Changed targets and affected evidence references. | Explicit dependency edges. Unknown edges invalidate the enclosing scope conservatively. |

Each row is **specified provisionally**, **implementation unavailable**, and **test evidence absent**. Scope becomes callable only through a registered descriptor.

Required descriptor fields are `tool_id`, `tool_version`, `build_digest`, `descriptor_revision`, `descriptor_digest`, `protocol_versions`, and `owning_requirements`.

Its capability fields are `check_ids`, `pattern_entry_revisions`, `profile_revisions`, `method_id`, `assumptions`, `limits`, `side_effects`, and `evidence_meaning`.

Its maturity fields separately record `design_state`, `implementation_state`, and exact `test_evidence`. A tested subset does not establish untested coverage.

Each check ID names target kinds, admitted inputs, check-unit expansion, supported predicates, output guarantees, and undecidable conditions.

TOOLPROTO-005 | The dispatcher must expose exact descriptors before the caller selects checking capabilities.

TOOLPROTO-006 | The caller must select a registered descriptor admitting each requested operation and target kind.

TOOLPROTO-007 | The caller must retain unknown or absent capabilities as gaps against affected guarantees.

TOOLPROTO-008 | The dispatcher must not advertise an unavailable implementation as callable.

TOOLPROTO-009 | Each descriptor must separate specified capability, implemented capability, and retained test evidence.

TOOLPROTO-010 | Each descriptor must identify its supported methods, patterns, profiles, assumptions, limits, effects, and evidence meaning.

TOOLPROTO-011 | The caller must invalidate affected evidence after a descriptor, method, profile, or dependency revision changes.

Dispatch admission differs from semantic support. A registered check can admit a feature description and report that feature as unsupported.

A descriptor declares both its admitted operations and its semantic coverage. Discovery gaps prevent dispatch when no operation admits the request.

Known unsupported features can receive preflight capability-gap receipts. A callable admitted check can instead return their located `unsupported` assessments.

Pattern-family allocation remains provisional. T03 can introduce additional checks without silently expanding an existing descriptor.

| T03 family | Required checking responsibility | Explicit remaining limit |
| --- | --- | --- |
| Values and types | `structure.inspect`, `slots.check`, `profile.check` | Actual type domains and language mappings await T02/T05. |
| Expressions | `names.resolve`, `slots.check`, `composition.check`, `profile.check` | Evaluation semantics and predicate methods await T03–T05. |
| Control flow | `structure.inspect`, `names.resolve`, `composition.check` | Exit targets and termination methods await T03/T04. |
| Functions | `slots.check`, `names.resolve`, `composition.check` | General precondition/postcondition implication has no universal checker. |
| Failure | `slots.check`, `composition.check`, `cases.evaluate` | Retry effects, cancellation, and cleanup require declared methods. |
| Abstraction | `names.resolve`, `composition.check`, `profile.check` | Arbitrary substitution proof remains an unresolved checking responsibility. |
| State and resources | `slots.check`, `composition.check`, `cases.evaluate` | Lifetime and invariant proofs need bounded supported methods. |
| Concurrent behavior | `composition.check`, `profile.check`, `cases.evaluate` | Unbounded schedules and external behavior remain unsupported. |
| External effects | `composition.check`, `cases.evaluate` | This protocol does not observe live services or establish actual effect safety. |
| Organization | `structure.inspect`, `names.resolve`, `composition.check` | Import and initialization rules await T03–T05. |

`revision.impact` applies to every family's evidence. No family currently has implemented semantic coverage in this protocol.

### Current prototype observation

The existing Rust `caac check --json` reports `syntax`, `semantic_review`, and diagnostics containing path, line, code, and message.

Source implements the declared syntax checks. Source and tests include draft generation, but these commands do not implement this envelope.

The inspected checker lacks exact input-revision, tool-version, capability-descriptor, check-unit, and retained-evidence fields.

Its JSON pass therefore establishes declared syntax only. No protocol adapter, semantic pattern checker, or runtime harness is claimed.

Current maturity: syntax implementation observed in source. Tests present. Tests not executed for this design task. Protocol conformance unavailable.

## D09 proposal: request and response envelopes

Proposal: use versioned UTF-8 JSON objects. A local Rust adapter receives one request through standard input and returns one response.

Standard error carries bounded operational logs. A host adapter can transport the same objects without becoming requirement authority.

This is a transport proposal, not T02's canonical contract representation. Referenced contract bytes retain their own meaning and identity.

JSON objects have unique keys. Unknown fields, unknown versions, duplicate keys, malformed values, and unresolved references cause rejection.

IDs are nonempty strings of at most 128 ASCII characters. Digests contain 64 lowercase hexadecimal characters and identify complete retained bytes.

### Request fields

| Field | Type and cardinality | Meaning and rejection rule |
| --- | --- | --- |
| `protocol_version` | One version string | Exact supported envelope version. No silent downgrade. |
| `request_id` | One unique ID | Caller-assigned invocation identity. Reuse with different bytes is invalid. |
| `tool` | One object | Exact `tool_id`, `tool_version`, `build_digest`, and `descriptor_digest`. |
| `contract` | One revision reference | Exact contract ID/revision, manifest digest, and retained location. |
| `check_units` | Nonempty array | Unique unit ID, check ID, and nonempty target list. Descriptor determines permitted grouping. |
| `inputs` | Array | Evidence references for all required artifacts, including profile, patterns, dependencies, and cases. |
| `parameters` | One object | Method-specific declared parameters. Unknown parameters are invalid. |
| `limits` | One object | Requested timeout, input bytes, output bytes, and unit count within descriptor maxima. |
| `retry_chain` | One object | Root request ID, attempt number 0 or 1, original total budget, and remaining budget. |
| `predecessor` | Optional object | Required at attempt 1. Previous request ID, digest, execution receipt reference, and retry reason. Absent at attempt 0. |
| `mutation` | Absent for all catalog entries | A present value is invalid for these read-only tools. |

A target contains `entity_id`, `revision_id`, and `kind`. An optional `slot_path` uses an ordered array of declared slot names.

The caller computes `request_digest` from the exact transmitted request bytes. The response echoes that digest.

### Response fields

| Field | Type and cardinality | Meaning and rejection rule |
| --- | --- | --- |
| `protocol_version`, `request_id`, `request_digest` | One each | Echo the validated invocation. Mismatch rejects the response. |
| `tool`, `contract` | One each | Echo exact tool build, descriptor, and input revision. |
| `assessments` | One per requested unit | Unique unit ID, check ID, targets, status, method ID, findings, evidence, assumptions, and limits. |
| `status` | One result state | Aggregate according to the result rules below. |
| `coverage` | One object | Exact requested, completed, and uncompleted unit IDs, with reasons for unfinished units. |
| `execution` | One object | Outcome `completed`, `timeout`, `cancelled`, `crashed`, or `limit-exceeded`, elapsed milliseconds, and operational reason. |
| `diagnostics` | Array | Located findings and operational diagnostics, ordered under D10. |
| `evidence` | Array | Retained artifacts linked from assessments. Empty for results with no usable evidence. |

An assessment's evidence states the covered check unit, target revision, method, assumptions, and observed conclusion.

`completed` means the method returned a determinate, unsupported, or indeterminate assessment. It does not mean the check passed.

TOOLPROTO-012 | The dispatcher must validate the complete request before any tool operation.

TOOLPROTO-013 | The dispatcher must reject unknown fields and unsupported envelope versions without silent conversion.

TOOLPROTO-014 | The dispatcher must check retained artifact bytes against every declared digest before analysis.

TOOLPROTO-015 | Each response must echo its request identity, request digest, exact tool build, descriptor, and contract revision.

TOOLPROTO-016 | Each response must account for every requested check unit exactly once.

TOOLPROTO-017 | Each assessment must identify its method, targets, result, assumptions, evidence, and limits.

TOOLPROTO-018 | Each finding must identify affected entities and revisions, including declared slots or relations when applicable.

TOOLPROTO-019 | Each evidence reference must identify retained bytes, covered scope, method, and input revisions.

TOOLPROTO-020 | The caller must reject evidence when referenced bytes are missing or their digest differs.

## Result meanings and receipt validation

| State | Exact meaning | Completion consequence |
| --- | --- | --- |
| `passed` | The supported method completed every condition in this unit without a detected violation. | Eligible for this method and scope only. |
| `failed` | The supported method identified a violated declared rule or conflicting canonical meaning. | Retain located findings. Repair or resolve the conflict. |
| `unsupported` | No registered method covers this check, feature, profile, or admitted structure. | Retain a capability gap and affected guarantees. |
| `indeterminate` | A supported method completed but could not decide the obligation under its declared assumptions. | Retain uncertainty and the undecided condition. |
| `execution-error` | The operation could not complete because of invalid input, unavailable artifacts, crash, timeout, cancellation, or limits. | No successful completion evidence for this invocation. |

Semantic uncertainty differs from malformed input. A missing required contract slot can produce `failed` from a valid request.

A schema-invalid request produces a dispatcher rejection receipt with code `invalid-request`. It does not become a fabricated checker response.

An unknown tool produces a capability-gap receipt. It does not impersonate the absent tool's identity or claim an invocation occurred.

Aggregate precedence is `execution-error`, `failed`, `unsupported`, `indeterminate`, then `passed`. All individual unit results remain visible.

An incomplete invocation has aggregate `execution-error`. Completed assessments remain diagnostic history until independently checked through a complete invocation.

Receipt states are `accepted`, `rejected`, and `stale`. Receipt reason codes include `invalid-request`, `invalid-response`, `identity-mismatch`, and `missing-evidence`.

The receipt retains raw request/response digests, expected identity, observed identity, validation reasons, and evidence eligibility.

TOOLPROTO-021 | The tool must use the five declared result states without substituting successful results for missing capability.

TOOLPROTO-022 | The tool must return `indeterminate` when its supported method cannot decide a semantic obligation.

TOOLPROTO-023 | The tool must return `unsupported` when no declared method covers a requested semantic guarantee.

TOOLPROTO-024 | The caller must record malformed requests as rejected receipts without invoking the checker.

TOOLPROTO-025 | The caller must reject malformed, mismatched, duplicate, or contradictory response records.

TOOLPROTO-026 | The caller must reject responses that omit requested units or report inconsistent coverage.

TOOLPROTO-027 | The caller must mark evidence stale when any required input identity differs from the current completion target.

TOOLPROTO-028 | The caller must preserve stale and rejected records without applying their findings as current evidence.

TOOLPROTO-029 | The tool must report aggregate `passed` only when every requested unit passed and execution completed.

TOOLPROTO-030 | The caller must not use partial results as successful completion evidence for the interrupted invocation.

TOOLPROTO-031 | The caller must not infer semantic completion or owner acceptance from an accepted syntax result.

## D10 proposal: bounded execution

These values are proposed operational bounds. They do not restrict valid contract meaning or permit omitted nested obligations.

| Category | Deterministic comparison | Proposed bounds | Cancellation and retry |
| --- | --- | --- | --- |
| Structure, slots, names | Same method, manifest, parameters, and descriptor produce equal assessments. | 8 MiB input, 4 MiB response, 10,000 units, traversal depth 128, 30 seconds. | Check cancellation before each unit and traversal step. No automatic retry. |
| Composition | Same rule set and graph produce equal assessments. | Same byte/unit/deadline bounds. At most 10,000 nodes and 10,000 edges. | Same cancellation checks. Supported bounded methods only. |
| Profile compatibility | Same profile/settings/mappings produce equal assessments. | Same common bounds. Exact registered profile revisions only. | Same cancellation checks. No substitute profile retry. |
| Case evaluation | Same finite cases and evaluator produce equal assessments. | Same common bounds. At most 10,000 supplied cases. | Same cancellation checks. No external implementation execution. |
| Revision impact | Same before/after manifests, lineage, and graph produce equal affected sets. | Same common bounds. At most 10,000 entities and 10,000 dependency edges. | Same cancellation checks. Missing lineage prevents a narrow safe-reuse claim. |

Caller bounds can reduce descriptor maxima. A tool-specific descriptor can use smaller maxima and explicitly unsupported methods.

Deterministic comparison excludes request identity, timestamps, elapsed time, operational logs, and artifact locations. Artifact digests and covered conclusions remain equal.

Diagnostics sort by target revision, entity ID, slot path, check ID, diagnostic code, then message, using ASCII ID order.

Messages use fixed templates with declared substitutions. Operational errors can vary and do not establish deterministic semantic disagreement.

The dispatcher enforces the deadline independently of cooperative tool checks. Cancellation records unfinished units and observed effects.

When output exceeds its limit, the dispatcher retains a bounded `limit-exceeded` receipt and rejects the truncated payload.

A timeout returns `execution-error` or a dispatcher-generated execution record if no valid response arrives. It never fabricates passed assessments.

Retry eligibility is limited to read-only transient operational failures. A caller can start one new invocation within its remaining chain budget.

That invocation uses a fresh request ID and links its predecessor. Unknown effects, invalid requests, semantic failures, and unsupported methods are ineligible.

The original invocation records attempt 0 and its total execution budget. A retry records attempt 1 under the same root request ID.

The caller subtracts all earlier attempt time from that fixed budget. Attempt 1 has no retry successor, even with a new request ID.

A retry preserves the original contract, descriptor, scope, and parameters. A changed checking target starts a separate analysis, not a transient-failure retry.

TOOLPROTO-032 | Each descriptor must define deterministic comparison fields, diagnostic ordering, bounds, deadlines, cancellation, and retry eligibility.

TOOLPROTO-033 | The dispatcher must enforce the smaller of caller limits and descriptor limits.

TOOLPROTO-034 | The tool must report unsupported traversal depth without omitting deeper obligations from the contract's required scope.

TOOLPROTO-035 | The dispatcher must classify timeout, cancellation, crash, and exceeded execution bounds as `execution-error`.

TOOLPROTO-036 | The dispatcher must reject truncated output instead of treating its missing suffix as an empty finding list.

TOOLPROTO-037 | The caller must preserve each failed attempt when a later invocation succeeds.

TOOLPROTO-038 | The caller may retry one eligible read-only operational failure with a fresh request identity and remaining budget.

TOOLPROTO-039 | The caller must not retry semantic failures or missing capabilities as transient operational failures.

TOOLPROTO-049 | The caller must preserve retry-chain identity, attempt count, and cumulative budget across successor requests.

TOOLPROTO-050 | The dispatcher must reject retry successors without a valid eligible predecessor or with exhausted chain attempts or budget.

## Mutation and repair boundaries

All proposed catalog entries are read-only. Current draft-generation commands remain outside this protocol. No generic patch tool is registered.

Any future mutating descriptor requires an explicit requirement and conceptual review before registration.

Its request names allowed target IDs and fields, expected old values, base revision, authorization reference, and requested changes.

Its descriptor defines admitted changes, preconditions, conflicts, atomicity, effects, and new-revision construction. An obsolete base rejects all changes.

TOOLPROTO-040 | The dispatcher must reject mutation fields for a read-only tool.

TOOLPROTO-041 | Each mutating descriptor must define allowed changes, preconditions, conflicts, atomic effects, and resulting revision identity.

TOOLPROTO-042 | A mutating tool must reject an obsolete base revision without applying any requested change.

TOOLPROTO-043 | A mutating tool must validate all requested changes before applying its declared atomic change set.

TOOLPROTO-044 | The caller must require explicit mutation authorization covering the declared change set.

The repair handoff contains the receipt, current revision, affected guarantees, located findings, supported-method limits, and unresolved decisions.

The LLM proposes a repair against that revision. It selects prepared structures instead of inventing schemas to satisfy a checker.

An owner-only choice returns to the owner with its decision ID. Missing capability remains a capability gap, even after prose changes.

The caller checks a new revision through new invocations. Reusing unaffected evidence requires an accepted T02 impact rule and verified dependencies.

TOOLPROTO-045 | The caller must send actionable findings and checking limits to the LLM with their exact base revision.

TOOLPROTO-046 | The LLM must separate proposed contract repairs, capability gaps, and owner-only decisions in its handoff.

TOOLPROTO-047 | The caller must check repaired revisions before using their evidence in the completion gate.

TOOLPROTO-048 | The caller must not remove a required guarantee merely because its checker is unavailable.

## Worked protocol fixtures

These are conceptual fixtures with expected outcomes, not executable tests or actual tool results.

Fixture symbols stand for complete retained values, not literal wire strings. `Q1` has a fresh request ID and exact request digest.

`R1` is an exact contract reference. `M1` is its complete retained manifest. `B1` is a hypothetical registered build and descriptor.

`U1` requests `explicit-exit-target` on `loop-2.exit_target`. `E1` retains the exact observed slot and method conclusion at R1.

B1 supports that one check under the proposed limits. No claim follows that B1 currently exists.

| Envelope | Fully assigned fixture fields |
| --- | --- |
| Valid request Q1 | Version `proposal-1`, unique request/digest, tool B1, contract R1/M1, units `[U1]`, all manifest artifacts, empty parameters, declared limits. Retry root Q1, attempt 0, fixed total/remaining budget, predecessor absent, mutation absent. |
| Valid failed response S1 | Exact Q1 identity/tool/contract echoes. U1 targets `loop-2.exit_target`, status `failed`, method `explicit-reference`, finding `missing-exit-target`, evidence E1, declared assumptions/limits. Coverage requests/completes U1, unfinished empty. Execution `completed`, aggregate `failed`. |
| Finding in S1 | Entity `loop-2`, revision R1, kind `pattern-instance`, slot path `[exit_target]`, code `missing-exit-target`, message `The nested loop has no explicit exit target.`, evidence E1. |
| Evidence E1 | Retained artifact with actual byte digest/count, media type, U1/R1 linkage, declared method and assumptions. It records the absent target, without selecting a target. |

The reviewer checks those symbolic bindings as explicit fixture assumptions. Production payloads require concrete resolvable references and valid digests.

The following fixtures replace only named fields in Q1/S1. Other fields retain the base assignments above. Every replacement includes consistent aggregate/coverage fields.

| Case | Input and changed fields | Expected observable and repair consequence |
| --- | --- | --- |
| TP-C01 | Q1/S1: nested loop has no explicit exit target. | Accepted failed assessment identifies `loop-2.exit_target`. LLM proposes a target only from contract authority. |
| TP-C02 | R2 explicitly supplies target `loop-outer`. New Q2 and matching S2 report passed U1 with R2 evidence. | Accepted `passed` for U1 only. No other check or owner gate passes by implication. |
| TP-C03a | Q3 requests profile guarantee G1 without any registered operation admitting that request. | Preflight capability-gap receipt. No invocation or tool result is claimed. G1 remains unresolved. |
| TP-C03b | Registered `profile.check` admits profile feature descriptions. Q3 supplies feature F1 outside its declared semantic coverage. | Valid invocation returns `unsupported` for F1 and affected guarantee G1. The descriptor's admitted operation does not promise F1 support. |
| TP-C04 | B1 supports a bounded constraint method, which cannot decide G2. Its complete assessment reports the undecided predicate. | Accepted `indeterminate`, with method reason and G2 reference. No parser-error substitution. |
| TP-C05 | Tool times out before U1 completes. Dispatcher records elapsed budget, U1 unfinished, aggregate execution-error. | No completion evidence. Retain a checkpoint and operational failure. |
| TP-C06 | Caller currently targets R2. S1 still echoes Q1/R1 correctly. | Receipt `stale`, even though S1 is internally well formed. Request new analysis. |
| TP-C07 | Same semantic input repeats under Q4 with a fresh ID. B1 returns the same conclusion, diagnostic order, and evidence digest. | Determinism passes after excluding permitted execution metadata. Contradictory conclusions fail comparison. |
| TP-C08 | Q1 omits its contract revision or contains duplicate JSON keys. | Dispatcher rejects `invalid-request` before analysis or changes. |
| TP-C09 | S1 echoes another request ID, build digest, descriptor, or manifest. | Caller rejects `identity-mismatch`. Findings cannot repair the current revision automatically. |
| TP-C10 | Response claims passed but omits U1 or supplies duplicate U1 assessments. | Caller rejects `invalid-response`. Empty findings cannot conceal incomplete coverage. |
| TP-C11 | U1 finishes, U2 times out. Response covers both but aggregate reports passed. | Caller rejects contradictory status. A corrected interrupted response has `execution-error`, with both unit records retained. |
| TP-C12 | Declared E1 digest differs from retained bytes, or E1 cannot be retrieved. | Caller rejects `missing-evidence`. No successful evidence eligibility. |
| TP-C13 | Output ends inside its diagnostics array, or exceeds its byte limit. | Dispatcher retains truncated/limit failure. Caller rejects the payload without inferring absent findings. |
| TP-C14 | Future mutating request targets obsolete R1 while current revision is R2. | Reject all changes. No partial patch or new revision. Capability remains unavailable today. |
| TP-C15 | Discovery lacks a descriptor for guarantee G3. | Record capability gap before invocation. No synthetic version, tool pass, or lowered guarantee. |
| TP-C16 | Tool crashes or caller cancels before completion. | Retain `execution-error`, unfinished units, and known effects. Read-only contract bytes remain unchanged. |
| TP-C17 | Contract depth exceeds supported method depth, with ample execution budget. | Report `unsupported` for the deeper required scope. Exhausted time instead reports execution-error. |
| TP-C18 | Syntax checker returns no diagnostics. | Declared syntax can pass. Semantic review and owner acceptance remain separate. Current CLI lacks this protocol envelope. |
| TP-C19 | LLM proposes repairs or an owner chooses a missing value. | Preserve separate interpretation/decision records. New contract revision requires new checking. |
| TP-C20 | Q5 retries eligible timeout Q1 with root Q1, attempt 1, valid predecessor receipt, and reduced remaining budget. | Accept retry within the chain budget. Reject another successor, missing predecessor, reset attempt counter, or increased remaining budget. |

### Requirement and case mapping

| Task obligation | Protocol clauses | Conceptual cases |
| --- | --- | --- |
| T06-001 | TOOLPROTO-002–TOOLPROTO-004, TOOLPROTO-012–TOOLPROTO-015 | TP-C01, TP-C06, TP-C08, TP-C09 |
| T06-002 | TOOLPROTO-015–TOOLPROTO-020 | TP-C01, TP-C02, TP-C12 |
| T06-003 | TOOLPROTO-018, TOOLPROTO-045 | TP-C01, TP-C03b, TP-C04 |
| T06-004 | TOOLPROTO-021–TOOLPROTO-023, TOOLPROTO-029 | TP-C01, TP-C02, TP-C03b, TP-C04, TP-C05 |
| T06-005 | TOOLPROTO-011, TOOLPROTO-027, TOOLPROTO-028 | TP-C06, TP-C09 |
| T06-006 | TOOLPROTO-007, TOOLPROTO-025–TOOLPROTO-031, TOOLPROTO-035, TOOLPROTO-036 | TP-C03a, TP-C03b, TP-C04, TP-C05, TP-C10–TP-C13, TP-C15–TP-C18 |
| T06-007 | TOOLPROTO-040–TOOLPROTO-044 | TP-C14 |
| T06-008 | TOOLPROTO-012–TOOLPROTO-014, TOOLPROTO-024 | TP-C08, TP-C14 |
| T06-009 | TOOLPROTO-001, TOOLPROTO-031, TOOLPROTO-045–TOOLPROTO-047 | TP-C18, TP-C19 |
| T06-010 | TOOLPROTO-006, TOOLPROTO-007, TOOLPROTO-010, TOOLPROTO-048. Family allocation table. | TP-C03a, TP-C03b, TP-C04, TP-C15, TP-C17 |
| T06-011 | TOOLPROTO-008–TOOLPROTO-010. Catalog and prototype maturity statements. | TP-C15, TP-C18 |

D09 maps to the envelope tables and TP-C08–TP-C10. D10 maps to TOOLPROTO-032–TOOLPROTO-039, TOOLPROTO-049–TOOLPROTO-050, and TP-C05, TP-C07, TP-C13, TP-C16, TP-C17, TP-C20.

Review outcomes remain unrecorded. These expected outcomes are design material for `the designer`, not self-acceptance evidence.

## Remaining gates and handoff

T02–T05 acceptance supplies actual references, pattern checks, relation rules, profiles, and their revisions before T06 closure.

D09 and D10 require recorded owner decisions. Exact payload schemas and descriptor registrations remain provisional until those decisions resolve.

T07 consumes the repair handoff and capability discovery. T11 consumes receipt validation, evidence freshness, and interruption behavior.

T13 owns the Rust architecture, adapter design, runtime implementation boundaries, and implementation evidence. None is supplied by this design draft.

## See also

- [CONTRACT.md](../../CONTRACT.md): method rules.
- [completion-controller.md](completion-controller.md): completion loop.
- [rust-architecture.md](rust-architecture.md): Rust implementation.
