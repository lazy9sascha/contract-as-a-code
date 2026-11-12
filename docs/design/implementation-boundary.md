# Downstream implementation boundary

Revision: `boundary-draft-3`. Status: provisional design. Designer: `the designer`. Reviewer: `the designer`.

This draft answers T12 r3, SHA-256 `d40ad7aaeb26fdd3a21703b32ba8997499b4ea85df1adbeeac86134c386f9a52`.

It defines proposed interfaces and outcomes. It installs no interception layer, runs no action, and claims no enforced runtime boundary.

## Exact inputs and responsibilities

| Input | Provisional identity and effect |
| --- | --- |
| T01 concept | concept-draft-1 `988442c52e9049b5e7cc67b3c2910f823809438a3152f35d0db713407a646f51`: contract authority and separate runtime authorization. |
| T02 model | model-draft-4 `eb8f757e91a8cbfa9c000347d412d954031a8b4a9643b7b374d53563b9fa4623`: exact entities, choices, static/runtime suppliers, evidence and staleness. |
| T05 profiles | profiles-draft-2 `e297021536f7227f3044d3d55b7f7928bd4c18f2629d65f5f0d98bb35527648f`: scoped meanings and explicit source/runtime gaps. |
| T06 protocol | tool-protocol-draft-3 `6f976c1d0bb63085000d6456fae374d12093bb5252bfcab6441f0ab7c276022c`: evidence identity and accepted/rejected/stale receipts. |
| T11 controller | controller-draft-5 `324aeffa90eb3bf40e6a78fcfdcd122b80e8894138d1cb5deacaf3ce8393f8d5`: completion object, typed-supplier gate, ModeSession, reopening and retained history. |

All five inputs are close gates. Independent provisional content review does not establish their owner acceptance or runtime implementation.

The contract harness owns contract completion and delegation eligibility. The implementation harness owns action preflight, execution records, and dependent-action stops.

The host supplies current runtime authorization and enforcement evidence. A contract owner supplies requirements and exact contract acceptance.

No real completed user contract is available here. The filled examples explicitly simulate a completed object and host grants.

## Completion object consumption

T11's Completion object is an immutable record containing these exact conceptual fields. This design references it instead of accepting a Boolean complete flag.

| T11 content | Required T12 use |
| --- | --- |
| Exact package and manifest digest | Every plan, binding and action targets that package revision. Retained bytes must match. |
| Profile | Bind language/settings and action-relevant compatibility. A required unsupported profile guarantee blocks delegation. |
| Catalog/relations | Resolve exact definitions, choices, effects, dependencies and exit/recovery obligations. |
| Criteria and current eligible evidence | Establish the completed target's declared scope and checking limits. Syntax alone is insufficient. |
| Independent reviews | Preserve their identity and coverage. No LLM self-report substitutes for required review. |
| Owner acceptance | Require the exact revision/scope and valid acceptance authority. Mock acceptance is fixture-only. |
| Policy and completion event | Identify the gate rules and retained successful event. An edited package does not inherit the old event. |

Delegation validates all required references and current applicability. Missing, stale, corrupt, or unaccepted completion data cannot authorize implementation.

For a mode with several required contracts, T11's ModeSession links each role to its exact completion run and object.

Delegation requires every required session member complete. A completed migration target alone cannot bypass an incomplete transition contract.

ACTION-001 | The contract harness must require a valid revision-bound completion object before delegating implementation.

ACTION-002 | The contract harness must require exact owner acceptance recorded in that completion object.

ACTION-022 | The contract harness must require all required ModeSession members complete before delegating implementation.

## D19 proposal: action classification and binding

An action can belong to several classes. Classification includes declared transitive effects, not only its tool's display name.

| Action class | Required binding content | Typical interception scope and failure |
| --- | --- | --- |
| Read source or local data | Exact target/boundary, purpose, admitted data class, required observable and governing clause/choice | Read adapter or file boundary. Deny outside target/data scope. |
| Change implementation artifacts | File/artifact targets, allowed change set, relevant guarantee/choice, expected result and validation obligations | Write/patch boundary with old-content identity. Reject stale or unrelated targets. |
| Execute validation/build code | Executable/tool identity, arguments, inputs/outputs, process/network/filesystem effects, time/resource limits | Process boundary plus transitive effect controls. Unknown bypass paths block required delegation. |
| Change runtime state | Target resource/state, admitted transition, concurrency/identity/recovery obligations | State-operation boundary. Nonconforming result retains known state/effects. |
| Cause external effect | Endpoint/recipient/resource, payload class, exact effect identity, allowed outcome, idempotency and failure policy | Network/service/tool boundary. Deny missing authorization or uncontrolled paths. |
| Delegate or schedule work | Child identity/scope, inherited action policy, reachable tool/effect paths and join/cancel responsibility | Launch/delegation boundary. Reject a required path that can bypass enforcement. |

An ActionBinding uses the following fields. The implementation planner proposes it. The contract harness checks its eligibility.

| Field | Type and cardinality | Meaning and missing rule |
| --- | --- | --- |
| id, revision | ID, 1 each | Stable binding identity with immutable revisions. Missing is invalid. |
| completion | RecordRef<T11 Completion object>, 1 | Exact authority target. Missing or stale blocks the action. |
| modeSession | RecordRef<T11 ModeSession>, 0..1 | Required when the mode has several required contract members. Absent is valid for a single-target run only. |
| basis | Set<ActionBasis>, 1..n | Tagged ClauseBasis or ChoiceBasis below. Relevance alone supplies no permission. |
| actionClasses | Set<ActionClass>, 1..n | All declared read/write/runtime/external/delegation effects. Missing classes invalidate coverage. |
| target | Typed target with scope and prestate identity, 1..n | File, data, resource, endpoint or child boundary. Unknown target prevents protected dispatch. |
| intendedObservable | Predicate over declared result/effects, 1 | Required result and effect bounds. Missing prevents conformance review. |
| profile | Ref<Profile>, 1 | Same selected profile/settings or explicit supported host-action profile. No silent substitution. |
| tool | Exact adapter/tool build and admitted operation, 1 | Registered action capability. T06's read-only check catalog does not supply execution capability. |
| dependencies | Set<ActionBindingRef>, 0..n | Predecessors whose conforming results permit this action. Empty declares independence. |
| boundaryRequirements | Set<InterceptionRequirement>, 1..n | Required enforcement tuples and evidence. Missing blocks delegation. |
| runtimeScope | AuthorizationScope, 1 | The protected operation/target/effect scope, or host-classified not-required state. |
| recovery | Defined outcome policy, 1 | Failure, interruption, unknown effects and retry/reconciliation rules. Missing blocks effectful dispatch. |

The binding does not grant runtime authorization. It establishes only contract eligibility within the completed revision's guarantees and declared choices.

RecordRef<T> identifies a retained runtime record by kind, stable ID/revision, owning run/session, and Artifact. It is not a canonical T02 entity Ref.

| ActionBasis variant | Required typed fields | Eligibility rule |
| --- | --- | --- |
| ClauseBasis | clause:Ref<Clause> | Resolve an obligation or permission in the exact completed package. Show this bounded action satisfies its declared behavior. |
| ChoiceBasis | location:EntityLocation, selection:TypedValue, chooser:Ref<Actor> | Resolve a declared Known Choice<T> value, verify selection has exact type T and lies in its admitted domain, and verify chooser authority. |

The planner owns these fields. Missing fields or an Unknown/Absent location block eligibility. Arbitrary Known values do not become implementation discretion.

EntityLocation identifies the containing canonical entity and declared logical path. Choice is typed data at that location, not an invented Choice entity.

All selected clauses, choice constraints, and applicable prohibitions remain conjunctive. A selected permission cannot discard another governing restriction.

A requirement can justify implementation work only through an explicit action-to-guarantee mapping with bounded targets and effects.

It cannot justify arbitrary adjacent behavior, credentials, endpoints, files or background processes. A missing implementation-choice boundary returns to contract work.

ACTION-003 | The implementation harness must bind each action to a clause or declared choice in the completed revision.

ACTION-004 | Each binding must identify its targets, intended observable, exact revision, profile, and required boundaries.

ACTION-005 | The implementation harness must reject missing or invalid bindings before invoking an action tool.

## D20 proposal: interception and coverage

Interception means a host boundary can inspect and deny the scoped operation before its relevant effect. Post-hoc logging alone is insufficient.

An InterceptionRequirement names action class, adapter/build, route, target scope, transitive paths, and required denial/effect coverage, each once.

An EnforcementEvidence record names that tuple, host/configuration revision, retained method/result, covered paths, bypass findings, limits and freshness.

The host owns evidence. A distinct reviewer checks coverage. Every field is required. Unknown path coverage cannot become an empty bypass set.

| Coverage state | Evidence meaning | Delegation outcome |
| --- | --- | --- |
| specified | Required boundary has a conceptual design only. | No enforcement claim or real delegation on that basis. |
| unavailable | No enforcing boundary covers this required tuple. | Reject delegation requiring it. |
| observed-subset | Instrumented evidence covers named routes only. | Admit only a separately complete required subset. Remaining required paths block delegation. |
| enforced-for-tuple | Current host/configuration and declared method establish pre-effect denial for every required path in this tuple. | Eligible within that exact tuple and declared evidence limits. |
| stale | Governing host, adapter, target, route or policy changed. | Reject until current evidence resolves coverage. |

Evidence can include denial-before-call observations, configuration review and controlled bypass attempts. Its method and limits remain explicit.

Finite negative tests alone do not prove arbitrary process containment. Every required transitive path needs an applicable enforcement argument or a gap.

| Path | Required evidence | Current product design state |
| --- | --- | --- |
| Registered direct file/tool calls | Adapter identity, request validation, pre-effect deny, target scope and result record | Specified only. No implementation supplied here. |
| Spawned shell or compiler | Process identity plus filesystem/network/child controls covering declared effects | Unavailable until a host enforcement profile provides evidence. |
| Child agent or scheduled task | Inherited policy, reachable capabilities, no uncontrolled required route, join/cancel record | Unavailable until the host proves that declared scope. |
| Browser/native application/service connector | Exact operation boundary, account/target scope and permission check before effect | Host-specific and unavailable in this product draft. |
| Direct SDK/network route outside wrapper | Applicable host containment or an explicit uncontrolled-path finding | Unknown/unavailable. Voluntary wrapper compliance does not establish enforcement. |

An implementation harness that cannot inspect or deny a required route rejects the delegation. It does not claim control over the rest of the machine.

Newly discovered bypass capability invalidates affected evidence and stops new dependent dispatch. Existing effects remain recorded, not retroactively prevented.

ACTION-006 | The contract harness must retain current interception evidence for every required action boundary.

ACTION-007 | The implementation harness must reject delegation when a required path lacks enforceable interception coverage.

ACTION-008 | The contract harness must report covered and uncontrolled paths without claiming enforcement outside the evidenced tuple.

## D21 proposal: runtime authorization

Runtime authority comes from the execution environment's policy and applicable current user grants. A contract permission is never that authority.

An AuthorizationRecord contains authority reference, policy revision, subject, operation/target/effect scope, validity/revocation state, and decision evidence.

The host owns it. All fields are required. Evidence references protected grants without storing credential secrets in the contract or repository.

Validity covers the dispatch linearization point: the host checks and enforces authorization immediately before the protected effect is admitted.

If it cannot bind the check to that admission boundary, authorization coverage is unsupported. A stale preflight screenshot is insufficient.

| Runtime state | Result before action dispatch |
| --- | --- |
| Current scoped grant already covers the operation | Reuse it after currentness/scope checks. No repeated permission question is required. |
| Host classifies operation as not requiring a protected grant | Record the policy classification and proceed if all other guards pass. |
| Grant missing, expired, scope-mismatched or revoked | Deny before the tool call. Record the specific missing authority and resume condition. |
| Grant status cannot be checked | Block protected action. Do not infer authority from old success or contract acceptance. |

Revocation before admission prevents the effect. Revocation after admission triggers the declared in-flight policy and stops new protected dispatch.

An admitted external effect can already exist. The harness reports that state and any cancellation acknowledgment without promising retroactive prevention.

A DenialRecord contains action/binding/completion references, protected scope, checked authority/policy, denial reason, no-call evidence and required next input.

The implementation harness owns this record. Every field is required. Denial does not change an immutable historical contract-completion result.

ACTION-009 | The implementation harness must check current runtime authorization before each protected action.

ACTION-010 | The implementation harness must reuse applicable scoped authorization without requesting the same permission again.

ACTION-011 | The implementation harness must not infer runtime authorization from contract permission or previous action success.

ACTION-012 | The implementation harness must deny missing or revoked authorization before its protected tool call.

## Action admission and outcome records

1. Validate the exact completion object, required ModeSession members, owner acceptance, binding, profile and plan dependencies.
2. Check current interception coverage for all required action paths and effects.
3. Validate target prestate, tool identity, runtime scope and current authorization at effect admission.
4. Journal action identity, intended effect identity and prestate before dispatch.
5. Invoke only the admitted bounded action and retain its result, known effects and conformance evidence.
6. Start a dependent action only after its required predecessor result is conforming and current.

ActionRequest contains action ID, binding revision, completion reference, exact input/target identities, intended effect identity, and dependency results.

ActionResult contains action ID, admission record, execution state, typed output, known effects, unknown effect scope, conformance assessment and evidence references.

All fields are required and owned by the implementation harness. Empty known effects means observed none under the method, not proved universal absence.

Execution states are denied, not-dispatched, running, succeeded, failed, interrupted, and outcome-unknown. These are separate from T11 contract states.

Conformance is conforming, nonconforming, or unknown for the exact intended observable. Successful process exit alone does not prove conformance.

T06 accepted/rejected/stale describes assessment receipt validity. It is not an execution result or authorization grant.

Read-only T06 checks can assess bounded evidence when their descriptors cover it. New execution adapters need their own accepted callable contracts.

| Outcome | Required continuation |
| --- | --- |
| Conforming current result | Record effects and allow only satisfied dependent edges. |
| Nonconforming result | Retain result, expected/observed difference and known effects. Stop dependent actions. |
| Interrupted or outcome-unknown | Retain journal and uncertain effect scope. Reconcile through declared observation/idempotency policy before any effect replay. |
| Invalid or stale assessment | Retain historical receipt. No current conformance follows. Recheck exact current inputs. |
| Newly unsupported interception/profile path | Stop new affected dispatch and report unsupported delegation. Preserve existing effects. |

Nonconformance does not automatically mean the contract is wrong. Repairing implementation can remain under the same completed revision when its requirements stay unchanged.

If required behavior changes, return to T11 contract work. No implementation repair silently revises the accepted target.

ACTION-013 | The implementation harness must record nonconforming results, known effects, and unresolved effect scope.

ACTION-014 | The implementation harness must stop dependent actions after a nonconforming or unknown required predecessor result.

ACTION-015 | The implementation harness must not claim rollback without a separately defined and evidenced recovery contract.

## D22 proposal: stale revisions and execution runs

An ExecutionRun contains run ID, exact completion reference, action graph, journal, current authorization/interception references, and predecessor run, each once.

It also contains modeSession:RecordRef<ModeSession>[0..1], required for a multi-member mode and otherwise absent.

The initial run explicitly has no predecessor. One execution run targets one immutable completion object or exact completed ModeSession member map.

Every action names its governing member completion. All required session prerequisites remain complete and current before dispatch.

A stale action or plan is rejected before dispatch. It cannot be silently rebound to a newly completed revision.

When governing requirements change, T11 reopens affected contract work and invalidates affected bindings/evidence. The old completion object remains historical.

After the new target completes, create a linked new execution run with new bindings. Preserve previous journals, effect identities and unresolved outcomes.

The new run does not reset an effect's duplicate/retry protection or turn unknown prior outcomes into new authorized effects.

An interruption with the same exact completion target can resume its existing run only after journal integrity, effect-state, freshness and authorization checks.

An outcome-unknown effect cannot resume by blind replay. A valid idempotency or observation/reconciliation contract is required.

ACTION-016 | The contract harness must reject action plans and evidence bound to another completion revision.

ACTION-017 | The contract harness must reopen affected contract work after required behavior or governing input changes.

ACTION-018 | The contract harness must invalidate bindings and evidence affected by the new completion target.

ACTION-019 | The implementation harness must start a linked new execution run after a changed contract revision completes.

ACTION-020 | The implementation harness must preserve effect identities and uncertain outcomes across resumption and new runs.

ACTION-021 | The contract harness must report contract-completion and implementation-execution status separately.

## Filled hypothetical action fixture

Fixture completion C1 contains package P1/manifest M1, profile F1, catalog L1/relations G1, criteria K1, current evidence E1,
distinct review V1, mock owner acceptance A1, gate policy Q1, and completion event CE1. Symbols denote hypothetical complete retained-value assignments.

P1 declares clause C-SAVE and resolved decision I-SAVE. I-SAVE.answer is Known Choice<WritePlan>, with Implementer as its declared chooser.

WritePlan is a typed record containing target, bytes and effects. This choice admits only `/fixture/result.txt`, bytes `ok`, and empty network effects.

I-SAVE retains its question, C-SAVE target/effect links, exact answer source D-SAVE, and resolved status. D-SAVE precedes P1's separate acceptance A1.

Binding B1 targets that file at old digest H0, intended result=bytes `ok`, profile=F1, adapter=FileWrite@v1, dependencies=[], recovery=observe exact bytes before retry.

B1 basis contains ClauseBasis(C-SAVE) and ChoiceBasis({entity:I-SAVE@P1,slotPath:[answer]}, the admitted WritePlan selection, Implementer).

Its required boundary tuple covers FileWrite@v1, that file, denied other paths, no process/network route, and current host configuration HC1.

Mock evidence IE1 establishes enforced-for-tuple under that declared fixture. Mock grant AU1 currently allows that one write under host policy HP1.

Action X1 names B1/C1/H0, input bytes `ok`, effect identity W1, and no predecessor actions. Admission journals X1/W1 before the write.

This is a single-target fixture. B1 and its execution run have no ModeSession. AB-C15 supplies the separate multi-member case.

Expected result: file contains `ok`, known effect W1, conformance=conforming. No actual file, adapter, permission or write exists from this fixture.

## Acceptance and recovery cases

| Case | Input/change from declared fixture | Expected conceptual result and evidence |
| --- | --- | --- |
| AB-C01 | X1 uses current C1/B1, HC1/IE1 and scoped current AU1. | Check current grant and dispatch once without asking again. Record conforming `ok` and W1. |
| AB-C02 | Remove B1's clause/Choice basis. | Deny before FileWrite call with missing-binding record. Existing runtime grant cannot repair contract eligibility. |
| AB-C03 | Plan names C0 while current target is C1. | Reject stale plan/evidence. No silent revision substitution or write. |
| AB-C04 | Replace adapter with a child process that can write other files or reach network outside its required boundary. | Required interception coverage is unsupported. Reject delegation. Wrapper logging supplies no arbitrary containment proof. |
| AB-C05 | AU1 is missing or revoked before effect admission. | No protected tool call. Retain denial/policy/currentness evidence. Contract P1's historical completion remains separate. |
| AB-C06 | Write records W1 but result bytes are `bad`, violating intended observable. | Record W1, nonconformance and observed bytes. Stop dependent publish action. No rollback claim. |
| AB-C07 | Owner changes P1's required output to `new`. | T11 reopens P2 work, invalidates affected B1/evidence, and retains C1. After P2 completes, use a new linked execution run. |
| AB-C08 | Process stops after W1 admission with no final output record. | Preserve outcome-unknown and effect identity. Observe/reconcile before replay. A new run cannot create a fresh effect key to bypass this rule. |
| AB-C09 | Permission is valid during planning but revoked before dispatch. | Admission check denies. A planning-time grant check is not sufficient. |
| AB-C10 | Revocation arrives after external effect admission. | Stop new dependent dispatch, apply in-flight cancellation policy and report known/unknown effects. Do not claim the admitted effect never occurred. |
| AB-C11 | Completion object lacks exact owner acceptance or required profile guarantee is unsupported. | Reject delegation before checking runtime grant. A tool pass or mock LLM statement cannot repair completion authority. |
| AB-C12 | Same C1 run resumes after interruption with known completed W1 and intact journal. | Reuse completed action record after currentness checks. Do not perform the write twice. |
| AB-C13 | Enforcement evidence covers one direct adapter but required browser/native route is uninspected. | Admit no delegation requiring the uninspected route. Report exact covered and unknown paths. |
| AB-C14 | Process exits successfully but conformance method is unavailable. | Execution=succeeded, conformance=unknown. Stop dependent actions requiring that result. Contract completion status is reported separately. |
| AB-C15 | Migration target completion exists, but its required transition member is still incomplete. | Reject delegation under the ModeSession guard. Runtime permission cannot bypass the missing required contract. |
| AB-C16 | ChoiceBasis points to an ordinary Known WritePlan instead of a declared Choice<WritePlan>. | Reject basis type. Known data does not grant discretion or runtime authority. |
| AB-C17 | ChoiceBasis selects `/fixture/other.txt` outside the admitted domain. | Reject before dispatch despite a valid file-write runtime grant. Permission and target bounds remain distinct. |

These cases evaluate interface guards and expected records. They are not executed permission, interception, adapter or fault-injection tests.

## T12 coverage and handoff

| Obligation | Design evidence | Cases |
| --- | --- | --- |
| T12-001 | Six action classes and transitive effect classification. | AB-C01/04/13. |
| T12-002 | Binding basis, ACTION-003. | AB-C01/02. |
| T12-003 | Binding target/observable/completion/profile, ACTION-004. | AB-C01/03. |
| T12-004 | Preflight and ACTION-005. | AB-C02. |
| T12-005 | EnforcementEvidence and ACTION-006. | AB-C04/13. |
| T12-006 | Coverage states and ACTION-007. | AB-C04/13. |
| T12-007 | Authorization admission and ACTION-009. | AB-C01/05/09. |
| T12-008 | Separate authority and ACTION-011. | AB-C02/05. |
| T12-009 | DenialRecord and ACTION-012. | AB-C05/09. |
| T12-010 | Exact completion references and ACTION-016. | AB-C03. |
| T12-011 | ActionResult and ACTION-013. | AB-C06/08/10. |
| T12-012 | Dependency gate and ACTION-014. | AB-C06/14. |
| T12-013 | Required change returns to T11, ACTION-017. | AB-C07. |
| T12-014 | Revision impact and ACTION-018. | AB-C07. |
| T12-015 | Independent execution/conformance/completion states, ACTION-021. | AB-C05/14. |
| T12-016 | Per-route map and ACTION-008. | AB-C04/13. |
| T12-017 | D22 linked new run versus same-target resume, ACTION-019/020. | AB-C03/07/08/12. |
| T12-018 | The complete case table traces action, authority, coverage and evidence consequences. | AB-C01–17. |
| T12-019 | Completion-object owner-acceptance guard, ACTION-002. | AB-C11. |

D19–D22 remain proposals. All current interception states are conceptual/unavailable, not actual enforced-for-tuple claims.

T13 receives action adapters, pre-effect admission, journal/recovery and host evidence requirements. T14 receives these cases and limitations.

Upstream acceptance, exact owner decisions and independent review remain open. This document cannot establish runtime enforcement or conceptual 0.1.0 completion.

## See also

- [CONTRACT.md](../../CONTRACT.md): method rules.
- [concept.md](../concept.md): product concept.
