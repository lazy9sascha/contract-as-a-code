# Contract-completion controller

Revision: controller-draft-5. Status: provisional design. Designer: `the designer`. Reviewer: `the reviewer`.

This document answers T11 r2, SHA-256 `8e92f7e7ca8ed78b3045ba33dc5e819b89ddb2b0686d0267fec348abd6821b00`.
It proposes D16–D18. It supplies no running controller, checkpoint store, or accepted owner policy.

The controller completes contract authoring and assessment. A completed contract can describe an intentionally ongoing target program.
Target continuation and harness convergence are separate obligations. The authoring budget never becomes an invented target-program deadline.

## Exact inputs and interface assumptions

| Input | Inspected revision and identity |
| --- | --- |
| T02 model | model-draft-2, SHA-256 `f307de558f9ae90038d376bc8b77f0f9fd3d0609c5f60d6fe0af5f21c91cf7e6`. |
| T03 patterns | patterns-draft-5 bundle manifest SHA-256 `83c5bd9f2b57d932c48d90b3c70d8f2773c6c7e662b19e7988c2efbb18843b4a`. |
| T04 composition | composition-draft-5, SHA-256 `2bd19c03679b25bb91146d58a392f2a40cfacb2d56e9c80f83a2e40bb8cf19d8`. |
| T06 tool protocol | tool-protocol-draft-3, SHA-256 `6f976c1d0bb63085000d6456fae374d12093bb5252bfcab6441f0ab7c276022c`. |
| T07 skill | skill-draft-2, SHA-256 `69199f3410755d810f507e93d5a4f49ec9119b8cf3efc8aa28b0c500dd46a9ae`. |
| T08 scratch | scratch-draft-5, SHA-256 `eca246a50db311eee8add9ae448091608cd6e467655276eb67d89ff8fc25db1e`. |
| T09 adoption | adoption-draft-1, SHA-256 `bcb4a83973a5b4deabf7cfc39af7ee8e89fb9bddad9ecef114bad7ac8920a549`. |
| T10 migration | migration-draft-1, SHA-256 `2fac8f561221a193b8982702131268ce114852fcde9fcdedc79480c012187cb4`. |

T06–T10 are input, design-output, and close-gate dependencies. Accepted revisions and owner decisions remain absent.
Mode interface: each mode returns explicitly named package proposals, boundaries, source evidence, decisions, findings, and check requirements.
Scratch has no implementation baseline. Adoption and migration retain their selected baselines without turning observations into requirements.
All modes use this state machine and gate. A mode cannot declare its own successful completion.

A `ModeSession` contains session ID, source scope, required package roles, member run IDs, shared policy/budget ledger, and aggregate status.
Every field occurs once. The controller owns the session. Each member run still targets exactly one package revision.
Scratch and ordinary adoption normally require one target-contract member. T10 can require separate target-contract and transition-contract members.
Documentation-only adoption can mark transition NotApplicable with T10's explicit reason and evidence.
Aggregate authoring completion requires every required member's exact current completion object. It does not grant migration execution authority.
The session shares budget reservations across its members. Creating another member cannot reset consumed work or an existing retry chain.

## Controller records

All records carry stable IDs and exact revision references. The host retains their bytes and identifies them by SHA-256.
The host owns execution records. The skill proposes content. Authorized requirement owners resolve decisions. Distinct reviewers supply independent assessments.

| Record | Required typed contents and missing rule |
| --- | --- |
| Run | run ID, selected mode/boundary, current package reference, exact policy/profile/catalog/descriptors, authority, state, event sequence. Initial package can be explicitly absent. |
| Work item | stable item ID, kind, target references, required inputs, dependency IDs, responsible actor, status, criterion IDs, blocking flag. Missing target or unknown blocking status prevents gate eligibility. |
| Assignment | action ID, exact context/package base, permitted scope, required output, policy reference, reserved budget, and work-item IDs. |
| Proposal | T07 handoff, exact base, proposed changed entities, source authority, expected checking impact, and application state. No completion_claim field is admitted. |
| Check invocation | Exact T06 request bytes/digest, descriptor/build, check units, input manifests, retry chain, execution state, and validated receipt references. |
| Decision/question | T02 decision, applicability, owner/source, question ID, delivery state, known values, pending answer, and affected work. Silence remains unanswered. |
| Progress entry | projected content fingerprint, blocking obligations, eligible coverage, decision applicability, capability identities, pending retry identity, and chosen next action. |
| Checkpoint | run/context/package identity, saved state, next event sequence, all work/decisions/questions/proposals/invocations, budget ledger, progress history, and retained event-prefix reference. |
| Completion object | exact package and manifest digest, profile, catalog/relations, criteria, current eligible evidence, independent reviews, owner acceptance, policy, and completion event. |

Work-item status is pending, in-flight, resolved, stale, or blocked. Resolved records identify the exact discharging evidence or decision.
Historical failures, rejected receipts, and superseded proposals remain retained. A later pass changes applicability, not their historical bytes.

Checkpoint manifests exclude their own digest and the mutable event-log tail. The coordinator records the checkpoint digest in a separate event.
Its event-prefix reference identifies only already retained events. The checkpoint does not create a self-hashing or future-event dependency.

CONTROL-001 | The controller must bind work, assignments, proposals, and evidence to exact package and dependency revisions.

CONTROL-002 | The controller must preserve owner decisions, questions, failed attempts, and unfinished obligations across interruption.

## D16 proposal: states and transitions

### Initialization

Creation is a guarded operation before an existing run state exists. It is not an implicit state transition.
The caller supplies a new run ID, creation authority, retained intent, mode/boundary, and initial-or-existing-package marker.
Duplicate run IDs with different creation bytes are rejected. Missing authority or unresolved creation identity rejects creation without a run.

After identity validation, the host journals creation and selects the first state through these guards in order:

| Creation condition | Initial result |
| --- | --- |
| Retained authoritative input is corrupt and recovery cannot establish its identity | failed with integrity reason and unfinished scope. No invented input. |
| Required policy or recoverable context is missing | paused with explicit missing reference, creation checkpoint, and resume condition. |
| Scratch has no package and known source content permits authoring | drafting with explicit initial/no-package marker. |
| Current package has a current repairable finding and authorized proposal inputs | repairing with exact base and work item. |
| Current package has ready required check/review work | checking with exact targets and plan. |
| Current package needs independently available known authoring | drafting with source-grounded assignment. |
| Only unresolved owner decisions remain, possibly alongside capability gaps | waiting-for-owner with all blocker reasons retained. |
| Only unsupported required capabilities remain | capability-blocked with affected guarantees. |

An initial package with no required work still enters checking for gate evaluation. Creation never initializes directly as complete.
An indeterminate creation condition remains a located context finding and pauses, rather than selecting a guessed state.

Resume is a separate operation on the existing run/checkpoint. It does not call creation with fresh budgets or empty decisions.
A linked new run after failed requires explicit creation authority and the failed predecessor reference.
It retains relevant historical action/retry identities and checks decision applicability. It cannot relabel a forbidden retry as new work.

### Existing states

One run has one state. Waiting and blocked states indicate that no independent authorized action can currently advance its required work.
The blocker register can retain several reasons. Waiting-for-owner takes display precedence over capability-blocked when both remain.
Integrity failure and policy suspension take precedence over those display reasons. No reason disappears because another has precedence.

| State | Entry meaning and retained data | Legal exits and guards |
| --- | --- | --- |
| drafting | A mode or authoring assignment can populate known content. Retain source/boundary and pending questions. | checking after valid proposed revision application; repairing when a current finding needs a proposal; waiting-for-owner or capability-blocked when no independent action remains; paused or failed under shared rules. |
| checking | Exact current check plan and declared reviews can run. Retain invocations, reservations, and receipts. | complete only through the gate; repairing for repairable current findings; drafting for newly required known authoring; waiting-for-owner or capability-blocked for unresolved blockers; paused or failed. |
| repairing | A current finding and authorized base admit a bounded proposal. Retain old findings and expected impact. | checking after atomic valid application; drafting if scope-consistent structure needs authoring; waiting-for-owner or capability-blocked when no independent action remains; paused or failed. |
| waiting-for-owner | An owner-only value blocks remaining useful work. Retain exact questions and known values without active compute. | drafting, repairing, or checking after applicable answer/new ready work; capability-blocked when owner blockers resolve but required gaps alone remain; paused or failed. Silence has no transition. |
| capability-blocked | Required semantics, profile, method, or host capability lacks support and no independent useful work remains. | drafting/checking after exact capability or scope evidence changes; repairing when new current evidence enables an authorized repair directly; waiting-for-owner for the next sole owner blocker; paused or failed. |
| paused | Policy budget, repeated state, explicit user pause, or interruption suspends dispatch. Retain saved prior state and resume condition. | saved state if still valid, or freshly selected drafting/repairing/checking/waiting/capability state after resume-condition and entry guards pass; failed on unrecoverable integrity error. No budget reset or direct completion. |
| failed | Unrecoverable checkpoint/input integrity failure or authorized run abort. Retain reason, known effects, evidence, and unfinished work. | No same-run continuation. A linked new run requires valid inputs and records this failed predecessor. |
| complete | Gate passed for one exact package/profile/policy. Retain immutable completion object. | drafting/repairing/checking only after a changed governing input creates reopened work and a new completion target. Preserve the earlier object as historical. |

Only these exits and the shared suspension/failure rules are valid. An invalid event leaves the state unchanged and records the missing guard.
An interruption records the saved state and enters paused when durable recording remains possible.
If the host stops before it records that event, recovery treats the last in-flight journal entry as interrupted, not successfully completed.

CONTROL-003 | The controller must select actions from unresolved work and current findings under the declared transition guards.

CONTROL-004 | The controller must reject invalid transitions without changing state or inventing missing guard evidence.

CONTROL-005 | The controller must distinguish unresolved owner meaning from unavailable checking capability.

### Control-event ingress

A ControlEvent contains event ID, run ID, expected event sequence, event kind, typed payload, and exact source/authority reference.
The core validates those fields and computes a legal transition. A caller does not submit the desired next state.
An exact duplicate event returns its existing receipt without applying it again. Reusing its ID with different bytes is rejected.
Stale sequence, missing authority, or invalid payload leaves state unchanged with a located rejected-event receipt.

| Event | Guard and computed outcome |
| --- | --- |
| owner-pause | Authorized unfinished run enters paused with saved work and owner-resume condition. A completed authoring run retains its completion result. |
| owner-abort | Authorized unfinished run enters failed with reason run-aborted and retained unfinished obligations. It makes no claim that requirements are false. |
| resume | Validate saved checkpoint, specific suspension condition, authority, budget, and fresh entry guards before selecting a current state. No caller-selected transition or budget reset. |
| owner-answer | Retain exact question/decision/source and recheck applicability. Schedule only work enabled by that answer. |
| requirement-change | Apply a valid revision-bound proposal under ordinary authority/current-base guards, invalidate affected evidence, and reopen work. |
| profile/method/capability-change | Recheck exact governing identities and coverage. Invalidate affected applicability and reopen required checking/authoring without lowering guarantees. |
| authoring-policy-change | Require authorized exact policy revision. Preserve consumed work, reservations, failed attempts, and retry identities. Reassess gate/progress/budget effects. |
| runtime-authorization-change | Forward affected execution eligibility to T12. Stop protected dispatch there when required. Historical contract-completion evidence is not retroactively erased. |
| host-interruption | Retain saved state/action intent and enter paused when recording remains possible. Recovery recognizes an unclosed journal action as interrupted. |

Pause/abort do not erase existing completion objects. A request to stop downstream execution follows T12's in-flight policy and effect journal.
Revoked requirement-acceptance authority differs from runtime-grant revocation: the former invalidates affected current gate applicability and requires renewed owner evidence.

CONTROL-024 | The core must validate control-event authority and freshness before computing any resulting transition.

CONTROL-025 | The core must preserve historical completion objects and effect records when processing interruption, pause, abort, or revocation events.

### Action selection and concurrency

First validate context and freshness. Incorporate applicable recorded answers before issuing duplicate questions.
Then select ready work by dependency order and stable work-item ID within the policy's priority classes.
The priority classes are authority/context repair, known-content authoring, current findings, required checks/reviews, then gate evaluation.
An unanswered question does not prevent independent known-content authoring.

Package changes are serialized. Two proposals against the same revision do not both apply without a current-base comparison.
Read-only checks can run concurrently when their immutable inputs resolve and the shared budget reserves all invocations.
A package/profile/descriptor change marks affected in-flight checks stale. Their eventual receipts remain historical, without current evidence eligibility.
Missing impact dependencies invalidate the enclosing affected scope conservatively under T06 and T02.

A skill suggestion is advisory. An LLM completion sentence has no transition authority and supplies no evidence.

CONTROL-006 | The controller must apply a proposal only when its exact base and declared mutation scope remain current and authorized.

CONTROL-007 | The controller must create a new package revision and invalidate affected evidence after an applied change.

CONTROL-008 | The controller must rerun affected checks against the resulting exact revision before reconsidering completion.

## Tool and repair loop

The controller expands work into explicit T06 check units against exact target/entity/slot/relation references.
It selects admitted registered descriptors, retains all manifest inputs, reserves budget, journals intent, and dispatches the exact request.
The response is evidence only after T06 identity, digest, coverage, execution, and receipt validation.

| Result or receipt | Controller action |
| --- | --- |
| Current accepted passed assessment | Add only its declared method/coverage to eligible evidence. Other criteria remain unresolved. |
| Current failed assessment | Retain located findings. Route owner-only meaning to questions and source-grounded repairs to the skill. |
| Unsupported | Retain capability gap and affected guarantees. Do not weaken the contract to remove the check. |
| Indeterminate | Retain undecided constraint and supported method limit. Seek a separately declared method or required review. |
| Execution-error | Retain failed attempt and unfinished coverage. Use only the exact eligible T06 retry rule when its guard passes. |
| Rejected or stale receipt | Preserve raw record and reasons. It cannot supply current successful evidence or current repair authority. |

T06 permits at most its declared one eligible read-only transient retry with fresh request identity and preserved retry chain.
The controller cannot evade that bound with another root ID or by reconstructing context.
Semantic failure, unsupported method, unknown effects, and invalid request are not transient operational retries.
A repaired package requires new analysis invocations, not a retry claiming the old invocation's inputs remained unchanged.

CONTROL-009 | The controller must validate T06 receipts before using assessments as current evidence.

CONTROL-010 | The controller must preserve unsupported, indeterminate, execution-error, rejected, and stale results without treating them as passed evidence.

CONTROL-011 | The controller must preserve retry-chain attempts and cumulative budgets across resumption and successor requests.

## D17 proposal: user-contract completion gate

The gate evaluates one exact current package, declared scope, capability profile, policy, and evidence manifest.
Each required condition below returns satisfied, unsatisfied, or unresolved. Only all satisfied permits complete.
Unresolved means missing meaning, authority, coverage, applicability, or valid evaluation. It never means permission to guess.

| Required condition | Passing evidence |
| --- | --- |
| Scope and authority | Defined actors, input/observable boundary, exclusions, source provenance, and applicable requirement authority. |
| Typed content | Required slots have valid static values, valid NotApplicable, or exactly one admitted typed runtime supplier. Runtime-enabled slots still permit valid constants. No required meaning or canonical conflict remains. |
| Composed behavior | Required name, order, type-flow, failure, effect, cleanup, substitution, and selected continuation rules have current passing assessments/reviews. |
| Coverage | Every required semantics/profile/check has a declared supported method and eligible current evidence. Unsupported required guarantees remain blockers. |
| Cases | Required outcomes, branches, boundaries, failures, and composition cases have current determinate review results under declared criteria. |
| Review | A reviewer distinct from the artifact designer checks meaning, sources, required cases, and limits. Required additional criterion-specific reviews also pass. |
| Decisions | No applicable blocking owner decision or unresolved required criterion remains. Recorded answers still govern their exact scope. |
| Owner acceptance | Required owner or explicitly authorized delegate accepts this exact package revision and scope with retained source evidence. |
| Freshness | All required evidence matches current targets, dependencies, definitions, profile, descriptors, methods, decisions, and assumptions. |

A manual semantic review can satisfy a declared semantic-review criterion. It cannot impersonate an implementation test or an unsupported universal proof.
A required liveness proof without a supported method remains a gap even when a finite stop trace succeeds.
A target intentionally ongoing under its accepted continuation policy need not reach runtime termination for authoring to complete.
Likewise, formal caller data needs a valid declaration, source, type, mapping, phase, and rejection behavior, rather than a fabricated future value.
Required runtime-enabled slots can lack a static Cell only when their exact admitted supplying relation passes source/type/availability checks.
A Known Expression<T> does not satisfy a T value slot. Static declaration/configuration/policy requirements cannot escape through an undeclared runtime supplier.

The owner acceptance record concerns a user's contract. It is separate from the repository's conceptual 0.1.0 owner gate.
Proposed mock acceptance in a fixture supplies simulated gate input only. It does not accept a real product or user contract.

CONTROL-012 | The controller must require every declared gate condition and criterion to pass for the exact current completion target.

CONTROL-013 | The controller must reject LLM assertions, syntax-only passes, and finite fixtures as substitutes for other required evidence.

CONTROL-014 | The controller must retain an immutable revision-bound completion object after the gate passes.

CONTROL-015 | The controller must reopen affected work when a completed package or its governing input changes.

CONTROL-023 | The gate must validate runtime suppliers instead of treating future invocation data as unresolved owner requirements.

## D18 proposal: progress, budgets, waiting, and recovery

An authorized caller supplies an exact policy revision with finite max authoring actions, tool invocations, and active elapsed milliseconds.
Optional token/input/output limits are explicit when enforced. Nonnegative integer limits admit zero, meaning no corresponding dispatch.
No numeric defaults are proposed here. Missing required budget fields cause paused(policy-missing), while preserving known draft content.
Waiting for an owner or user resume consumes no active computation budget. Its retained state remains unfinished indefinitely without an answer.
An optional absolute deadline requires its own declared clock and expiry outcome. The controller does not invent one.

Before each dispatch, the host reserves the action's maximum admissible consumption from the remaining ledger.
On a validated completed receipt, it commits measured use and releases unused reservation.
An interrupted attempt without a trustworthy usage receipt retains its full reservation conservatively. Recovery cannot reset it to zero usage.
T06 per-invocation bounds and retry-chain budgets also apply. The smaller permitted limit controls dispatch.

At scheduling boundaries, the controller records a progress fingerprint of content, unresolved obligations, eligible coverage, applicable decisions, capabilities, and pending retry state.
The content projection excludes revision labels, timestamps, log growth, and non-normative display text. Its exact method belongs to the policy revision.
Controlled clause text, source authority, and behavior-affecting values remain included. The method does not claim arbitrary prose equivalence.
An exact repeated unresolved fingerprint detects a cycle such as A → B → A.
A newly generated revision ID or repeated LLM assertion supplies no progress by itself.
An eligible pending T06 retry remains part of the fingerprint, preserving its one bounded opportunity without permitting an endless fresh chain.
After a repeated unresolved fingerprint, pause(non-progress) retains the cycle and requires changed relevant input, capability, or authorized policy evidence to resume.
This is exact-state detection, not universal semantic-equivalence analysis. Finite budgets still bound undetected variation.

Checkpoints precede effectful local revision application and tool dispatch. Controller tools in the current T06 catalog are read-only.
Each action has a stable journal key and intended base/output references before dispatch or local application.
On recovery, a recorded applied proposal resolves to its retained output revision. It is not applied a second time.
A pending proposal with no application record stays proposed until current-base and integrity checks pass.
An unresolved in-flight tool call remains interrupted. Any allowed successor preserves the original chain and reservation.
No implementation action or live external effect occurs in this authoring loop. Future effectful tools need separate accepted contracts and recovery rules.

Advertising resumable implementation requires T13's eventual durable atomicity/recovery interface.
This conceptual design requires that interface but does not claim a checkpoint store already enforces it.

CONTROL-016 | The controller must stop dispatch when its required budget is missing or exhausted without declaring completion.

CONTROL-017 | The controller must detect repeated unresolved fingerprints and retain the non-progress cycle before suspension.

CONTROL-018 | The controller must preserve consumed and reserved budgets instead of resetting them after interruption.

CONTROL-019 | The host must journal action identity and intended revision effects before dispatch or local proposal application.

CONTROL-020 | The controller must verify checkpoint integrity, freshness, and prior application state before resuming an action.

CONTROL-021 | The controller must preserve questions as unanswered until an applicable authorized answer is recorded.

CONTROL-022 | The host must satisfy the declared durable recovery interface before advertising resumable implementation.

## Concrete hypothetical traces

These traces simulate policy and tool records. They claim no live tool, durable store, or real owner acceptance.
Fixture policy P-FIX explicitly allows four authoring actions, eight tool calls, and 100 active milliseconds.
All fixture checks have hypothetical registered read-only builds and exact retained inputs. T06 descriptor limits also apply.

| Case | Inputs and sequence | Expected conceptual outcome |
| --- | --- | --- |
| CC01 | retryCount Unknown in R1. Independent input-order slot is Known. waiting-for-owner records Q-count. Mock answer says one additional attempt. | Preserve known order. Record answer and applicability, apply proposal R2, invalidate affected evidence, check R2. No fresh decision register. |
| CC02 | R2 gets a passed slots receipt. An edit creates R3 before gate evaluation. | Receipt remains historical for R2. R3 cannot complete until required affected checks pass. |
| CC03 | Repairs change content A to B and back to A, with the same open obligations and eligible coverage. | Repeated unresolved fingerprint pauses with non-progress cycle. New revision labels do not hide the cycle. |
| CC04 | First read-only tool attempt times out after reserving ten milliseconds; exact eligible retry succeeds using remaining chain budget. | Preserve failure and retry identities. Success supplies only current covered evidence. Chain permits no third attempt. |
| CC05 | Budget permits zero remaining calls while required checks remain. | paused(budget-exhausted), checkpoint and unfinished checks retained. No completion. |
| CC06 | Proposal R1→R2 is journaled and applied. Process stops before the skill receives confirmation. | Recovery finds the retained application/output and resumes from R2. It does not apply the change twice. |
| CC07 | LLM says complete while payment→shipment recovery finding remains open. | Gate rejects completion and routes the explicit composition obligation. |
| CC08 | Required duplicate-effect method is unsupported, while syntax passed. | capability-blocked after independent work finishes. Syntax cannot discharge duplicate-effect coverage. |
| CC09 | Mock fixture has all typed slots, current passed required assessments, independent case review, applicable decisions, and exact mock owner acceptance. | checking→complete creates an exact fixture completion object. It does not accept the real repository baseline. |
| CC10 | A completed ongoing-service contract has active/observation/stop evidence but no promised global runtime termination. | Gate evaluates its declared continuation criteria. It cannot invent a termination requirement or require running the service forever. |
| CC11 | Same ongoing-service fixture additionally requires an unsupported unbounded liveness proof. | Required proof remains a gap. Finite tick/stop cases cannot satisfy that criterion. |
| CC12 | Owner remains unavailable after Q-count delivery. Other known authoring first completes. | Eventually waiting-for-owner with Q-count unanswered. No automatic answer, repeated redispatch, or false complete result. |
| CC13 | New owner requirement edits a previously completed revision. | Preserve old completion object, create reopened target, invalidate affected bindings/evidence, and return to guarded work. |
| CC14 | Context omits a pending decision register or checkpoint digest differs. | No claimed safe resumption. Recover valid retained input or enter failed with exact integrity reason and unfinished work. |
| CC15 | Two checks run concurrently; an owner answer changes their target package before results arrive. | Both late receipts stay bound to the old revision. Only new current checks can satisfy the revised gate. |
| CC16 | Both owner question and unsupported method block work. Owner answers the final question, but required method remains unavailable. | waiting-for-owner→capability-blocked. Answer survives; no fake checking or drafting entry is needed. |
| CC17 | While capability-blocked, new current method evidence identifies a source-grounded repair with valid base and authority. | capability-blocked→repairing directly under the explicit guard. |
| CC18 | Fresh authorized scratch creation has intent and policy but no package. Variant creation lacks policy. | First initializes drafting with no-package marker. Second initializes paused(policy-missing). Neither defaults to complete. |
| CC19 | Paused waiting-for-owner receives the applicable answer before resumption, leaving ready checks. | Resume revalidates context and selects checking. It does not require reentering an invalid waiting state. |
| CC20 | Migration target member completes while its required transition member has an unresolved recovery decision. | Preserve target-complete, transition-incomplete, and aggregate-incomplete. No cutover authorization follows. Both use the shared session budget. |
| CC21 | Loop.items has a valid typed supplier from a declared List<Pair> argument port; actual input is not yet supplied. | Authoring checks the supplier contract. It does not invent a list or ask the owner for future caller data. Runtime materialization precedes loop consumption. |
| CC22 | Unknown authoring retry policy is relabeled as a caller supplier, or Known Expression<List> is stored as a List value. | Reject the declaration/slot satisfaction. Required owner policy remains open; expression and evaluated value remain different types. |
| CC23 | Supplier is dangling, type-incompatible, cyclic without state/delay, late, or competes with a constant. | Keep the located source/structure/availability finding. The gate cannot complete that contract. |
| CC24 | Authorized pause/abort, duplicate event, runtime-grant revocation, and governing profile change arrive separately. | Pause retains work; unfinished abort records terminal run-aborted; duplicate is idempotent; runtime revocation affects execution; profile change reopens affected contract work. No caller-selected state or erased historical completion. |

## Task coverage and handoff

| Obligation | Controller evidence | Cases |
| --- | --- | --- |
| T11-001 | Work items, deterministic action selection, CONTROL-003–CONTROL-005 | CC01, CC07, CC12. |
| T11-002 | Serialized current-base application, CONTROL-006 | CC06, CC15. |
| T11-003 | New-revision checks, CONTROL-007/008 | CC01, CC02. |
| T11-004 | Freshness and conservative impact, CONTROL-001/007/015 | CC02, CC13, CC15. |
| T11-005 | Checkpoint and question records, CONTROL-002/020/021 | CC01, CC06, CC12, CC14. |
| T11-006 | waiting-for-owner versus capability-blocked | CC08, CC12. |
| T11-007 | D18 progress fingerprint and CONTROL-017 | CC03. |
| T11-008 | D18 ledger and CONTROL-016/018 | CC04, CC05. |
| T11-009 | Gate and CONTROL-012/013 | CC07, CC08. |
| T11-010 | Historical receipts and CONTROL-002/010/011 | CC04. |
| T11-011 | Completion object, CONTROL-001/014 | CC09, CC10, CC11. |
| T11-012 | Reopening and CONTROL-015 | CC13. |

T12 consumes only the revision-bound completion object and separate execution authority.
T13 consumes checkpoint, journal, atomic revision application, context, and budget-persistence requirements.
T14 simulates these cases and distinguishes authoring completion, target continuation, implementation evidence, and conceptual acceptance.
D16–D18, accepted T06–T10 outputs, and independent review remain closure gates.
The inspected T09/T10 records now reconcile the session's baseline, coverage, target, and separate transition handoffs.

## See also

- [contract-model.md](contract-model.md): contract representation.
- [tool-protocol.md](tool-protocol.md): tool interfaces.
- [CONTRACT.md](../../CONTRACT.md): method rules.
