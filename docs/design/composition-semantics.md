# Composition semantics

Revision: composition-draft-8. Status: provisional design. Designer: `the designer`. Reviewer: `the reviewer`.

This document answers T04 revision 2, SHA-256 `ad292b7ccb6c73c1efd378f0766944ca9be589684dd7b64ef68a7cbd840f7512`.
It proposes D06/D07. Neither has owner acceptance. No executable analyzer or universal proof is claimed.

## Inputs and boundaries

The [model](contract-model.md), `model-draft-2`, has SHA-256 `f307de558f9ae90038d376bc8b77f0f9fd3d0609c5f60d6fe0af5f21c91cf7e6`.
Its exact references, typed bindings, value states, and assessment limits apply provisionally.
T02 and T03 remain close gates. Their acceptance is absent.
The hashes above identify historical drafting inputs. T14 selects the revised model/catalog phase rules through its exact baseline and compatibility map.

T03 supplies explicit child roles, slot types, effects, exits, and checking requirements through its provisional catalog.
The inspected `patterns-draft-5` bundle manifest has SHA-256 `83c5bd9f2b57d932c48d90b3c70d8f2773c6c7e662b19e7988c2efbb18843b4a`.
EXP-EVAL, FLOW-LOOP, FN-OPERATION, CON-TASK, and CON-COORDINATE use `p2`. Other selected definitions retain `p1`.
Relevant proposed IDs include `FLOW-LOOP`, `FLOW-BRANCH`, `FLOW-EXIT`, `FN-OPERATION`, `FAIL-RETRY`, `FAIL-CANCEL`, and `FAIL-CLEANUP`.
`FAIL-EXCEPTION`, `ABS-INHERIT`, `STATE-RESOURCE`, and external-effect patterns supply their corresponding local obligations.
The catalog remains a design-output input. Final review requires its exact artifact identity and compatible role definitions.

Owner sources are B07, B20, B22 and the supplied explanation.
The owner example below retains unknown decisions. Separate hypothetical fixtures supply review values only.

Included scope: structural relations and their behavior. Language mappings, tool transport, controller scheduling, and live effects belong to other tasks.

## Typed relations

An instance is one exact PatternInstance revision. A frame is its execution context during one invocation.
A definition can serve several instances. Each invocation receives a separate frame unless an explicit sharing relation states otherwise.
A trace is the ordered results, state changes, and effects admitted by the contract.
A transfer is an explicit normal or abrupt exit with a target and payload.

Every relation uses T02 common fields plus `relationType`, `from`, `to`, and `bindings`.
The author owns relation definitions and bindings. Each field is required, except fields explicitly marked optional below.
Missing references, duplicate identities, wrong target kinds, or undeclared binding names produce structural findings.
Every canonical entity reference identifies the entity, exact revision, and package. A display name alone cannot identify a target.
Slots, runtime frames, and activations are not new T02 entity kinds.
An `EntityLocation` contains `entity:Ref<Entity>` and `slotPath:List<Name>`, each once.
The path identifies declared fields within that entity. An empty path addresses the entity itself.
A slot location resolves through its enclosing instance and definition, rather than through an unsupported Slot entity reference.

A runtime `FrameKey` contains `run:ID`, `invocation:ID`, `instance:Ref<PatternInstance>`, and `parent:FrameKey|initial`, each once.
The controller or execution host allocates a unique invocation ID before entry and preserves it across resumption.
Iteration and recursive calls receive distinct invocation IDs. They do not change the static instance's entity identity.
Runtime frames instantiate canonical ContextBinding data. They are execution records, not canonical package entities.

| Relation type | Source → target | Required typed bindings and meaning |
| --- | --- | --- |
| contains | Parent instance → child instance | `role:Name`, `position:nonnegative Integer`, `context:ContextBinding`. Select a declared child role and its order. |
| precedes | Instance → instance | `when:Predicate`, `completion:normal\|any-terminal`. State the required first completion and branch where ordering applies. |
| binds | Source-owning entity → consumer or result-owning instance | `source:ValueEndpoint`, `target:ValueEndpoint`, `mapping:Expression<T>`, `phase:EvaluationPhase`, `failure:TransferPolicy`. Endpoints identify declared values rather than invented record fields. |
| invokes | Calling instance → operation instance | `arguments:Map<Name,ArgumentMapping>`, `resultTarget:EntityLocation`, `failureTarget:EntityLocation`, `continuation:ContinuationPolicy`. Invocation does not imply containment. |
| handles | Failure source → handler instance | `errorType:Ref<Type>`, `guard:Predicate`, `priority:Integer`, `resume:propagate\|return\|continue`, `resumeTarget:Ref<Instance>`. Define interception and continuation. |
| inherits | Extension → base contract revision | `extensionPoint:Name`, `binding:Map<Name,Binding>`, `preserved:Set<Ref<Clause>>`. Empty preserved set cannot establish substitution. |
| owns | Owning instance → resource instance | `acquisition:Ref<Instance>`, `release:Ref<Instance>`, `lifetime:ScopeRef`, `releaseFailure:TransferPolicy`. Each activation instantiates its ownership. |
| shares | Accessing instance → state-owning instance | `statePath:List<Name>`, `access:read\|write\|read-write`, `ordering:Predicate`, `atomicity:Text`, `invariant:Set<Predicate>`. State permitted interference. |
| cancels | Requesting instance → cancellation boundary | `mode:cooperative\|abort-before-effect`, `acknowledgment:Predicate`, `inFlight:TransferPolicy`, `effects:EffectPolicy`. Define cancellation scope and race outcome. |

`Ref<Instance>` abbreviates T02 `Ref<PatternInstance>`. The resource and state owners are instances of their declared prepared patterns.
`ScopeRef` contains `owner:Ref<PatternInstance>` and `activationPath:List<Name>`, each once.
Its path names declared parent-context links from the current frame to one active owner. A missing or ambiguous path is invalid.
The corresponding runtime FrameKey identifies that owner's particular activation. No canonical reference points directly to a FrameKey.
`TransferPolicy` maps each admitted transfer and known effect state to one typed continuation or result.
`EffectPolicy` names admitted effects, retained effects after interruption, and required recovery responsibilities.
An undefined policy branch is unresolved meaning. Text naming an unspecified policy does not fill that branch.
`ContinuationPolicy` selects terminating or ongoing behavior. Terminating requires its declared total-return predicate.
Ongoing requires active invariants, observation boundaries, explicit stop/cancellation target, acknowledgment policy, and the final observable after acknowledgment.
No global program deadline or termination guarantee follows from the controller's separate contract-authoring budget.

`ContextBinding` contains `imports:Map<Name,ScopedLocation>`, `constants:Map<Name,Binding>`, `parameters:Map<Name,FormalParameter>`, `state:Set<EntityLocation>`, and `exits:Set<ExitBinding>`.
Each field occurs once. Empty collections import, bind, share, or permit nothing respectively.
Each imported or shared entity carries a type, access mode, lifetime, and owner. A missing property prevents context checking.
ScopedLocation contains an exact declaration EntityLocation and owning ScopeRef selecting its activation.
Constant canonical values can instead use explicit NotApplicable scope with proof that no runtime activation is required.
Runtime imports resolve only within that selected active FrameKey. A same-named declaration in another frame does not satisfy the import.

An `ExitBinding` contains `kind`, `target:Ref<PatternInstance>`, `activationPath:List<Name>`, admitted `payloadType`, and `cleanupPath`, each once.
The activation path resolves exactly one active FrameKey through declared context links. Its instance reference equals target.
The path never searches for the nearest matching name or recursive activation. An unresolved activation blocks the transfer.
`cleanupPath` is the ordered resource-owner scopes crossed before target delivery. Empty means no owned scope is crossed.
All values use T02 Known/Unknown/Absent/NotApplicable states. Unknown never selects a policy branch implicitly.

### Static declarations and runtime values

A FormalParameter is contained record data, not a T02 entity, constant Binding, or runtime value.
Its required fields are `name:Name`, `type:Ref<Type>`, `source:ParameterSource`, `mapping:Expression<T>`, `phase:entry`, `access:immutable|declared-mutable`, and `lifetime:ScopeRef`.
The author owns all fields. Missing type/source/lifetime prevents context checking. No parameter value defaults implicitly.

ParameterSource is either an invocation argument or a parent role input.
An argument source names the exact operation instance and argumentPath into its signature input type.
It also names the static activationPath to that operation's frame. A direct contains child uses the declared parent link.
An empty argumentPath selects the whole typed argument. The signature's inputs field names a Type, not an argument value.
A role source names the exact parent instance, declared child role, and payloadPath into that role's admitted input type.
Its activationPath identifies the exact active parent frame before role entry. It never searches for a matching instance name.
For a loop body, the role payload is the current selected item. It is not the entire input list or an invented slot.
The explicit mapping is identity for matching types, or a declared value-preserving domain inclusion/conversion checked before materialization.

ArgumentMapping is either Constant(TypedValue) or FromLocation(EntityLocation, typed mapping expression).
The caller evaluates mappings in declared order before callee entry. Their source frames and effects obey the ordinary availability rules.
An invocation validates argument presence, source-path existence, and types before creating the body environment.
Invalid or missing arguments follow the operation's declared rejection observable before body actions or effects.

At entry, the host associates the FormalParameter location with the exact FrameKey and admitted TypedValue of its declared type.
The runtime environment stores that value for the declared lifetime. Immutable parameters cannot be reassigned.
Declared-mutable parameters require explicit state ownership, write ordering, and invariant rules. The flag alone grants no shared mutation.
Children see only explicitly imported parameters after materialization. Siblings do not gain parameter visibility implicitly.

A name expression targeting a constant Binding reads its Known typed value.
A name expression targeting a FormalParameter type-checks against the declaration, then reads that invocation's materialized runtime value.
An unentered frame, expired parameter, or unavailable source produces the declared runtime failure rather than an owner-meaning decision.
Future caller data is not an authoring Unknown when its source, domain, and behavior are already specified.

EntityLocation paths are logical schema paths. Before each component, resolution unwraps a Known Binding's TypedValue record payload.
It never follows an arbitrary entity reference or unwraps Unknown, Absent, or NotApplicable into a record.
The resulting location identifies either declared data or the exact named declaration. Type checking rejects an unexpected target kind.
Thus [bindings,context,parameters,input] can select a FormalParameter inside a relation's Known ContextBinding record.

### Runtime slot suppliers

ValueEndpoint is a tagged declaration location, runtime port, or consumer data slot. Its tag determines its admissible fields and phase.
These exact logical field names define the endpoint shape. All listed fields have cardinality one unless explicitly conditional.

| kind | Exact fields after kind | Rules |
| --- | --- | --- |
| declaration-location | `location:EntityLocation` | The location ends at a visible Binding or FormalParameter declaration. No runtime frame is a canonical entity. |
| runtime-port | `instanceRef:Ref<PatternInstance>`, `activationSelector:StaticSelector`, `port:PortName`, `fieldPath:List<Name>`, conditional `role:Name` | Fields are flat. role is required only for role-input/role-result and absent otherwise. Empty fieldPath selects the complete typed port value. |
| data-slot | `instanceRef:Ref<PatternInstance>`, `slotName:Name` | The named slot exists and permits the declared supplier phase. No activationSelector or fieldPath is present. |

Unknown fields or tags are invalid. The author owns each endpoint and every required field.
For declaration-location, from equals location.entity. For runtime-port, its owner equals instanceRef. For data-slot, to equals instanceRef.
A result-mapping target runtime-port likewise requires to=target.instanceRef. Ownership mismatch is a structure finding before evaluation.
A declaration location is the canonical EntityLocation of a constant Binding or FormalParameter declaration.
A RuntimePort names an exact instance, static activation selector, port name, and fieldPath into that port's declared type.
Port names are argument, result, failure, role-input, or role-result. Role ports name the exact declared child role.
Their types come from the selected signature, child role, or prepared definition's declared input/output semantics.
An expression's resultType configures its result port. A value constructor's declared output semantics identifies its constructed value type.
For VAL-VARIANT, the exact alternatives/payload types and explicit output-type identity determine its result port, such as NResult.
None of these ports is an extra canonical PatternInstance field.
They are runtime interface ports, not extra fields of PatternInstance or new T02 entities.
A DataSlot endpoint names an exact consumer instance and one declared runtime-enabled slot.
EvaluationPhase names entry, before-consumption, selected-role-entry, or result-delivery, with explicit ordering and branch applicability.
The from reference equals the source endpoint's canonical owner. For a FormalParameter inside a Relation, that owner is the containing Relation entity.
The to reference equals the consumer/result endpoint's exact instance. Runtime frames never appear as canonical from/to entities.
Concrete type binds-supplier specializes binds with a DataSlot target. Package.relations indexes these suppliers for required-slot satisfaction.
Concrete type binds-result specializes binds with a declared RuntimePort target. It maps result delivery without supplying a pattern configuration field.

A result port differs from FN-OPERATION.result, which configures its Type. An argument port differs from FN-OPERATION.inputs for the same reason.
Result-delivery mapping writes the declared observable port after its producer finishes. It cannot change the configured result Type.
An unselected branch's suppliers do not execute. A selected branch cannot consume its slot before required suppliers finish.
An expression provider that has no existing result requires sourceContext:ContextBinding in its supplier bindings.
The declared phase allocates its evaluation frame, resolves scoped imports, evaluates the prepared expression, and then delivers its typed result.
SourceContext capture scopes resolve from the consumer's active frame before allocating the provider frame, whose parent is that consumer frame.
Missing sourceContext, unavailable capture, or missing ordering is a finding. A supplier cannot evaluate an unselected branch's expression early.

A data slot can use a declared runtime supplier only when its selected definition admits that evaluated-data phase.
The canonical package records a typed binds relation naming exact source, target slot, mapping, evaluation phase, and availability conditions.
This relation contains a known mapping description. It does not store Expression<T> as a Known value of type T.
The required slot has exactly one source: an admitted constant Binding or one declared runtime supplier.
No source, several suppliers, or a constant competing with a supplier creates a located structure/typing finding.
Dangling paths and a producer unavailable before consumption also create findings.
Supplier dependency cycles require explicit state/delay semantics. An unqualified self-cycle cannot produce a value before its own input exists.

Authoring checks the supplier's declared source type, output type, visibility, ordering, admitted values, and failure handling.
At invocation, the host evaluates the source and mapping, validates TypedValue<T>, and supplies the consumer slot before its declared use.
Failed production follows the declared relation failure policy. The consumer cannot read an unmaterialized slot or replace it with an empty value.
Unknown required owner policy cannot become a runtime supplier without an explicit source-backed permission to parameterize that policy.

Concrete example: R-input declares FormalParameter input:BatchInput from operation's whole invocation argument, with immutable entry lifetime.
Loop.items has a binds supplier from R-input's materialized parameter, mapping identity List<Pair>, before the loop's input evaluation.
For case [Pair(-7,2)], runtime input is that actual typed list. The loop receives a list, not an expression record.
For case [], it receives Known empty List<Pair>. Neither case changes the contract's source mapping or invents an owner answer.

The same rule supplies value/payload slots from operation results. Static definition references and unsupported policy choices cannot bypass their authoring obligations.

COMPOSE-025 | The author must distinguish formal runtime parameter declarations from constant bindings and materialized runtime values.

COMPOSE-026 | The host must validate and materialize admitted parameter values before executing their dependent body actions.

COMPOSE-027 | The checker must require exactly one admitted constant or typed runtime supplier for each required data slot.

COMPOSE-028 | The checker must reject dangling suppliers, premature consumers, invalid types, and undeclared cyclic data dependencies.

COMPOSE-029 | The checker must reject runtime suppliers for declaration, configuration, or policy slots that require authoring resolution.

COMPOSE-030 | The checker must reject nonexistent ports, invalid field paths, and output types unsupported by their prepared definitions.

COMPOSE-001 | The author must bind every child to a declared role and exact parent context.

COMPOSE-002 | The checker must report invalid relation targets, missing bindings, and undeclared roles as structural findings.

## Context, visibility, and execution

A child receives only its explicit imports, parameter bindings, declared shared state, and allowed exit bindings.
The child can access its own local values after their declared initialization point.
An imported value becomes available only after its producing operation completes under the relevant ordering predicate.
Unordered producer/consumer access creates a data-availability finding.

Names are unique within a frame. A child name can match an outer name only with an explicit distinct local binding.
References identify their declaring frame. The checker never chooses the nearest same-named value as an implicit replacement.
A closure lists captured entity revisions, value/reference capture mode, lifetime, and permitted mutation.
An escaping reference cannot outlive its owner. A declared ownership transfer can create another owner before escape.

Every sibling ordering follows its role and precedes relations. Unordered siblings permit only declared concurrent interleavings.
Conflicting writes require an explicit coordination rule and invariant boundaries. Declared concurrency alone supplies neither.
An inner action cannot weaken outer preconditions, invariants, effect limits, deadlines, cleanup, or completion obligations.

COMPOSE-003 | The checker must resolve names through explicit frame bindings without inferred shadowing.

COMPOSE-004 | The author must define each value's availability, access mode, and lifetime across a binding edge.

COMPOSE-005 | The author must preserve outer invariants and effect restrictions when adding child behavior.

COMPOSE-006 | The author must define ordering and invariant boundaries for shared mutable state.

## Exit targets and cleanup

| Transfer | Required target | Propagation and permitted observable |
| --- | --- | --- |
| Normal | Declared next role or operation result | Deliver typed result after required local cleanup. No implicit sibling skip. |
| return | Exact enclosing operation invocation | Stop its unfinished sequential work. Cross listed scopes, perform cleanup, and deliver its declared result. |
| break | Exact active enclosing loop instance | Stop that loop. Perform crossed cleanup. Produce its declared loop result, then follow its declared successor. |
| continue | Exact active enclosing loop instance | Perform crossed iteration cleanup. Apply that loop's declared update, then test its next-iteration condition. |
| throw | Exact handler boundary or exact operation's uncaught-failure result | Search only the explicit handler chain. Select by guard and priority. Cross cleanup scopes before delivery. |
| cancellation request | Exact cancellation boundary, including invocation identity | Record the request. Apply its acknowledgment and in-flight policy. A request alone does not prove work stopped. |
| cancellation acknowledgment | Request's exact boundary and identity | Stop further affected work according to its policy. Retain completed effects and unresolved in-flight effects. |

A target outside the active enclosing context is invalid unless a separately declared nonlocal-control mechanism supports it.
Handler priorities are unique within one boundary. Overlapping guards with no explicit ordering create a conflict finding.
No matching handler propagates to the declared uncaught-failure target. Missing that target is a finding.

Before each transfer, cleanup follows reverse successful-acquisition order across crossed scopes.
A resource enters one of not-acquired, owned, released, or release-failed states for that acquisition identity.
No acquisition means no release call. A successful release occurs once. Repeated transfer delivery cannot release that acquisition again.
A release failure retains its known resource state and follows the explicit releaseFailure policy.
That policy decides whether the original transfer survives, becomes a combined failure, or propagates another declared result.

A child failure does not automatically roll back siblings, shared state, or external effects.
Each parent role states whether it continues, cancels, joins, or stops siblings for each admitted failure.
If siblings already started, the parent accounts for each terminal or unresolved result before declaring its own completion.
Compensation is a separate declared action with its own failures. It cannot erase the original effect history.

COMPOSE-007 | The author must give every abrupt exit one exact target and typed payload policy.

COMPOSE-008 | The checker must reject exit targets outside their declared active context.

COMPOSE-009 | The author must define cleanup and its failure result for every crossed ownership scope.

COMPOSE-010 | The author must define sibling, parent, state, and external-effect outcomes after each admitted child failure.

COMPOSE-011 | The checker must not infer rollback, cancellation acknowledgment, or compensation from a failure result.

## Obligations introduced by edges

Individual pattern completeness does not prove these relational obligations.
Each generated obligation identifies its relation, affected guarantees, required value, responsible actor, and checking method.

| Connected behavior | New obligation |
| --- | --- |
| Producer → consumer | Type compatibility, value availability, conversion loss, and whether failure prevents consumption. |
| Loop → retry → effect | Attempt identity, duplicate-effect bound, retry termination, deadline interaction, and retained effects. |
| Payment success → shipment | Payment identity flows into shipment. Shipment failure needs retained-payment state and a recovery owner. |
| Cancellation → active retry | Define request/acknowledgment order, in-flight completion, effect observation, and whether another attempt can start. |
| Owner scope → abrupt exit | Release each acquired resource once, including failure and cancellation paths. |
| Shared state → concurrent children | Define atomic boundaries, allowed ordering, invariants, conflict outcomes, and cancellation joins. |
| Base guarantee → replacement | Preserve admitted base inputs and guarantees, including effects, failures, and termination. |

COMPOSE-012 | The checker must enumerate obligations introduced by every declared composition edge.

COMPOSE-013 | The author must resolve required edge obligations before accepting the composed concrete contract.

## D06 proposal: structural relationships and bounds

Proposal: containment is a finite rooted forest of instances. Each child has one structural parent in one instance revision.
Definitions can be reused through distinct instances. A contains cycle or two structural parents is a structural contradiction.
Cross-references use typed relations instead of a second structural parent.

Invocation cycles are recursion, not automatically containment errors. Recursive bindings require an admitted domain and explicit continuation semantics.
A selected bounded-recursion definition requires its base case, progress measure, bound, and boundary result.
An ongoing recursive behavior needs a prepared definition with active, observation, and stop semantics. Missing support remains a capability gap.
A semantic recursion bound, when required by behavior, defines its boundary failure and effect state.
A missing required recursion rule remains a finding. An explicitly nonterminating operation cannot satisfy a required termination guarantee.
Inheritance and value-initialization cycles are rejected unless their selected semantic pattern explicitly defines the cycle and its observable behavior.

No universal semantic nesting depth is proposed. The selected language profile can impose a semantic bound with a defined result.
A checker can separately declare its analysis-depth or work limit. Reaching that limit leaves uncovered obligations explicit.
A tool limit cannot truncate the contract or count omitted nested obligations as passed.

COMPOSE-014 | The checker must reject cyclic containment and conflicting structural parent assignments.

COMPOSE-015 | The author must define recursive bindings through an explicit recursion pattern and its required continuation rules.

COMPOSE-016 | The checker must distinguish semantic bounds from analysis limits and preserve uncovered nested obligations.

D06 remains proposed. Exact catalog interfaces and language-specific bounds require downstream reconciliation.

## D07 proposal: inheritance and substitution

An extension names the exact base revision, legal extension point, inherited clauses, and concrete bindings.
A base abstract slot can remain unbound only in an explicitly abstract contract.
A concrete implementation target binds every abstract operation and parameter needed by admitted behavior.

For every base-admitted input, a substitute admits that input and preserves the base's permitted trace constraints.
Its precondition cannot strengthen the caller's obligations. Its guarantees cannot weaken results, failure behavior, declared continuation, invariants, or effect restrictions.
Preserving an ongoing contract means preserving its observation and stop guarantees, without adding a global termination requirement.
A replacement occurs only at a declared extension point. An additional operation cannot change inherited obligations indirectly through shared state.
Several bases contribute all applicable obligations. There is no leftmost-base or latest-definition precedence.
An incompatible requirement becomes a located conflict. Unknown compatibility remains an unresolved checking obligation.
An intentionally different input domain requires a distinct contract boundary, not a silent substitute.

COMPOSE-017 | The author must identify exact base revisions, inherited obligations, and permitted replacement points.

COMPOSE-018 | The reviewer must reject substitutes that strengthen base caller obligations or weaken base guarantees.

COMPOSE-019 | The checker must preserve inherited conflicts instead of selecting an arbitrary precedence.

COMPOSE-020 | The author must bind required abstract slots before treating a contract as a concrete implementation target.

D07 proposes explicit conjunction and behavioral substitution. Owner acceptance remains absent.

## Unknowns, contradictions, and tool limits

| Situation | Conceptual finding | T06 assessment meaning |
| --- | --- | --- |
| Required target or policy missing | Located structural or missing-meaning finding | A supported structure/slot check can fail. This does not prove semantic contradiction. |
| Known constraints admit no required trace, with a witness or supported proof | Proven contradiction with involved clauses and method | Failed composition check within the proof's stated coverage. |
| Known predicates exceed the method's decision capability | Undecided constraint and exact affected guarantee | Indeterminate from a supported method, or unsupported when no method covers it. |
| Owner value is Unknown | Open decision and dependent obligations | No successful completion evidence for dependent checks. Do not invent the value. |
| Requested nesting exceeds the method's declared supported depth | Located static capability gap and uncovered obligations | Unsupported. No semantic truncation or fabricated check success. |
| Tool exhausts elapsed time, memory, or work during admitted analysis | Unfinished coverage and execution receipt | Execution-error. Completed units remain history under T06's incomplete-invocation rule. |

A contradiction report includes exact references, assumptions, conflicting constraints, witness or proof artifact, and method limits.
An inconclusive check cannot use the contradiction label solely because the tool failed to prove compatibility.

COMPOSE-021 | The checker must distinguish contradictory meaning, missing meaning, unsupported checking, and inconclusive analysis.

COMPOSE-022 | The checker must identify its witness or proof before reporting a proven semantic contradiction.

COMPOSE-023 | The controller must preserve unresolved composition obligations instead of accepting guessed semantics.

COMPOSE-024 | The reviewer must bind composition evidence to exact nodes, relations, definitions, profiles, and decisions.

## Required owner example

These instance names illustrate structure. Their prepared definitions use T03 revisions declared in the input boundary above.

```text
orders: FLOW-LOOP
  eligible: FLOW-BRANCH
    payment: FAIL-RETRY
      capture: FN-OPERATION
      retryEligible: FLOW-BRANCH
    afterPayment: FLOW-BRANCH
      shipment: FN-OPERATION
      shipmentFailure: FAIL-RESULT
```

Orders execute in declared order. Ineligible orders follow a declared skip branch.
Payment success enables shipment. Failed payment does not imply permission to ship.
Successful capture followed by failed shipment retains the capture effect and creates a recovery obligation.
Retry introduces duplicate-effect and identity obligations before its attempts can be considered complete.

| Decision | Explicit unknown value | Behavioral consequence | Responsible owner |
| --- | --- | --- | --- |
| O-ELIGIBILITY | Eligibility predicate and domain | Cannot choose processed orders. | Contract owner. |
| O-ATTEMPTS | Retry count, eligible failures, delay, and deadline | Cannot derive retry termination or start another attempt. | Contract owner. |
| O-PAYMENT-ID | Payment identity and duplicate handling | Cannot establish capture uniqueness across attempts. | Contract owner. |
| O-SHIP-FAILURE | Result and retained-payment state after shipment failure | Cannot derive the composed order result. | Contract owner. |
| O-COMPENSATION | Whether compensation exists, its owner, triggers, and failure outcomes | No refund or rollback can be inferred. | Contract owner. |
| O-CANCEL | Cancellation acknowledgment and in-flight capture policy | A late success leaves effect handling unresolved. | Contract owner. |

These are Unknown cells linked to open decisions, not absent slots or permitted implementation choices.
The populated structure remains incomplete. No invented numeric policy closes the owner scenario.

## Filled hypothetical fixtures

The following values belong only to conceptual review fixtures. They are not answers to O-ELIGIBILITY through O-CANCEL.

| Fixture | Complete declared inputs and policy | Derived trace and expected result |
| --- | --- | --- |
| H1: retry then shipment failure | One eligible order A. One additional attempt, zero delay, 10-unit total deadline. Attempt 1 returns TemporaryFailure with no effect. Attempt 2 returns Captured(P-A). All attempts reuse P-A and the stub permits at most one capture. Shipment returns Failed. Recovery records CapturedShipmentFailed(P-A), without compensation. Each stub finishes within one unit. | Two payment attempts, one capture, one shipment call. Final CapturedShipmentFailed retains P-A. No refund is inferred. |
| H2: nested break | Outer loop O has [1,2]. Inner loop I has [a,b]. Body (1,a) emits break targeting I. Other bodies emit Normal. Both use bounded continuation, ascending order, resultPolicy=discard, and a break result mapped to Completed([]). | Body sequence is (1,a),(2,a),(2,b). The break skips (1,b) and does not stop O. Both expose Completed([]) with effects retained. |
| H3: resource cleanup | Acquire R succeeds once. Body result is either Success or Error. Both cross the same owner scope. Release succeeds and records its acquisition ID. No concurrent transfer delivery exists. | Either input path calls release once. Final body result survives. No path releases before acquisition or twice. |
| H4: late capture after cancellation | One capture is in flight. Cooperative cancellation stops new attempts and shipment at acknowledgment. In-flight capture may finish. It returns Captured(P-A) after acknowledgment. Parent joins it and returns CancelledWithCapture(P-A). Cleanup succeeds. No compensation is defined or performed. | No new attempt or shipment starts. Capture remains visible in the terminal result. Cancellation does not erase its effect. |
| H5: substitution | Base admits integers x>=0 and returns nonnegative result without effects. Child requires x>0. | x=0 is a witness of strengthened precondition. Substitution fails. A separate restricted contract is possible. |

H1's duplicate guarantee is an explicit stub assumption, not proof about a payment provider.
H4 defines acknowledgment and join policy expressly. The unresolved owner scenario does not inherit this fixture's choices.

## Clause-to-case handoff

| T04 obligation | Rules and evidence | Cases |
| --- | --- | --- |
| T04-001 | ContextBinding, availability, visibility, COMPOSE-001–COMPOSE-006 | C02, C06. |
| T04-002 | Exit table, ExitBinding, COMPOSE-007–COMPOSE-009 | C01, C02, C07. |
| T04-003 | Typed relations and edge-obligation table, COMPOSE-012/013 | C05, C08. |
| T04-004 | Parent/sibling policies, effects, cleanup, COMPOSE-009–COMPOSE-011 | C01, C04, C05. |
| T04-005 | Outer context restrictions, COMPOSE-005 | C04, C09. |
| T04-006 | D07, COMPOSE-017/018/020 | C03, C10. |
| T04-007 | D07 conjunction, COMPOSE-019/021 | C03, C11. |
| T04-008 | D06 structural/recursive/tool bounds, COMPOSE-014–COMPOSE-016 | C12, C13. |
| T04-009 | Finding taxonomy and witness requirements, COMPOSE-021/022 | C11, C13. |

| Case | Input | Expected conceptual result |
| --- | --- | --- |
| C01 | H4 late capture after cancellation, then the owner O-CANCEL Unknown variant | H4 derives CancelledWithCapture. Owner variant remains blocked on explicit policy and retained effect. |
| C02 | H2 break inside two loops | Exactly target I. Outer O continues with the derived body sequence. |
| C03 | H5 child rejects base-admitted zero | Located substitution conflict with x=0 witness. |
| C04 | H3 Success and Error body inputs | Exactly one release on both paths, preserving body result. |
| C05 | H1 capture succeeds and shipment fails | Retain P-A and explicit recovery result. Owner variant remains unresolved instead of inventing compensation. |
| C06 | Two frames declare name item but child imports outer item explicitly | Resolve the exact imported reference. No implicit nearest-name selection. |
| C07 | Return/continue/throw have missing targets | Located structural findings. No guessed enclosing target or handler. |
| C08 | Retry and capture have complete local slots but no identity edge | New duplicate-effect obligation blocks composed completion. |
| C09 | Inner child writes shared state outside outer effect allowance | Conflict with the outer guarantee. Inner behavior cannot override it. |
| C10 | Concrete extension leaves required abstract operation Unknown | Block concrete-target acceptance. Abstract status does not imply implementation readiness. |
| C11 | Two inherited result constraints conflict, then a separate unsupported implication question | First needs contradiction witness or proof. Second remains indeterminate/unsupported and cannot become a proved conflict. |
| C12 | Contains cycle, and separately a recursive invocation with base/progress/boundary result defined | Reject structural cycle. Inspect recursion under its explicit semantics rather than rejecting every invocation cycle. |
| C13 | Nested contract exceeds declared method depth, then an admitted check exhausts its time budget | First returns unsupported. Second returns execution-error. Both preserve uncovered obligations without semantic truncation. |
| C14 | Ongoing operation has active invariant and explicit observed stop, without global termination | Its composition preserves continuation. Contract completion does not require executing the target until termination. Missing liveness evidence remains a check gap. |
| C15 | Declared input:List<Pair> supplies Loop.items through identity mapping; actual cases [] and [Pair(-7,2)] | Materialize the actual list before loop evaluation. Static mapping is known; future input is neither a Known expression-as-list nor an owner Unknown. |
| C16 | RetryCount owner value is Unknown, then an undeclared caller source is proposed to bypass it | Reject the supplier for that authoring policy. Preserve the original owner decision. |
| C17 | Required data slot has both constant and supplier, wrong path/type, late producer, or undelayed cycle | Each variant yields its located structural/type/availability finding. No ambiguous or nonexistent runtime value passes authoring checks. |

All case outcomes are designer-derived. Independent review and accepted T02/T03 revisions remain required.
T05 receives scope/exit/effect obligations. T06 receives exact relation targets and uncertain-check categories. T11 receives blocking composition findings.
D06/D07, upstream acceptance, and exact catalog reconciliation remain open closure gates.
