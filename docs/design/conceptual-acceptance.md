# Integrated conceptual baseline

Revision: acceptance-draft-1. Status: review candidate, not owner-accepted.
Designer: `the designer`. Independent reviewer: `the reviewer`.

This document answers T14 r2, SHA-256 `d304cdca501ffb8fb60e3a6b255fabe9bc27fa50452f92f20ea10bcc259b8127`.
It integrates the thirteen product designs and proposes D27–D30.
It does not implement the product, grant live authority, or claim conceptual 0.1.0 acceptance.

## Exact baseline and two gate results

The baseline manifest identifies all selected artifact bytes, including this document and the fixture data.
It excludes itself and mutable event/review records. The coordinator and reviewer record its digest separately.
Changing any selected artifact creates a new baseline and requires affected review. A link to a file does not mean its future latest revision.
The compatibility map selects governing revisions and records substantive phase changes separately from historical drafting inputs and editorial repairs.

The actual conceptual gate remains unaccepted while required upstream closure records, owner decisions, or exact baseline acceptance are absent.
The simulated user-contract gate can pass inside an explicitly hypothetical fixture. That result does not close a repository ticket.

| Evidence class | Current meaning |
| --- | --- |
| Conceptual design | Defined model, patterns, composition, profiles, tools, skill, modes, controller, action boundary, and architecture. |
| Independent content review | Exact artifact and case inspection by a different agent. It does not supply owner acceptance. |
| Fixture assessment | Explicit model input and independently derived expected outcome. It is not a real tool invocation or runtime test. |
| Existing Rust prototype | Bounded scaffold, syntax checker, and batch implementation source/tests. Their actual executed results remain separately recorded. |
| Runtime capability | Full semantic tools, controller persistence, installed skill, and host interception remain unimplemented or unavailable. |

ACCEPT-001 | The conceptual gate must require exact reviewed T01–T13 outputs and their recorded close-gate evidence.

ACCEPT-002 | The conceptual gate must reject missing meaning, stale required evidence, unhandled gaps, failed cases, or missing required owner acceptance.

ACCEPT-003 | The reviewer must distinguish conceptual case inspection from tool execution, implementation tests, and publication.

## Integrated responsibility and interface map

| Stage | Actor | Canonical interface and observable | Owning design |
| --- | --- | --- | --- |
| Intent and authority | Owner and skill | Exact source, selected boundary, applicable decisions, proposal/accepted/observed distinction. | T01 concept and T07 skill. |
| Package construction | Skill under controller assignment | T02 typed entities, static cells, runtime-source relations, exact revisions, initial cases. | T02 model; T08 scratch or T09/T10 modes. |
| Pattern selection | Skill | Exact definition revisions, applicable required slots, declared evaluated-data phases, unused capabilities retained. | T03 catalog. |
| Composition | Author and checker | Typed endpoints, FormalParameter declarations, FrameKey instantiation, order/exit/effect/substitution obligations. | T04 composition. |
| Language interpretation | Profile author and checker | Exact version/settings, feature mapping, counterexample or located gap. No profile substitution. | T05 profiles. |
| Checking | Controller, dispatcher, tool | T06 exact request, descriptor/build/input identity, five result states, validated receipt and scoped evidence. | T06 protocol. |
| Repair and questions | Skill and owner | T07 revision-bound proposal or exact decision-linked question; preserve known values. | T07 skill. |
| Repetition and gate | Contract controller | Guarded states, journal/checkpoint, progress/budget ledger, current criterion evidence, immutable completion object. | T11 controller. |
| Implementation handoff | Contract and implementation harnesses | Completion/member map, clause or located Choice basis, typed action, interception tuple, current runtime authority. | T12 boundary. |
| Host/core execution | Rust core and host adapter | Typed requests/results/control events, atomic revision ownership, durable resume, explicit unsupported outcomes. | T13 architecture. |

Scratch has no implementation baseline. Adoption inventories selected existing responsibilities without converting observations into requirements.
Migration separates target contract, transition contract, readiness, and execution authorization.
A multi-contract ModeSession requires every required member complete. A complete target cannot bypass an incomplete migration transition.

ACCEPT-004 | The reviewer must trace each integrated case through source, pattern, slots, composition, checking, repair, gate, and action handoff.

## Concrete fixture boundary

The fixture source declares a pure finite batch operation.
The input has zero through two pairs, numerator in {-7,0,7}, and denominator in {0,1,2}.
Validation precedes processing. Zero denominator returns a per-item domain error and permits later items. Nonzero division truncates toward zero.
Output order and duplicate items are preserved. No external effect belongs to that operation.

These are mock source choices for a conceptual fixture, not accepted values for the real batch example or the owner's payment scenario.
The finite table contains all 91 admitted lists and four explicit invalid-input cases.
Its nine per-pair result entries come from the explicit mathematical relation, not the Rust implementation.
The data are generated authoring artifacts. Their existence does not establish that a semantic tool or compiler executed them.

The fixture profile names FixtureMath version 1, exact mathematical integers, sequential evaluation, and no external effects.
It tests T05's profile interface. It does not fill the unresolved Rust 1.85 archive or real host assumptions.
Real ECMAScript Number division still differs from integer truncation. Its missing common counterpart remains an explicit gap.

### Static meaning and runtime input

The package records constant configuration separately from declared runtime data sources.
FormalParameter input declares its source as the operation's exact argument port and whole-argument path.
A typed binds-supplier relation supplies Loop.items before consumption. It carries a known mapping, not a Known expression masquerading as a list.
At invocation, [] or [Pair(-7,2)] becomes the actual typed runtime value in that frame.
Neither future caller value is an unresolved owner requirement.

Declaration/configuration/policy slots stay authoring-resolved. An unknown retry policy cannot be relabeled as caller data to evade a decision.
EntityLocation identifies declared data. RuntimePort identifies a signature/role value. A configured result Type is not a runtime output field.

Each package contains readable graph views and an explicit expandedPackage with full entity, Binding, TypedValue, and case records.
The exact fixture-codec defines structural metadata refinements, closed trace relations, external profile resolution, and projection rules.
Constant bindings contain actual TypedValues. Runtime supplier targets have no competing static cell.
The codec is fixture interpretation data, not an installed parser or a new accepted product storage encoding.

## Positive repair and handoff trace

The machine-readable trace supplies exact artifact references and the records summarized here.
Every tool build, owner answer, review, grant, and enforcement result in that trace is explicitly hypothetical.

| Step | Exact fixture record and state | Derived observable |
| --- | --- | --- |
| 1 | Source F-SOURCE → scratch context C0 and package R0. FN-OPERATION, FLOW-LOOP, FLOW-BRANCH, FAIL-RESULT, and EXP-EVAL are selected by exact revision. | Known boundary, types, parameter declarations, slot suppliers, cases, and source provenance exist. |
| 2 | R0 divide.divisionRule contains invalid enum value toward-zero. T06 request Q0 checks that target under a fixture descriptor. | Response failed, located slot finding. A valid accepted receipt is not a passed check. |
| 3 | T07 assignment A-repair consumes R0, current finding, and F-SOURCE. Proposal names exact R0 base and the corrected truncate-zero value. | One source-grounded repair creates R1. No owner answer or requirement value is invented. |
| 4 | New current checks cover R1 slots, parameter/supplier names, composition, profile relation, and finite cases. | Fixture responses pass only their declared coverage. Old R0 failure remains historical. |
| 5 | Checkpoint CP1 retains context, exact R1, applied proposal, decisions, invocations, consumed/reserved budget, and progress state. | Resumption cannot apply the repair twice or recreate an unused budget. |
| 6 | Independent mock semantic/case review and exact mock FixtureOwner acceptance target R1. All fixture gate criteria are current. | Simulated checking→complete produces CO1 for R1/profile/policy. Actual product and real user contracts remain unaccepted. |
| 7 | T12 binding resolves a declared located Choice basis, exact CO1, scoped file target, fixture enforcement tuple, and current mock grant. | Eligible bounded mock handoff. No real tool, file write, runtime permission, or implementation result is asserted. |

The fixture's T06 request includes protocol_version, request_id, exact tool and contract, check_units, inputs, parameters, limits, and retry_chain.
The caller hashes the transmitted bytes. Responses echo that identity, account for every unit, and include status, coverage, execution, diagnostics, and evidence.
Validated receipt state and assessment status remain separate. Unknown or unsupported required units cannot be hidden by an aggregate pass.

The implementation choice is a Known Choice<T> at a declared data location, not an invented standalone Choice entity.
The action basis names that location and its completed revision. Runtime authority remains a separate current host record.

ACCEPT-005 | The reviewer must reject fixture evidence that violates the declared model, type, phase, revision, or protocol interface.

## Integrated negative cases

Each case changes the stated record in the positive fixture or the named nested scenario.
The reviewer checks the resulting path independently. A prose promise of safety does not replace the located data or guard.

| Case | Concrete mutation | Required trace and gate result |
| --- | --- | --- |
| IG01: missing source | Remove the sole Loop.items supplier while leaving its static cell absent. | slots/source check fails; T07 retains finding; T11 cannot complete. Future input is not guessed as []. |
| IG02: wrong phase/type | Store Expression<List> as a Known List, supply before its producer, or add a competing constant. | Type/availability/double-source finding. No false materialized value. |
| IG03: stale | Send R1's gate an assessment or repair bound to R0. | Receipt/evidence is stale or proposal is rejected before application. Preserve historical result. |
| IG04: unsupported | Select a profile/check without declared required coverage. | Located unsupported result or capability-gap receipt; capability-blocked after independent useful work ends. |
| IG05: timeout | Replace a complete tool response with timeout/uncompleted units. | execution-error, checkpoint/reservation retained. Only exact eligible T06 retry rules apply. |
| IG06: nested cancellation | Payment success arrives after the outer cancellation acknowledgment under the explicit H4 policy. | Retain Captured(payment-ID), stop new attempts/shipment, join in-flight work, return CancelledWithCapture. Unknown owner policy variant remains blocked. |
| IG07: duplicate effect | Two retry attempts use different payment identities and a finite witness contains two captures against an at-most-one guarantee. | Failed composition/effect check with exact witness. A new run or request ID cannot erase the duplicate effect or discharge the guarantee. |
| IG08: unfinished owner meaning | O-ELIGIBILITY, O-ATTEMPTS, O-PAYMENT-ID, O-SHIP-FAILURE, or O-COMPENSATION remains Unknown. | Preserve the order/payment/shipment structure and known facts. User-contract gate remains incomplete without inventing real business values. |
| IG09: denied runtime authority | CO1 and binding are valid inside the fixture, but the mock grant is removed or revoked before admission. | T12 denies before a protected call. Historical contract completion remains separate. |
| IG10: bypass | A required delegated process can evade the declared interception tuple. | Unsupported delegation, no enforcement claim outside the evidenced boundary. |
| IG11: ongoing program | Use T03's explicit ongoing service policy, then require the authoring gate to wait for global target termination. | Reject that added requirement. Defined active/observation/stop behavior can be contract-complete; unsupported required liveness proof still blocks its own criterion. |
| IG12: target only | Migration target member is complete while the required transition member has an unresolved recovery decision. | Preserve target-complete and aggregate-incomplete. No implementation/cutover bypass through the target or a runtime grant. |
| IG13: missing design outcome | Remove the rule handling a required unsupported check from the integrated design. | Conceptual gate rejects the design itself, even though a different design could validly specify a blocked user-contract outcome. |
| IG14: obsolete authority | An older overview calls the syntax checker the primary product without a reconciled T01 entry. | Follow current owner direction and reject baseline acceptance until authority reconciliation exists. |
| IG15: false implementation proof | Supply passing Rust/Python tests for a missing conceptual case or real semantic tool. | Record only their actual scope. They cannot establish missing conceptual meaning or future runtime capability. |

The nested scenario preserves the owner's unknown values. Its separate filled H1/H4 policies are hypothetical examples, not answers to those questions.
The explicit unsupported-result path can be conceptually complete. An absent rule for that path cannot.

ACCEPT-006 | The reviewer must evaluate negative cases independently against exact fixture records and owning design guards.

ACCEPT-007 | The gate must distinguish a defined blocked user-contract outcome from an undefined product-design outcome.

## Consolidated decisions and source authority

One exact baseline acceptance can accept the bundled technical proposals. Thirty separate permission questions are unnecessary.
The following register preserves decision IDs and owners. Proposed designs remain proposals until that acceptance is recorded.

| Decisions | Proposed disposition | Authority class and remaining consequence |
| --- | --- | --- |
| D01–D02 | Contract-completion product above implementation; this concept/method/design responsibility map; conceptual versus implementation release separation. | Product-boundary acceptance. Preserve B05–B09/B24–B27. No return to a checker-only product. |
| D03–D04 | Combined revisioned semantic package, explicit constant/runtime-source distinction, no conflicting representation precedence, stable identity with lineage. | Technical design proposal, reviewed through model and integration cases. |
| D05 | Thirty-four prepared counterparts with explicit discovery and unsupported gaps. | Baseline scope acceptance. Preserve the broad all-pattern direction beyond this finite initial inventory. |
| D06–D07 | Typed relations, exact activation/phase/port semantics, explicit continuation, conjunctive inheritance, behavioral substitution. | Technical semantics proposal. No universal proof or silent depth truncation. |
| D08 | Accept only the explicitly scoped conceptual profile mappings and gap outcomes in T05. Rust 1.85 remains source-pending; real host/runtime support remains unavailable. | Coverage/scope acceptance. No full-language or supported-Rust claim. Additional selected capabilities require evidence. |
| D09–D10 | Exact read-only protocol, five result states, receipt validation, bounded descriptors and preserved retry chains. | Technical interface proposal. No installed semantic checker follows. |
| D11–D12 | Revisioned skill context, exact decisions/proposals, explicit mode routing and uncertainty. | Technical host/skill proposal. Quoted source never becomes authority. |
| D13–D15 | Populated scratch, bounded adoption, separate migration target/transition/readiness/authorization. | Workflow proposals. Real source/business decisions remain source-linked and unresolved where genuinely missing. |
| D16–D18 | Guarded controller, typed control ingress, shared-budget multi-contract sessions, exact evidence gate, durable journal/resume requirements. | Technical controller proposal. Caller-supplied budgets have no invented defaults. Ongoing target behavior remains distinct. |
| D19–D22 | Completion/member-bound actions, located Choice basis, evidenced interception tuple, reusable current authorization, retained effect identities across runs. | Technical action-boundary proposal. Actual enforcement and grants remain host-specific and unavailable here. |
| D23–D25 | Rust core owns state and guarded transitions; host adapters carry typed skill/tool/control interactions and durable records. | Technical architecture proposal. CLI stays an adapter. Future implementation needs its own authorization and checks. |
| D26 | Proposed disposition: accept the retained current Rust conformance baseline with historical Python parity explicitly unknown. Recovering original Python evidence is an alternative for a later parity claim. | The prior conversion was authorized. This records a historical evidence exception, not a retrospective removal-gate pass. It does not require recreating missing bytes. |
| D27 | Use the concrete finite batch repair/handoff fixture, nested payment/cancellation/duplicate cases, and ongoing/multi-contract negatives here. | Proposed integrated fixture coverage, not accepted real business policy. |
| D28 | Independent exact-artifact content review for every task, all task cases, and these integrated negative/positive paths. | Proposed conceptual evidence threshold. Runtime tests remain separate. |
| D29 | Owner accepts one exact manifest, its declared coverage/limits and decision dispositions; changed selected bytes reopen affected review. | Final owner gate. Actual acceptance source and revision are currently absent. |
| D30 | T01 reconciles old guidance; retain separate conceptual and user-contract results; no unmapped statement silently disappears. | Proposed authority/gate policy preserving source obligations. |

Routine data layout, identifiers, fixture values, and scheduling choices are reviewable technical proposals within the authorized work.
They are not new owner requirements. Real owner-only values such as payment compensation remain open in their user-contract examples.

ACCEPT-008 | The designer must preserve each source obligation and distinguish technical proposals from owner decisions and fixture assumptions.

ACCEPT-009 | The owner must accept the exact integrated baseline before conceptual 0.1.0 can pass.

## Whole-plan trace and actual remaining gaps

| Ticket | Deliverable and integration responsibility | Required evidence |
| --- | --- | --- |
| T01 | Product concept and authority map. | Role/gate/source cases and exact boundary acceptance. |
| T02 | Contract model and runtime-slot satisfaction rule. | Identity, value, source, and phase cases. |
| T03 | All ten family catalogs and explicit definition lineage. | Thirty-four paired cases and ongoing/type/context/runtime-source checks. |
| T04 | Composition and materialization semantics. | Nested exits, effects, cleanup, substitution, runtime ports/suppliers and tool limits. |
| T05 | Versioned semantic profile and coverage schema. | Scoped source-backed divergences, counterexamples, truthful pending/unavailable states. |
| T06 | Tool catalog and protocol. | Exact envelopes, five statuses, stale/retry/coverage and failure cases. |
| T07 | Skill instructions and host context. | Source-grounded slots, decision reuse, formal parameter/supplier example, repair and malicious-source cases. |
| T08 | Scratch authoring. | Populated normalization and notification examples with correct type/source/phase boundaries. |
| T09 | Bounded plugin adoption. | Actual selected source/test inspection, partial coverage, observations distinct from requirements. |
| T10 | Migration planning. | Target/transition/readiness/authority distinctions, irreversible effect and recovery cases. |
| T11 | Controller and control ingress. | Initial states, mixed blockers, fresh gate, interruption, journal/budget/progress, runtime data, and ModeSession cases. |
| T12 | Implementation action boundary. | Exact completion, Choice location, authority/currentness, tuple coverage, multi-member and effect recovery cases. |
| T13 | Rust core/host architecture and prototype transition. | Typed interfaces, control/resume, source-bound inventory, honest historical parity and consumer limits. |
| T14 | This integrated flow, fixtures, register, manifest and independent case inspection. | Exact baseline and review records, then final owner acceptance. |

The review records and coordinator events identify their actual states. A positive content review does not mark a ticket complete.
Missing owner/upstream closure remains visible. Historical Python evidence is not retrospectively manufactured.
The real Rust archive/profile and host/tool/enforcement gaps retain defined unsupported or pending outcomes.

ACCEPT-010 | The reviewer must reject stale cross-task references or missing required evidence instead of inferring successful integration.

ACCEPT-011 | A changed baseline must reopen affected review while preserving unaffected historical results.

ACCEPT-012 | The project must report conceptual, prototype, implementation, runtime, and publication results separately.

The independent review records case inputs, expected and observed outcomes, artifact identities, findings, and limits outside this manifest.
No accepted conceptual release, full runtime harness, or live delegated action exists merely because this review candidate and fixture files exist.

## T14 obligation and case map

| Obligation | Exact review surface and cases |
| --- | --- |
| T14-001, T14-002, T14-012 | Responsibility map; trace steps 1–7; source/package/profile/request/receipt/checkpoint/completion/action records. |
| T14-003, T14-004 | Whole-plan map; ACCEPT-001/002/010; IG01–05, IG08, IG12–13. Actual close gates remain open. |
| T14-005 | Independent review of trace negativeMutations and IG01–IG15; author expectations are not review results. |
| T14-006 | D01–D30 register; compatibility selection; unresolved real owner acceptance blocks conceptual closure. |
| T14-007 | T01 authority map; IG14. |
| T14-008, T14-011 | Exact baseline manifest; D29; ACCEPT-009; no actual owner acceptance record exists. |
| T14-009, T14-010, T14-013, T14-015 | Evidence classes; trace world/provenance; IG04, IG10–11, IG15; D08/D26. |
| T14-014 | Nested scenarios retain owner Unknowns; hypothetical H1/H4 are separate; IG06–08. |
| T14-016 | ACCEPT-011, D29; IG03. |
| T14-017 | Positive fixture CP1/CO1 and IG01–15 simulate guarded outcomes without live tools. |

## See also

- [concept.md](../concept.md): product concept.
- [CONTRACT.md](../../CONTRACT.md): method rules.
