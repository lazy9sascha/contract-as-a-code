# Migration from observed behavior to target contracts

Revision: `migration-draft-4`. Status: provisional design. Designer: `the reviewer`. Reviewer: `the designer`.

This document answers T10 r2. It proposes D15 and a planning workflow. It performs no implementation, cutover, or service action.

## Authority and exact inputs

Migration reuses T09 observations. It adds explicit disposition, target, transition, readiness, and recovery meaning for one selected boundary.

| Input | Exact identity and limit |
| --- | --- |
| T10 r2 | SHA-256 `480fc80cffa28803e1f07fe626e84d9e249022c31851e9a346af3c94328e418d`. Current task scope and acceptance cases. |
| Retained T09 input | adoption-draft-1, SHA-256 `bcb4a83973a5b4deabf7cfc39af7ee8e89fb9bddad9ecef114bad7ac8920a549`. Emerging baseline/coverage schema, not accepted behavior. |
| Skill draft 3 | SHA-256 `71600bbb0e3003d89c940db5f646359e3f6a972e28feef073b502e677d26dee7`. Provisional questions, proposals, typed data sources, and exact handoff. |
| Retained profile draft 2 | Inspected SHA-256 `e297021536f7227f3044d3d55b7f7928bd4c18f2629d65f5f0d98bb35527648f`. Provisional source/version/runtime gaps remain visible. |
| T03 catalog | Manifest SHA-256 `1fe4321c706bfb5617ebd668940d4c077db858be03bb1c298843bc5291963654`. Explicit itemType, supplier eligibility, and typed reference aliases; no implemented checker claim. |
| T02 model draft 4, [T06 protocol](tool-protocol.md) | model-draft-4 `eb8f757e91a8cbfa9c000347d412d954031a8b4a9643b7b374d53563b9fa4623`; tool-protocol-draft-3 `6f976c1d0bb63085000d6456fae374d12093bb5252bfcab6441f0ab7c276022c`. Provisional identity, phase, and evidence interfaces. |
| Owner sources | Continued conceptual design authority and B12's mode direction. No migration policy or live-action acceptance is inferred. |

T05/T07/T09 are inputs and close gates. Final T09 revisions can require explicit schema reconciliation before acceptance.

The retained T09 input supplies only baseline, observation, coverage, disposition, and currentness schemas here. Its batch candidate is not a migration fixture input.

Those consumed schemas remain unchanged in phase-reconciled adoption draft 2. T14's compatibility map records that exact scope without relabeling historical observations.

T11 owns completion. T12 owns downstream action binding and runtime authorization. Their interface handoffs remain provisional.

## Terms and boundaries

| Term | Meaning |
| --- | --- |
| Current record | Exact T09 baseline observation, with its authority and coverage limit. |
| Disposition | Keep, change, remove, or unresolved treatment of one selected observed behavior. |
| Target contract | Required behavior after the proposed transition, with source or owner-decision provenance. |
| Transition contract | Permitted intermediate behavior, operations, failures, effects, and recovery between current and target boundaries. |
| Migration plan | Revisioned links between baseline, disposition records, target/transition contracts, cases, and evidence requirements. |
| Cutover | An action that changes which state, protocol, implementation, or route serves the selected operational boundary. |
| Recovery | A declared action after failure, with responsible actor and known effect constraints. |
| Reversal | A defined inverse action that restores declared prior guarantees within an explicit admitted domain. |
| Compensation | A separate forward action addressing an irreversible effect without erasing its history. |
| Execution authorization | Current authority for exact live actions, targets, environment, and bounded effects, separate from contract acceptance. |

Actors are the analyst, LLM skill, contract owner, state/effect owners, consumer owners, reviewer, controller, and execution host.

The analyst supplies observations and mappings. The skill proposes contracts. Owners resolve requirements and operating responsibility. The reviewer checks evidence.

The controller evaluates contract gates. A downstream authorized host performs any later live action. This planning workflow performs none.

## Input and disposition records

T09 remains canonical for Baseline, FileRecord, BoundaryEdge, Observation, TestExpectation, CoverageItem, InspectionMethod, and currentness.

Migration refers to those exact records. It does not rename observations into accepted requirements or duplicate their source-discovery rules.

| Migration record | Required fields and meaning |
| --- | --- |
| Plan | ID/revision, exact baseline/coverage refs, boundary, target refs, transition refs, dispositions, state/effect/consumer inventory, case/evidence requirements, decision refs. |
| BehaviorDisposition | Observation ref, responsibility ID, keep/change/remove/unresolved, proposed/accepted authority, rationale, exact answer/source ref, target refs, transition impact. |
| TargetLink | Exact target clause/instance, source observation links, requirement source or owner decision, accepted/proposed status, changed observable, affected cases. |
| StateItem | Exact schema/data boundary, read/write owners, invariants, current/target forms, writers/readers, consistency boundary, allowed intermediate states, retention and recovery. |
| EffectItem | Exact producer/consumer, protocol/version, identity, delivery/acknowledgment meaning, duplicate policy, irreversible boundary, observation and compensation/stop policy. |
| ConsumerItem | Exact consumer/version, boundary edges, current/target protocol expectations, compatibility evidence, upgrade/order responsibility, unknown dependencies. |
| Step | ID, actor, dependencies, preconditions, exact read/write/effect scope, transformation, invariants, observable result, validation, timeout/failure/recovery, idempotency and retry policy. |
| RecoveryRule | Trigger, known/uncertain effects, actor, stop conditions, reversal/compensation reference, admitted domain, preserved records, success/failure observables. |
| ReadinessRecord | Exact plan/baseline/contract revisions, required conditions, evidence refs, unresolved findings, and ready/not-ready result. It grants no execution authority. |

All listed fields occur once. Lists are finite and empty only with an explicit none or NotApplicable explanation. Missing behavior remains a decision.

The analyst owns record structure and provenance. Requirement and operational owners supply their respective decisions. Unknown ownership cannot become analyst authority.

Each inventoried selected behavior gets a disposition, including unmapped observations. Unknown coverage stays an explicit readiness gap.

Keep preserves a stated target guarantee only after its source/decision establishes that requirement. Existing implementation alone cannot do so.

Change identifies before/after observables. Remove records authorized requirement removal and affected consumers. Unresolved preserves current behavior outside any approved cutover scope.

T09's rejected candidate does not automatically mean removal of observed behavior. That current behavior still needs its own migration disposition.

MIGRATE-001 | The analyst must reference the exact T09 baseline, observations, coverage, and currentness records before deriving migration obligations.

MIGRATE-002 | The analyst must record keep, change, remove, or unresolved for every selected observed behavior.

MIGRATE-003 | Each target obligation must retain its exact requirement source or owner-decision provenance.

MIGRATE-004 | The analyst must separate observed current behavior, proposed target requirements, and permitted transition behavior.

MIGRATE-005 | The analyst must preserve unresolved behavior outside an approved cutover boundary without silently treating it as removed.

MIGRATE-006 | The analyst must identify state owners, effect owners, consumers, and crossing protocol dependencies within the transition boundary.

MIGRATE-007 | The skill must expose unknown ownership, compatibility, and coverage as readiness blockers.

## Ordered planning and prepared structures

1. Resolve the current T09 baseline and retain its coverage limits. Recheck source, profile, dependency, and consumer identities.
2. Record disposition for each selected observation. Preserve rejected candidates separately from removals of current behavior.
3. Populate the target contract from explicit requirements and accepted decisions. Retain source links and unknown values.
4. Inventory state, writers, readers, effects, consumers, and irreversible boundaries introduced or changed by that target.
5. Select exact prepared structures for the actual transition. Bind their slots and enumerate cross-pattern obligations.
6. Define preconditions, intermediate invariants, step outcomes, post-cutover checks, recovery responsibility, and non-reversible effects.
7. Construct cases for success, boundaries, partial failure, uncertain effects, stale baseline, and unknown ownership.
8. Send target and transition revisions, evidence requirements, decisions, and findings through T07 to the shared T11 loop.

| Required behavior | Prepared counterparts from T03 | Migration-specific obligations |
| --- | --- | --- |
| Ordered bounded transformation | FLOW-SEQUENCE p1, FLOW-LOOP p3, FN-OPERATION p2. | Explicit itemType, continuation, failure, resultPolicy, item mapping, and retained partial results. |
| State/version activation | STATE-MACHINE p1 and selected state/resource definitions. | Allowed old/shadow/active states, ownership, atomic activation, writer coordination, and invariant boundaries. |
| Persistence/transaction | EXT-PERSIST p2, EXT-TRANSACTION p1 only within a declared atomicity boundary. | Commit domain, partial effects, ambiguous acknowledgment, recovery, and consumer visibility. |
| Duplicate-sensitive output | EXT-MESSAGE p1, EXT-IDEMPOTENCY p1, and FAIL-RETRY p1 only when required. | Stable effect identity, receiver guarantees, timeout meaning, bounded retries, and irreversible exposure. |
| Cleanup or compensation | FAIL-CLEANUP p1, STATE-RESOURCE p1, EXT-COMPENSATION p2 only when required. | Acquisition identity, exactly-once release within supported semantics, compensation failure and retained history. |

Pattern selection does not prove operational support. A selected profile or checker without the required runtime semantics remains a gap.

No transition pattern is inserted for a documentation-only adoption that changes no state, code, protocol, route, or external effect.

MIGRATE-008 | Each transition step must define preconditions, admitted effects, intermediate invariants, observable results, and failure behavior.

MIGRATE-009 | Each cutover plan must define observable post-cutover checks and their responsible actor.

MIGRATE-010 | Each recovery rule must identify its trigger, actor, known effects, admitted actions, and failure outcome.

MIGRATE-011 | The analyst must distinguish reversible steps, irreversible effects, and unresolved reversibility.

MIGRATE-012 | Each reversal claim must identify its inverse, admitted domain, retained state, and conditions preserving required guarantees.

MIGRATE-013 | The analyst must not describe compensation as erasure or automatic rollback of an irreversible effect.

## D15 proposal: readiness, evidence, and authorization

Proposal: distinguish three results. `target-complete` and `transition-complete` come from separate exact T11 gate evaluations.

`execution-authorized` comes from current host/action authority under T12. Neither contract-completion result supplies that authority automatically.

| Category | Required plan/evidence before readiness | Authorization consequence |
| --- | --- | --- |
| Documentation-only adoption | Explicit proof of no code/state/protocol/route/effect change; disposition/target/source review. Transition is NotApplicable with reason. | No live cutover is proposed. Later code/deployment work needs its own action binding. |
| Reversible isolated change | State/effect owners, exact inverse domain, retained prior state, consumer compatibility, staged validation, and postchecks. | Named operational owner authorizes exact bounded environment/actions. Reversal needs its own valid binding. |
| Persistent shared-state transition | Schema/data identities, writer/read coordination, consistency/activation boundary, retained snapshot or equivalent recovery source, consumer ordering, and partial-failure cases. | Named state owner and required operating authorities authorize steps after all readiness conditions pass. |
| Irreversible or externally visible effect | Effect identity, receiver semantics, uncertain-delivery policy, compensation or stop-before-effect condition, and required evidence. | Explicit effect scope and responsible owner remain necessary. Unknown duplication or recovery blocks affected live steps. |

Every category requires current source/dependency/consumer inputs, resolved blocking decisions, and evidence for its actual selected guarantees.

Categories combine when a plan has several effect kinds. All applicable evidence and authority obligations apply together, without selecting the weaker category.

Required evidence identifies method, version, environment assumptions, covered behavior, and limits. Designed cases are not executed migration evidence.

A plan can remain useful and incomplete while evidence is unavailable. Its readiness record states not-ready and names each missing condition.

An execution authorization record names actor, exact plan/contract revisions, action set, targets, environment, effect bounds, and applicable validity/revocation conditions.

Required authorization fields come from the runtime host contract. Missing permission blocks live dispatch. Planning continues within existing authority.

MIGRATE-014 | The controller must keep target completion, transition completion, readiness, and execution authorization as separate records.

MIGRATE-015 | The readiness reviewer must reject affected cutover readiness while ownership, compatibility, recovery, or required evidence remains unresolved.

MIGRATE-016 | The execution host must check current explicit authorization before every live cutover action.

MIGRATE-017 | The execution host must reject authorization bound to different plans, contracts, environments, targets, or effects.

MIGRATE-018 | The analyst must not infer live-action permission from planning authority or a completed target contract.

## Partial failure and resumption

Every step reports not-started, completed, failed-before-effect, failed-with-known-effects, or effect-uncertain. These are transition-domain outcomes, not T06 checker statuses.

Completed steps retain action identity, observed effects, validation results, and exact next preconditions. A lost response does not mean failed-before-effect.

Recovery first establishes the last verified state and uncertain effects. It resumes only a declared step whose base, preconditions, and authority still hold.

A retry requires the step's explicit eligibility, bounds, duplicate-effect handling, and current authorization. Unknown delivery does not justify a fresh identity.

No replay occurs solely because the process lacks a success receipt. A receiver query or guaranteed same-identity deduplication can establish a safe next action.

A reversal after new writes needs a defined inverse for those writes and current consistency evidence. Restoring an old snapshot alone can lose accepted new work.

Source, schema, profile, callback, consumer, or owner-policy changes invalidate affected mapping/readiness evidence. An unchanged filename cannot preserve readiness.

MIGRATE-019 | The plan must preserve known and uncertain effects after interruption or partial failure.

MIGRATE-020 | The execution host must not replay an uncertain effect without the transition's declared duplicate-safety or reconciliation evidence.

MIGRATE-021 | The analyst must invalidate affected transition assumptions and readiness after baseline or dependency changes.

MIGRATE-022 | The controller must preserve failed transition evidence and unresolved work when planning resumes.

## Worked hypothetical cases

These fixtures declare their own source requests and policies. They are not real operating decisions, execution results, or proposed universal migration defaults.

### MC01: stateless contract adoption

Fixture baseline B-STATIC observes a pure function returning its input text. Fixture owner requires that exact behavior, without code or deployment changes.

| Record | Populated fixture value |
| --- | --- |
| Disposition | OBS-IDENTITY: keep, backed by fixture owner decision D-KEEP. Observed code alone does not approve it. |
| Target | Input Text, identical output codepoints, no effects. Selected FN-OPERATION p2 and exact expression mapping. |
| Transition | NotApplicable: no state, code, protocol, route, or effect changes. No live cutover. |
| Recovery | NotApplicable: no operational step is performed. No rollback subsystem is manufactured. |
| Actor/evidence | Analyst retains baseline and decision. Reviewer checks source/target equivalence within the declared method. |

Expected outcome: proposed target enters T11. No executable transition or live permission is claimed. Unknown source coverage still blocks full target completion.

### MC02: persistent-state conversion

Fixture baseline B-LEDGER contains account rows A=12 and B=0 in integer minor units. Fixture owner requires records `{minor:Int,currency:USD}`.

The hypothetical state owner is LedgerOwner. Writers and two readers have explicit fixture protocol versions. No other consumer is assumed absent.

| Record | Populated fixture value |
| --- | --- |
| Disposition | OBS-BALANCE: change representation, preserve integer minor amount. Source is D-CURRENCY's explicit fixture decision. |
| Target | `minor` equals prior integer exactly. `currency` is singleton USD. Invariant preserves account IDs and values. |
| Transform | A→{minor:12,currency:USD}, B→{minor:0,currency:USD}. Inverse extracts minor only for records with exact USD currency. |
| Steps | Prepare isolated shadow; copy exact baseline rows; validate bijection and counts; coordinate writers; reconcile admitted changes; compare activation epoch; activate compatible readers/writers. |
| Cutover preconditions | Exact baseline/current epoch, known owner, accepted target/transition, compatible consumer versions, invariant checks, coordinated writer boundary, recovery evidence, current action authority. |
| Postchecks | Reader sees both exact records, unchanged amounts/IDs, current write route, no old writer after activation, and recorded validation results. LedgerOwner owns failures. |
| Recovery | Before activation, retain old route and isolate invalid shadow. After activation, stop affected writers and reconcile all accepted new writes through the inverse before reversal. |

The prepared state machine distinguishes old-active, shadow-validating, activation-ready, and new-active. Each transition binds its actor and current epoch.

The fixture declares LedgerRow as Record{account:ID,minor:Int}. Its two literal rows retain A/12 and B/0 under that exact type.

Shadow copying uses FLOW-LOOP p3: kind=collection, itemType=LedgerRow, items=that static literal list, body=exact transform operation, continuation=bounded.

Its static termination predicate is remaining rows reach zero. The body signature declares LedgerRow→target entry, preserving account identity and minor amount.

Its order=ascending baseline index, failure=stop, resultPolicy=collect. Results retain each transformed row's tagged outcome. No undeclared slot is added.

Expected outcome: representation preservation is reviewable. Live readiness remains false until operational evidence and authorization exist.

### MC03: outbound message and uncertain delivery

Fixture current behavior sends an unkeyed message. Fixture owner requires at most one receiver-side effect for logical message K.

Disposition changes the protocol to stable identity K and receiver deduplication. ConsumerOwner owns the receiver guarantee and its evidence.

The target/transition use explicit message, idempotency, and retry structures from the catalog. The sender records K before any delivery attempt.

Failure input: receiver applies K but acknowledgment is lost. Sender records effect-uncertain and retains K. A timeout does not prove nondelivery.

With verified receiver same-K deduplication or authoritative status query, the declared recovery can continue within its bounds. A new K would violate the guarantee.

Without that receiver capability, cutover readiness is blocked before duplicate-sensitive dispatch. Compensation is not assumed to undo the delivered message.

Expected outcome: ConsumerOwner supplies capability evidence or a stop-before-effect policy. Contract completion does not fabricate receiver support or permission.

### MC04: partial conversion failure

Use MC02. Shadow row A converts successfully; row B fails validation before activation. Old-active remains the serving state.

The step records A's shadow write, B's failure, old activation epoch, and unchanged serving route. LedgerOwner isolates the shadow and decides repair or disposal.

No partial new schema becomes visible to consumers. Repeating the entire conversion needs explicit idempotent shadow-write keys and fresh current-baseline checks.

Variant failure: activation succeeds but confirmation is lost. Recovery reads the authoritative activation epoch before choosing any action. It does not activate twice blindly.

Expected outcome: known/uncertain effects remain distinct, partial work stays retained, and live reversal or replay needs its own preconditions and authority.

### MC05: unknown state ownership

Baseline identifies shared stored rows and a consumer callback, but no authorized owner for the writer or its consistency boundary.

Disposition and target can remain proposed. StateItem.owner and coordination/recovery responsibility remain Unknown D-OWNER.

Expected outcome: no cutover-ready result. Ask the responsible system owner for authority/ownership evidence. Preserve current behavior outside approved transition scope.

### MC06: source or consumer changes

The plan targets baseline B1, but selected source hash, imported schema, or consumer version changes to B2 before readiness evaluation.

Expected outcome: retain B1 history, create the new T09 baseline, reassess affected dispositions/assumptions, and invalidate stale transition/readiness evidence.

An identical selected filename supplies no currentness evidence. An unchanged unaffected observation remains historical with its own scope.

### MC07: accepted target without execution permission

Mock T11 gate completes the target and transition at exact revisions. Host authorization for production writes remains absent.

Expected outcome: controller retains completion objects, but downstream host refuses live actions. Existing planning authority needs no repeated approval to refine the plan.

### MC08: rejected candidate versus removal

T09 records an observed branch and an owner rejects a proposed new requirement derived from it. No answer addresses removal of the current branch.

Expected outcome: candidate rejection is retained, while migration disposition remains unresolved. The planner cannot delete the observed behavior by interpretation.

## Shared-loop handoff and review map

The T07 handoff contains exact baseline/coverage refs, target and transition package proposals, disposition/state/effect/consumer records, owner decisions, cases, and check requirements.

It separates proposed, accepted, observed, and inferred records. Readiness and runtime authority remain separately referenced, without a skill completion claim.

MIGRATE-023 | The workflow must hand target and transition contracts, provenance, decisions, cases, and checking gaps to the shared controller.

MIGRATE-024 | The skill must reuse applicable decisions while preserving their exact source and affected migration revisions.

| T10 obligation | Design evidence | Cases |
| --- | --- | --- |
| T10-001 | BehaviorDisposition, MIGRATE-001/002/005. | MC01–MC05, MC08 |
| T10-002 | TargetLink, source/authority separation, MIGRATE-003/024. | MC01–MC03, MC08 |
| T10-003 | Separate contract records, MIGRATE-004/014. | MC01–MC04, MC07 |
| T10-004 | StateItem/EffectItem/ConsumerItem, MIGRATE-006/007. | MC02, MC03, MC05 |
| T10-005 | Step/ReadinessRecord, D15, MIGRATE-008/009/015. | MC01–MC04, MC06 |
| T10-006 | RecoveryRule and partial outcomes, MIGRATE-010/019/020/022. | MC02–MC05 |
| T10-007 | Reversal domain and compensation distinction, MIGRATE-011–013. | MC01–MC04 |
| T10-008 | Unresolved disposition/scope, MIGRATE-005/007/015. | MC05, MC08 |
| T10-009 | Separate live authorization, MIGRATE-014/016–018. | MC02–MC05, MC07 |
| T10-010 | Shared T07/T11 handoff, MIGRATE-023/024. | MC01–MC08 |

D15 maps to the category/evidence/authorization table and MC01–MC07. These cases supply expected design outcomes for independent review.

No runtime migration, reversal, consumer test, implementation, or live authorization was executed. Positive design review cannot establish those results.

Remaining gates: accepted T05/T07/T09 outputs, final exact baseline-interface reconciliation, D15, and independent review. No ticket completion is claimed.

## See also

- [scratch-workflow.md](scratch-workflow.md): scratch workflow.
- [plugin-adoption.md](plugin-adoption.md): plugin adoption.
- [CONTRACT.md](../../CONTRACT.md): method rules.
