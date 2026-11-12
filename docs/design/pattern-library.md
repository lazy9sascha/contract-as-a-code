# Prepared pattern library

Revision: `patterns-draft-7`. Status: provisional design. Designer: `the designer`. Reviewer: `the designer`.

This draft answers T03 r2, retained SHA-256 `7b687d6d47e3ef69057da4e937cbe526989ebb8e95efcfe3acad0a698a1e1e9b`.

## Authority and limits

The owner requires prepared programming counterparts across a broad library. This proposal supplies 34 entries across all ten task families.

It does not claim all patterns, all languages, accepted semantics, implemented tools, or executed examples.

Inputs are provisional [T02](contract-model.md), `model-draft-4`, SHA-256 `eb8f757e91a8cbfa9c000347d412d954031a8b4a9643b7b374d53563b9fa4623`,
and [T06](tool-protocol.md), `tool-protocol-draft-3`, SHA-256 `6f976c1d0bb63085000d6456fae374d12093bb5252bfcab6441f0ab7c276022c`.


## Definition interface

A PatternDefinition is a T02 entity. Revised entry identities appear below. All unlisted entries retain p1.

Each row's ID is its logical identity. Its kind is PatternDefinition. Source is B07/B19–B23 plus T03 r2 and the declared proposal case.

Initial p1 entries have empty lineage. Each revised entry has one revises relation to the following exact predecessor.

| Current definition | Predecessor and canonical locator | Retained predecessor package manifest |
| --- | --- | --- |
| EXP-EVAL p3 | EXP-EVAL p2, T03-draft-5/patterns/expressions.md | `83c5bd9f2b57d932c48d90b3c70d8f2773c6c7e662b19e7988c2efbb18843b4a` |
| FLOW-LOOP p3 | FLOW-LOOP p2, T03-draft-5/patterns/control.md | `83c5bd9f2b57d932c48d90b3c70d8f2773c6c7e662b19e7988c2efbb18843b4a` |
| FN-OPERATION p2 | FN-OPERATION p1, T03-draft-1/patterns/functions.md | `16c7a743fd58dabcb0c8a3730e4c7ed438182eb40459dca542e35a81be24bbf7` |
| CON-TASK p2 | CON-TASK p1, T03-draft-1/patterns/concurrency.md | `16c7a743fd58dabcb0c8a3730e4c7ed438182eb40459dca542e35a81be24bbf7` |
| CON-COORDINATE p2 | CON-COORDINATE p1, T03-draft-1/patterns/concurrency.md | `16c7a743fd58dabcb0c8a3730e4c7ed438182eb40459dca542e35a81be24bbf7` |
| FN-CALLBACK p2 | FN-CALLBACK p1, T03-draft-5/patterns/functions.md | `83c5bd9f2b57d932c48d90b3c70d8f2773c6c7e662b19e7988c2efbb18843b4a` |
| EXT-COMPENSATION p2 | EXT-COMPENSATION p1, T03-draft-5/patterns/effects.md | `83c5bd9f2b57d932c48d90b3c70d8f2773c6c7e662b19e7988c2efbb18843b4a` |
| VAL-LITERAL p2 | VAL-LITERAL p1, T03-draft-5/patterns/values.md | `83c5bd9f2b57d932c48d90b3c70d8f2773c6c7e662b19e7988c2efbb18843b4a` |
| VAL-RECORD p2 | VAL-RECORD p1, T03-draft-5/patterns/values.md | `83c5bd9f2b57d932c48d90b3c70d8f2773c6c7e662b19e7988c2efbb18843b4a` |
| VAL-COLLECTION p2 | VAL-COLLECTION p1, T03-draft-5/patterns/values.md | `83c5bd9f2b57d932c48d90b3c70d8f2773c6c7e662b19e7988c2efbb18843b4a` |
| VAL-VARIANT p2 | VAL-VARIANT p1, T03-draft-5/patterns/values.md | `83c5bd9f2b57d932c48d90b3c70d8f2773c6c7e662b19e7988c2efbb18843b4a` |
| FLOW-MATCH p2 | FLOW-MATCH p1, T03-draft-5/patterns/control.md | `83c5bd9f2b57d932c48d90b3c70d8f2773c6c7e662b19e7988c2efbb18843b4a` |
| FLOW-EXIT p2 | FLOW-EXIT p1, T03-draft-5/patterns/control.md | `83c5bd9f2b57d932c48d90b3c70d8f2773c6c7e662b19e7988c2efbb18843b4a` |
| FAIL-RESULT p2 | FAIL-RESULT p1, T03-draft-5/patterns/failure.md | `83c5bd9f2b57d932c48d90b3c70d8f2773c6c7e662b19e7988c2efbb18843b4a` |
| FAIL-EXCEPTION p2 | FAIL-EXCEPTION p1, T03-draft-5/patterns/failure.md | `83c5bd9f2b57d932c48d90b3c70d8f2773c6c7e662b19e7988c2efbb18843b4a` |
| EXT-IO p2 | EXT-IO p1, T03-draft-5/patterns/effects.md | `83c5bd9f2b57d932c48d90b3c70d8f2773c6c7e662b19e7988c2efbb18843b4a` |
| EXT-PERSIST p2 | EXT-PERSIST p1, T03-draft-5/patterns/effects.md | `83c5bd9f2b57d932c48d90b3c70d8f2773c6c7e662b19e7988c2efbb18843b4a` |

These paths resolve within retained RUN-002 artifact bundles. The manifest plus definition ID/revision identifies each predecessor without reusing an identity.

| Field | Type and cardinality | Owner and missing rule |
| --- | --- | --- |
| identity | T02 entity ID, revision, kind, source, lineage | Library author. Common T02 rules apply. Kind is PatternDefinition. |
| purpose | NonemptyText, 1 | Library author. Missing prevents selection. |
| slots | Map<Name,SlotDeclaration>, 1 | Library author. Missing prevents instantiation. |
| semantics | Defined input-to-observable relation, 1 | Library author. Undefined meaning is a gap. |
| childRoles | Map<Name,ChildRole>, 1 | Library author. Empty explicitly permits no children. |
| contexts | Set<ContextTag>, 1..n | Library author. Missing prevents placement. |
| failures | Defined tagged failure policy, 1 | Library author. Explicit none is permitted. Missing is unresolved. |
| continuation | Termination predicate, finite bound, or explicit ongoing policy, 1 | Library author. Missing prevents behavior review. |
| checks | Set<CheckRequirement>, 1..n | Library author. Missing prevents evidence planning. |
| positive | Filled instance and expected result, 1 | Library author. Missing prevents entry review. |
| negative | Changed binding and expected finding, 1..n | Library author. Missing prevents entry review. |
| maturity | Specification, implementation, test coverage tuple, 1 | Maintainer. Missing cannot mean support. |

A ChildRole contains `type:ChildKind`, `multiplicity:minimum/maximum`, `order:OrderPolicy`, `visible:Set<Name>`, and `exits:Set<ExitKind>`, each once.

Its author owns those fields. Missing fields block composition review. Empty visibility grants no local bindings. Empty exits grants no nonlocal exit.

The entry's roles/context column supplies these fields through the common rules below and named per-entry exceptions.

Context tags are expression, statement, operation, handler, type, module, and contract. Child kinds are Expr<T>, Pred, Op<I,O,E>, and Definition.

An Op reference includes exact input/result types, admitted effects E, continuation, and tagged failure results. Unknown components remain decisions.

A CheckRequirement contains category ID, method obligation, target path, and evidence limit. Each has cardinality 1 and belongs to the library author.

Category IDs are T06 `structure.inspect`, `slots.check`, `names.resolve`, `composition.check`, `profile.check`, `cases.evaluate`, and `revision.impact`.

They name proposed responsibilities. No registered checker implementation follows from these names.

PATTERN-001 | The library author must define every entry field before advertising that entry as specified.

PATTERN-002 | The skill must instantiate exact definition revisions instead of inventing slot schemas.

PATTERN-003 | The checker must locate missing bindings by instance ID and slot path.

PATTERN-004 | The skill must retain unknown capabilities as gaps without replacing their semantics with unrestricted prose.

## Slot and example conventions

`R` means required with true requiredness. `C(predicate)` means required exactly when the named predicate holds. `O` means optional.

Each row lists all behavior slots. Every entry also has O `annotation:Text`, with absence meaning no non-normative annotation.

Annotation permits empty text. It changes no behavior. Every other Text slot uses NonemptyText unless a row explicitly permits empty.

All defaults are absent. A row supplies no default implicitly. Every example leaves annotation absent.

An R slot without a static value or valid admitted runtime supplier is missing-required. A false C guard permits declared NotApplicable.

Unknown requiredness creates a finding. T02 Known, Unknown, Absent, and NotApplicable rules govern every binding.

Conditional guards evaluate their parent discriminants first. A false parent disables descendant slots without evaluating absent child discriminants.

For EXP-EVAL, kind=name disables operator, shortCircuit, conversion, loss, and rounding. Missing kind instead leaves affected requiredness unresolved.

For kind=operator, only and/or enables shortCircuit. For kind=convert, only declared-rounding enables rounding. This rule creates no implicit values.

Lists and sets admit empty exactly when their declared minimum is zero. Scalars have no empty value unless their type explicitly admits one.

Every declared list is finite. Canonical references resolve exact entity revisions. Positive static cells use Known values.

Source-bound positive cases instead use the explicit supplier rules below. They never label a future value as Known.

Negative instances copy their positive instance and change only the named binding. Other findings remain unchanged.

Child roles are ordered as their slot list unless a row explicitly permits concurrency. Each child receives only its declared inputs and lexical bindings.

Absent child roles permit no children. Nonlocal exits require an explicit target and the role's declared exit permission.

`pure` means no external effects. `none` is an explicit empty effect set or no-child declaration, not an unknown value.

Types: Unit has value unit. Bool contains false and true. Int uses exact mathematical integers. Nat contains zero and positive integers.

An uninhabited fixture type uses T02 Scalar Int with domain predicate false, exact precision, dimensionless unit, and integer equality.

Duration is Nat milliseconds. PositiveDuration excludes zero. Result<T,E> has exactly Ok(T) or Error(E). Ref, Type, Predicate, Expression follow T02.

Types with finite machine bounds require an explicit overflow policy. This library does not infer a language's numeric rules.

Fixture operations: Id returns its Int input. Yes returns true from Unit. No returns false from Unit. Done returns unit from Unit.

These operations terminate, are pure, and have no failure result. Fixture Emit records one named event and returns unit, then terminates.

Fixture Try returns its supplied Ok(unit) or Error(Temporary) result and records no other effect. These are proposed examples, not observed code.

Fixture EmitIndex accepts Nat i, appends event Tick(i), returns unit, and terminates. Fixture Const(n) returns Int n without effects.

Fixture TryAt accepts Nat attemptIndex. Index zero returns Error(Temporary). Later indices return Ok(unit). Every call terminates without external effects.

Retry roles supply attemptIndex explicitly through T04 bindings. It starts at zero and increases by one only for each admitted retry.

Fixture names and inline fixture definitions denote local exact revision `f1`. Literal event names have the singleton type Event(name).

Fixture endpoint/store policies state finite relations in their row. A production instance instead references a complete accepted policy definition.

Canonical ExitTarget contains `instanceRef:Ref<PatternInstance>` and `activationPath:StaticSelector`, each once. The author owns both required fields.

StaticSelector identifies the permitted lexical/enclosing activation path. It contains no future runtime invocation ID and requires no invented owner answer.

At invocation, T04 resolves that selector to a FrameKey containing run, invocation, instance, and parent identities.

Every nonlocal return, break, continue, throw, and cancellation uses the selected active occurrence. A matching definition name alone is insufficient.

Fixture names such as frame1 denote static activation selectors until invocation. They do not pretend a runtime frame already exists.

T04's package relation graph supplies explicit argument/result mappings and imported names. It does not add undeclared pattern slots.

EntityLocation uses T04's exact entity reference plus declared slotPath. A nested Binding is data, not a separately referable T02 entity.

EXP-EVAL p2 corrected its name-binding slot to EntityLocation. P3 also stores its rounding Predicate inline, without inventing a Predicate entity.

Compatibility note: p1/p2 operator semantics remain unchanged. Conversions without declared-rounding retain their selected fields and behavior.

Name consumers require the corrected EntityLocation. Declared-rounding consumers require p3's inline Predicate. Each assessment retains its exact old/new scope.

The prior patterns-draft-3 manifest is `511d87d650ce20624da7ec1b796b79540e12cc8fd57b91839fc3375242e8124a`. Name-binding consumers require the corrected p2 location.

Slot notation TypedValue denotes a typed data value, not an implicit Expression<T> wrapper. Its concrete type follows the entry's static type/signature constraints.

FLOW-LOOP p3 has a static itemType binder. Its body signature explicitly determines output/failure types and therefore the collected result element type.

Unsupported or unbound generic type derivations remain findings. A future call's observed values cannot fill a missing static type binder.

### Reference aliases and runtime records

The following aliases constrain existing T02 kinds. They do not declare new canonical entity kinds.

| Catalog notation | Exact target kind and constraint |
| --- | --- |
| Ref<Operation> | Ref<PatternInstance> with the declared callable signature and continuation. |
| Ref<Resource> | Ref<PatternInstance> bound to STATE-RESOURCE. |
| Ref<CON-TASK>, Ref<CON-COORDINATE>, Ref<FAIL-CANCEL>, Ref<FAIL-CLEANUP> | Ref<PatternInstance> bound to the named exact definition revision. |
| Ref<RecordType>, Ref<VariantType>, Ref<ResultType> | Ref<Type> with the corresponding form. ResultType is the declared Ok/Error variant form. |
| Ref<Contract> | Ref<Package> for the exact contract revision. |
| Ref<Definition> | Exact Type, PatternDefinition, RelationDefinition, PatternInstance, Clause, or Package reference under the declared role. |
| Ref<Endpoint>, Ref<Store> | Ref<PatternInstance> defining the external callable or state/resource boundary, with explicit scope and guarantees. |
| Ref<AtomicityPolicy>, Ref<ConsistencyPolicy>, Ref<IdempotencyPolicy> | Ref<Package> defining that policy's domain, guarantees, failures, and limits. An opaque name is insufficient. |
| Ref<Scope> | Ref<Scope> under T02's Scope entity payload. |

OperationSignature and Predicate are inline typed data. FN-CALLBACK p2 and EXP-EVAL p3 do not reference them as invented entities.

RecordRef<T> identifies a retained runtime record by record kind, stable ID/revision, owning run, and Artifact. It is separate from canonical Ref<T>.

EffectLocation contains `result:RecordRef<ActionResult>` and `effectID:ID`, each once. It selects a declared known effect within that exact result.

EXT-COMPENSATION p2 uses EffectLocation. Its prior actor, trigger, compensation, failure, deadline and repeated-effect semantics remain unchanged.

### Runtime-data supplier eligibility

Each slot has supplierAllowed=false except the following exact evaluated-data slots. The table gives the complete true set.

| Definition | Runtime-data slots admitting a typed supplier |
| --- | --- |
| VAL-LITERAL p2 | value |
| VAL-RECORD p2 | fields, with the declared record shape/type constraints still fixed |
| VAL-COLLECTION p2 | items |
| VAL-VARIANT p2 | tag, payload, constrained by fixed alternatives |
| FLOW-MATCH p2 | value |
| FLOW-LOOP p3 | items, only in collection mode |
| FLOW-EXIT p2 | value, only for return |
| FN-CALLBACK p2 | arguments |
| FAIL-RESULT p2 | value, only in construct mode |
| FAIL-EXCEPTION p2 | error, only in raise mode |
| EXT-IO p2 | request |
| EXT-PERSIST p2 | key, value when write mode enables it |
| EXT-COMPENSATION p2 | effect, a typed EffectLocation |

Definition/type references, operation selection, retry counts, order, continuation, owner policy, deadlines, choice bounds and permission settings remain static.

A future source-backed bounded parameter policy needs an explicit definition revision. This table supplies no automatic policy deferral by type alone.

Package.relations is the sole supplier index. A binds-supplier relation targets DataSlot{instanceRef,slotName}; no new PatternInstance field is introduced.

Its source is a typed T04 declaration-location or runtime-port endpoint. Mapping, phase, availability, and failure policy are explicit and checked.

At authoring, the target static Binding key is absent. Exactly one valid supplier satisfies requiredness without claiming its future TypedValue exists.

At invocation, argument/producer values materialize and validate before consumption. Invalid input follows the declared rejection before dependent effects.

An unknown owner policy, conflicting literal/source, late/cyclic producer, or unavailable port remains a finding. Supplier declaration does not prove unsupported liveness.

Old static instances retain unchanged behavior under an explicit scoped compatibility assessment. Their historical source revision is not silently rewritten.

Every entry inherits structural/type/name checks, per-case evaluation, and revision impact. Its check column lists additional specific obligations.

Check shorthand: S=structure.inspect, T=slots.check, N=names.resolve, C=composition.check, P=profile.check, E=cases.evaluate, R=revision.impact.

All checks remain specified responsibilities with unavailable implementation and absent test evidence in this library draft.

Program continuation differs from harness convergence. An ongoing service can have precise behavior without promising global program termination.

An ongoing policy declares its stop trigger, active invariants, observation boundary, cancellation target, and final observable after acknowledgment.

Missing cancellation enforcement or required liveness evidence remains a gap. The controller's contract-authoring budget does not become a program deadline.

PATTERN-005 | Each selected instance must resolve applicable required slots through declared static bindings or admitted typed runtime suppliers.

PATTERN-006 | The checker must reject unknown slot names without inventing their meaning.

PATTERN-007 | The author must declare child roles, effects, exits, and context before composing an instance.

PATTERN-008 | The skill must omit unused instances without removing their library definitions.

## Inventory

Ten family catalogs keep complete counterparts readable. Their rows are canonical definitions, not a name-only coverage list.

| Family | Entries and catalog | Distinctions covered |
| --- | --- | --- |
| Values and types | [VAL-LITERAL, VAL-RECORD, VAL-COLLECTION, VAL-VARIANT](patterns/values.md) | Units, precision, ranges, records, sequences, sets, variants, optional/null values. |
| Expressions | [EXP-EVAL](patterns/expressions.md) | Names, operators, explicit order, short circuit, conversions. |
| Control flow | [FLOW-SEQUENCE, BRANCH, MATCH, LOOP, EXIT, RECURSION](patterns/control.md) | Order, exhaustive alternatives, loops, break/continue/return targets, bounded recursion. |
| Functions | [FN-OPERATION, CALLBACK](patterns/functions.md) | Arguments/results, pre/postconditions, effects, callbacks, higher-order calls. |
| Failure | [FAIL-RESULT, EXCEPTION, RETRY, DEADLINE, CANCEL, CLEANUP](patterns/failure.md) | Tagged errors, throws/handlers, retries, timeout, cancellation, cleanup. |
| Abstraction | [ABS-INTERFACE, GENERIC, INHERIT, COMPOSE](patterns/abstraction.md) | Abstract bindings, generics, dispatch, inheritance, composition. |
| State and resources | [STATE-MACHINE, RESOURCE](patterns/state.md) | Initial state, transitions, invariants, ownership, acquisition/release, lifetime. |
| Concurrent behavior | [CON-TASK, COORDINATE](patterns/concurrency.md) | Task lifecycle, async join, ordering, synchronization, shared-state policy. |
| External effects | [EXT-IO, PERSIST, TRANSACTION, MESSAGE, IDEMPOTENCY, COMPENSATION](patterns/effects.md) | I/O, state storage, atomic groups, delivery, duplicate effects, recovery obligations. |
| Organization | [ORG-MODULE](patterns/organization.md) | Imports, visibility, dependencies, initialization. |

## D05 proposal and discovery

Decision owner: repository owner. State: proposed, unresolved.

Proposed conceptual baseline: review these 34 counterparts, their discriminated variants, and all composition obligations against T04's scenarios.

The inventory is a finite starting coverage set. The owner's all-language and all-pattern direction remains broader than this baseline.

| Detected gap | Required record | Permitted next action |
| --- | --- | --- |
| Behavior fits no entry or variant | Source obligation, observable difference, candidate family | Design a new versioned counterpart and cases. |
| Entry exists but lacks required semantics | Exact entry/slot/relation, affected guarantee | Revise that definition with impact review. |
| Language feature differs from common meaning | Exact profile/source, semantic difference | Add a T05 mapping or declare unsupported. |
| Meaning exists but no check method covers it | Criterion, category, method limit | Keep capability gap and assign checking work. |
| Composed entries add an edge obligation | Exact relation and exposed effect/exit | Apply T04 or create a tracked semantic gap. |

Discovery compares a source program or requirement against control flow, effects, state, type, lifecycle, and boundary responsibilities.

It then checks each unmatched responsibility against all ten families. A name match or syntax resemblance does not establish coverage.

Potential additional families include reflection, dynamic loading, foreign interfaces, metaprogramming, and distributed consensus. They are explicit unmapped candidates here.

No candidate is impossible or supported merely because this list names it. Acceptance requires its own prepared meaning, cases, and checking responsibility.

PATTERN-009 | The reviewer must retain every unmatched responsibility as a gap with its source and behavioral consequence.

PATTERN-010 | The maintainer must invalidate affected instances and evidence after a definition revision changes.

## Review mapping

| T03 obligation | Evidence and cases |
| --- | --- |
| T03-001 | Definition interface plus every catalog row and check column. |
| T03-002 | Common slot rules plus each R/C declaration, optional annotation, cardinalities, and negative cases. |
| T03-003 | Each meaning/termination/failure column. |
| T03-004 | Common ChildRole interface plus each roles/context and check column. T04 owns edge semantics. |
| T03-005 | Every entry's positive and negative assignments. They are conceptual case expectations. |
| T03-006 | Discovery table, explicit additional-family gaps, PATTERN-004/009. |
| T03-007 | Per-entry inherited maturity tuple and separate method limitations. |
| T03-008 | PATTERN-008 and optional instance selection. |

Required ticket cases: FLOW-BRANCH lacks otherwise, FLOW-LOOP lacks termination, FAIL-RETRY lacks duplicatePolicy, ABS-INTERFACE lacks concrete bindings.

For FLOW-LOOP, the required missing-termination case also covers a missing ongoing continuation/stop policy. Neither mode receives an implicit bound.

Each corresponding negative case identifies that exact missing slot. Repair supplies only that slot and does not waive other checks.

T02 acceptance and D05 remain close gates. T04/T05/T06/T07 consume this draft provisionally. No reverse completion dependency is created.

Verification scope: the Rust syntax checker checks this index's numbered clauses. Family catalogs are structured definition data without independent normative clauses.

Catalog review checks entry count, required fields, paired assignments, typed references, and semantics manually. A `no-clauses` diagnostic on catalog data is not a semantic verdict.
