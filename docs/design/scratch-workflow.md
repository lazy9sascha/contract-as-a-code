# Scratch authoring from intent

Revision: scratch-draft-7. Status: provisional design. Designer: `the designer`. Reviewer: `the reviewer`.

This document answers T08 revision 2, SHA-256 `16361a69fdd678bc9caa5343eeb5b26a57fd0bedd82d00ee7e93646ae1b6d5a9`.
It proposes D13. Upstream acceptance and owner decision evidence remain absent.

Scratch creates a populated contract for a new behavior boundary. It supplies one entry into T11's shared completion loop.
It does not create implementation code, an invented baseline, or its own completion gate.

## Provisional inputs

| Input | Inspected identity and limit |
| --- | --- |
| T02 model | `model-draft-2`, SHA-256 `f307de558f9ae90038d376bc8b77f0f9fd3d0609c5f60d6fe0af5f21c91cf7e6`. Typed cells, revisions, and evidence. |
| T03 interface and catalog | `patterns-draft-5`, bundle manifest SHA-256 `83c5bd9f2b57d932c48d90b3c70d8f2773c6c7e662b19e7988c2efbb18843b4a`. Corrected EntityLocation and explicit predecessor lineage. Acceptance absent. |
| T04 composition | Retained `composition-draft-5`, SHA-256 `2bd19c03679b25bb91146d58a392f2a40cfacb2d56e9c80f83a2e40bb8cf19d8`. Semantic rules unchanged from reviewed draft4; catalog identity refreshed. |
| T07 skill | Retained `skill-draft-2`, SHA-256 `69199f3410755d810f507e93d5a4f49ec9119b8cf3efc8aa28b0c500dd46a9ae`. Context, questions, proposals, and exact catalog handoff. |
| T11 task | Shared repetition, evidence freshness, and completion responsibilities. The controller design remains a future interface input. |

Catalog observation is not accepted library support. Missing or changed definitions remain explicit capability or revision findings.
The input hashes record drafting history. T14's final baseline and compatibility map select the revised model/catalog/composition phase semantics without circular embedded hashes.
T03/T04/T07 are input, design-output, and close-gate dependencies. T11 is an interface handoff, without a reverse close gate.

## D13 proposal and input contract

Proposed minimum input: one nonempty behavior request, its retained source, and an identifiable candidate boundary.
Actors, observables, and policies can remain partly unknown while known source values populate a draft.
A missing boundary becomes a targeted scope question. It prevents implementation assignment, not all independent authoring.
No implementation files or selected programming language are required to start.

The skill consumes these conceptual fields. Their caller supplies exact source and definition references.

| Field | Type and cardinality | Missing or empty rule |
| --- | --- | --- |
| request | Retained source reference plus NonemptyText, 1 | Missing means no source-grounded behavior to draft. Return input-needed. |
| boundary | T02 scope or explicit proposed scope, 1 | Unknown records a scope decision. No guessed accepted boundary. |
| actors | Set of actor definitions, 0..n | Unknown responsibility remains a decision. Empty grants no implicit actor authority. |
| known | Set of source-linked bindings and constraints, 0..n | Preserve exact known values, including zero, false, and empty. |
| implementation | none-in-boundary, present-in-boundary, or unknown, 1 | Unknown does not prove absence. No baseline is fabricated. |
| mode | Explicit scratch selection or T07 routing report, 1 | Conflicting source coverage returns a mode-boundary finding. |
| library | Exact available pattern definitions and their limits, 0..n | Empty leaves required pattern capabilities unresolved. |
| profile | Exact profile or explicit language-neutral status, 1 | No language mapping follows from an omitted preference. |
| context | T07 context identity and assignment, 1 | Missing required action context returns context-missing. |

The host owns retained input records. The skill owns the proposed interpretation and binding provenance.
Owner acceptance of a generated requirement is a separate record, never an inferred input default.

## Ordered workflow

1. Resolve the request and existing applicable decisions. Separate quoted source from execution authority.
2. Record the selected behavior boundary, actors, observables, and exclusions. Retain any unknown scope as a decision.
3. Check whether implementation exists inside that boundary. Reconcile scratch selection with the owner's actual adoption scope.
4. Select exact prepared definitions that express known behavior. Record capability gaps instead of inventing schemas.
5. Populate known slot values and runtime input mappings. Link each to its source, definition revision, and requiredness.
6. Construct typed relations and enumerate obligations introduced by nesting, calls, failures, and effects.
7. Create cases for known branches, boundaries, failures, and unresolved outcomes. Keep unknown expected results explicit.
8. Return the populated initial revision, findings, decisions, cases, and checking needs to T11.

Each step preserves known material when another step needs an owner answer or missing capability.
Question delivery, resumption, budgets, repetition, and the final gate remain T11 responsibilities.

SCRATCH-001 | The skill must establish the selected boundary before assigning implementation responsibilities.

SCRATCH-002 | The skill must select exact prepared definitions for behavior that its sources require.

SCRATCH-003 | The skill must populate every source-determined slot with its value and provenance.

SCRATCH-004 | The skill must record missing behavior-affecting values as decisions with affected slots and consequences.

SCRATCH-005 | The skill must derive initial cases for declared outcomes, branches, boundaries, and failures.

SCRATCH-006 | The skill must not create an implementation baseline when no implementation source belongs to the selected boundary.

SCRATCH-007 | The skill must not add speculative features to make an unfinished contract appear complete.

SCRATCH-008 | The skill must hand its proposed revision and unfinished obligations to the shared controller.

## Output and question contract

The output is a T07 handoff with these mode-specific contents. All fields occur once unless stated otherwise.

| Field | Required content |
| --- | --- |
| base | Explicit initial/no-existing-package marker or exact current package revision. |
| proposal | Initial package draft, selected scope, actors, terms, typed slots, clauses, pattern instances, and relations. |
| source_map | Source passage and authority classification for every known requirement and proposed interpretation. |
| unknowns | Decision IDs, owner, question, affected slots, and observable consequences. Empty means none recorded, not implicit acceptance. |
| gaps | Required unsupported patterns, profile mappings, or checks and their affected guarantees. |
| cases | Inputs, applicable clauses, expected observable or explicit unresolved decision, and boundary/failure purpose. |
| baseline | Explicit not-applicable for scratch. An implementation-status uncertainty is a separate finding. |
| checks | Required T06 check units and target references. Unavailable tools remain gaps. |
| controller_handoff | Context identity, proposed revision, open obligations, and requested next assessment under T11. |

A question names the exact missing decision. It preserves known values and explains which observable cannot yet be derived.
The skill reuses applicable recorded answers. It does not ask for an already known input or repeated authorization.
Unknown owner requirements differ from routine proposed contract structure. A structural proposal can proceed with its proposal status explicit.

If code exists outside the boundary, scratch still applies to the new behavior.
If code exists inside it, the skill reports the mismatch and preserves the owner's selected adoption scope for plugin or migration.
It does not silently inspect unrelated files or relabel existing behavior as new.
An explicitly abstract draft can retain extension points. A concrete target needs their required bindings or a blocking finding.

## Populated normalization fixture

Fixture source N-SOURCE is hypothetical: accept Text. Empty input returns Error(EmptyRecipient).
Nonempty input returns Ok with exactly the input's codepoints. Non-Text input returns InvalidInput. Produce no external effects.
Only this behavior belongs to the fixture. It does not define a delivery service or an owner-approved universal normalization rule.

Text is a finite Unicode codepoint sequence. Empty means length zero. Text equality compares codepoints in order.
Normalizer is the responsible actor. Admitted input is Text.
Exact type NResult is Variant{Ok:Text,Error:EmptyRecipientType}. EmptyRecipientType has one EmptyRecipient tag with Unit payload.
InvalidInput is a separate pre-body type-rejection observable under FN-OPERATION.rejection. It is not a third NResult alternative.

| Instance and prepared definition | Populated bindings |
| --- | --- |
| N-operation: FN-OPERATION p2 | actor=Normalizer, inputs=Text, result=exact NResult reference, requires=true, ensures=the source relation, effects=[], failures=exact EmptyRecipientType reference, body=N-branch, continuation=terminating, termination=one finite branch, rejection=InvalidInput. |
| N-branch: FLOW-BRANCH p1 | condition=input length equals zero, then=N-empty, otherwise=child, else=N-preserve. |
| N-empty: FLOW-EXIT p2 | kind=return, target=N-operation's static activation selector, value.type=exact NResult reference, value=Error(EmptyRecipient(unit)). |
| N-preserve: FLOW-EXIT p2 | kind=return, target=N-operation's static activation selector. Static value key is absent; S-return is its sole typed runtime supplier. |
| N-input: EXP-EVAL p3 | kind=name, resultType=Text, order=left-to-right, binding={entity:R-N-body exact relation revision,slotPath:[bindings,context,parameters,input]}. This location selects the declared FormalParameter below. Disabled conditional slots are NotApplicable. |
| N-ok: VAL-VARIANT p2 | alternatives=exact NResult alternative/type references, tag=Ok, optional-form=false, nullable-form=false. Static payload key is absent; S-payload supplies it. Its output has exact NResult type. |

The contains relations name body, then, and else child roles. The exit context maps both returns to N-operation's active invocation.
R-N-body is the contains Relation from N-operation to N-branch, with role=body and position=0.
Its Known ContextBinding record contains a FormalParameter named input, type Text, source kind argument, operation N-operation, port argument, argumentPath=[].
Source activationPath=[parent], mapping=identity Text, phase=entry, access=immutable, and lifetime owner=N-branch with activationPath=[].
These are static declaration fields. They contain neither a future Text value nor Known Expression<Text> stored as Text.
At N2 entry the runtime frame materializes Text "A". At N1 entry it materializes empty Text. N3 rejects before body entry.
The static relation declares the parameter location. T04's exact FrameKey environment holds its invocation-specific TypedValue<Text> after validation.
No input key is added to FN-OPERATION's declared pattern slots. The inputs slot still names the signature's input type.
S-payload is a binds-supplier from N-input's result RuntimePort to DataSlot{N-ok,payload}, mapping identity Text, before constructor consumption.
S-return is a binds-supplier from N-ok's result RuntimePort to DataSlot{N-preserve,value}, mapping identity NResult, before return consumption.
Each has exact from/to instance references, selected-else applicability, and rejection before dependent effects on invalid production.
Their sourceContext captures input's declaration at its owning N-branch activation. Relative consumer paths are [parent,parent] for S-payload and [parent] for S-return.
The provider frames inherit only those explicit immutable captures. They do not evaluate the unselected empty-input branch's suppliers.
These are known source/mapping declarations, not owner Unknowns or Known future runtime values. Static target cells remain absent.
N-preserve.value retains N-ok's exact NResult type. No implicit variant widening or type conversion occurs along these edges.
Input type rejection occurs before body evaluation or effects. It does not pass InvalidInput through an NResult return binding.

- N1: Input empty Text produces Error(EmptyRecipient), with no external effects.
- N2: Input "A" produces Ok("A"), preserving codepoints, with no external effects.
- N3: Non-Text input produces InvalidInput before any body action.

N1–N3 are illustrative fixture relations, not additional normative clause IDs or executed test records.
Initial questions are empty because the hypothetical source specifies these observables. Owner acceptance and actual tool evidence remain absent.

## Populated notification fixture with an unresolved policy

Hypothetical source M-SOURCE: process a finite list of recipients in ascending index order and notify each recipient.
Notification can succeed or return TemporaryFailure. The request does not define retries, duplicates, or failure continuation.

| Instance and prepared definition | Known binding | Unresolved binding or edge obligation |
| --- | --- | --- |
| M-loop: FLOW-LOOP p3 | kind=collection, continuation=bounded, order=ascending, body=M-notify, termination=remaining count reaches zero. A declared argument-port supplier supplies items; no static list cell is invented. | itemType=Unknown D-RECIPIENT, failure=Unknown D-CONTINUE, resultPolicy=Unknown D-RESULTS. |
| M-notify: FN-OPERATION p2 | actor=Notifier, inputs=one recipient, tagged Success/TemporaryFailure results, declared notification effect | Concrete notification operation and its effect/result guarantees remain Unknown D-NOTIFY. |
| M-retry candidate: FAIL-RETRY p1 | Structure is available for the retry decision | D-RETRY determines whether retry exists. retryCount, eligible, delay, deadline, deadlineOutcome, exhaustion, and duplicatePolicy remain unresolved if selected. |

M-notify's requires, ensures, effects, body, continuation, conditional continuation policy, and rejection slots remain explicit Unknown targets of D-NOTIFY.
Its failures slot records the source's TemporaryFailure tag. The exact recipient input type remains Unknown D-RECIPIENT.
These missing operation details do not erase the source-backed loop order or known result alternatives.

M-retry is a proposed alternative structure, not an accepted extra retry requirement.
Known loop bindings survive each question. Zero recipients produces no notification attempts under the loop's source-backed empty behavior.
One TemporaryFailure leaves continuation and retry outcomes unresolved. The skill does not choose no-retry or three attempts.
A retry selection creates identity and duplicate-effect questions before a composed contract can complete.

The handoff contains D-CONTINUE, D-RESULTS, D-NOTIFY, D-RECIPIENT, and D-RETRY with exact target paths, initial cases, and required composition checks.
An owner answer of retryCount=0 remains a meaningful explicit choice if retry semantics count additional attempts.
The workflow preserves that answer and resumes affected work through T11.

## Review cases and handoff

| Case | Input | Expected conceptual result | T08 coverage |
| --- | --- | --- | --- |
| SC01 | N-SOURCE with explicit empty-input behavior | Populated N-operation, branch, values, returns, bindings, and N1–N3. No delivery subsystem. | T08-001/002/003/005/007. |
| SC02 | M-SOURCE with unspecified retry behavior | Known loop structure plus precise D-RETRY and D-CONTINUE. No invented retry count. | T08-002/003/004/005/007. |
| SC03 | Concrete target names an abstract interface without operation binding | Preserve declared interface and create a blocking binding finding. No invented implementation body. | T08-004/007. |
| SC04 | Request provides no existing code | baseline=not-applicable, no synthetic source file or digest. | T08-006. |
| SC05 | Existing code lies outside a newly selected boundary | Continue scratch only inside the new boundary. Preserve outside-scope status. | T08-001/006. |
| SC06 | Existing code lies inside selected scratch boundary | Return a mode mismatch and source-linked adoption choice. Do not fabricate absence. | T08-001/006. |
| SC07 | A required normalization pattern or method is unavailable | Preserve populated known behavior and an exact capability gap. No guessed semantics or successful check. | T08-002/004/008. |
| SC08 | A source contradicts itself about empty input | Record both observables and a decision. Do not select the easier branch. | T08-004/007. |
| SC09 | Applicable recorded owner answer resolves retryCount=0 | Reuse the answer and preserve other bindings. No repeated question or implicit absence conversion. | T08-003/004/008. |
| SC10 | Initial draft reaches the controller with missing decisions | Handoff includes proposed revision, questions, cases, findings, check needs, and scratch baseline status. T11 owns next state and gate. | T08-008. |

The designer derived these outcomes. Independent review remains required against the exact retained artifact.
D13, accepted T03/T04/T07 revisions, catalog reconciliation, and the final controller interface remain unresolved acceptance work.
Contract-authoring completion does not require a target program to terminate globally. This finite-list fixture does not impose a universal program deadline.
No user contract, runtime host, or conceptual release is marked complete by this workflow design.

## See also

- [plugin-adoption.md](plugin-adoption.md): plugin adoption.
- [migration-workflow.md](migration-workflow.md): migration workflow.
- [CONTRACT.md](../../CONTRACT.md): method rules.
