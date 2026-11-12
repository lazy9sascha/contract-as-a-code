# Contract model

Revision: model-draft-5. Status: provisional design. Designer: `the designer`. Reviewer: `the designer`.

This draft answers T02, specification r2, SHA-256 `8111cf7a150d94168d8866dcc7619fba97979d0eb91326f3694f3991888ffa76`.

It proposes meaning and conceptual checks. It supplies no executable validator, parser, proof engine, or accepted implementation target.

## Source boundary

The coordinator retains the run record, source captures, and specification snapshots.

The T01 r2 input identity is `929f5c0175240c4794c77aedb7723b02eaddcdc5940b675a46038d2a425bdbd1`.

The provisional [concept draft](../concept.md) was inspected at SHA-256 `988442c52e9049b5e7cc67b3c2910f823809438a3152f35d0db713407a646f51`.

T01 is an input and a close gate. Its product concept, D01, D02, and owner acceptance remain provisional inputs.

This draft assumes the contract governs required behavior above implementation harnesses. Tools supply bounded evidence. The controller applies the completion gate.

| Source | Use here | Authority limit |
| --- | --- | --- |
| B05–B07, B17, B20–B23 | Precise meaning, prepared structures, typed slots, nesting | Source statements do not select a storage encoding. |
| B24–B27 and supplied controller explanation | Contract above implementation and repeated repair | They do not establish implemented checking capabilities. |
| `CONTRACT.md`, captured draft | Clause identity, terms, behavior, source authority, review | Reconciliation through T01 remains open. |
| `examples/batch-division.md`, captured `draft-1` | Concrete values, cases, inheritance, error results | Its limits and choices apply only to that example. |

Source hashes and access conditions are in the retained T02 specification. This draft preserves the source text in place.

## Meaning and ownership

A contract defines admitted inputs and permitted observable behavior within its scope. A model-valid contract can still lack acceptance evidence.

A semantic package is one immutable contract revision with its canonical elements and exact dependency references.

An entity is an independently referable record. An entity revision identifies one immutable definition of that entity.

A slot is a named position declared by a pattern definition. A static Binding supplies its value or explicit unresolved state.

An explicitly admitted runtime supplier can instead supply evaluated data at invocation. It is a separate typed relation, not another Cell state.

A type defines admitted values and equality. It includes units and boundaries when those distinctions affect behavior.

A clause states one obligation, prohibition, or permission. Its controlled sentence and typed meaning belong to the same entity.

An observable is a result, error, state change, or external effect exposed by the contract.

A decision records missing or disputed meaning and the actor authorized to resolve it. A finding reports a defect or checking limit.

An acceptance predicate is a declared Boolean condition over named inputs. Check results describe whether a method established that condition.

The owner controls requirement changes and acceptance. The author maintains definitions and source links. A resolver supplies an authorized decision answer.

A checker records method results. A reviewer challenges those results and meaning. An observer records code behavior without creating requirement authority.

MODEL-001 | The author must define each term before its first normative use.

MODEL-002 | The author must assign one meaning to each entity within its declared scope.

MODEL-003 | The observer must label code observations separately from approved requirements.

MODEL-004 | The author must preserve unresolved meaning as a decision or finding.

## Field conventions

These tables define conceptual fields, not JSON, Markdown syntax, a database schema, or a wire protocol.

`1` means exactly one. `0..1` means optional. `0..n` means a finite collection. `1..n` means a nonempty finite collection.

Lists preserve order. Sets reject duplicate members. `Ref<T>` identifies an entity and its exact revision in a named contract revision.

`Text` is a finite string. `Name` is nonempty text unique in its declared namespace. `ID` is an opaque stable identifier.

`Artifact` identifies retained bytes by location and SHA-256 digest. An unavailable artifact is unresolved, not an empty file.

`Predicate` identifies typed inputs, their bindings, and a defined Boolean relation. An undefined relation creates unresolved meaning.

`Expression<T>` denotes a value of type `T` under named semantics. An unsupported evaluation method creates a capability finding.

`TypedValue` contains `type:Ref<Type>` and `value` admitted by that type, each once. Its author owns both required fields.

`Map<K,V>` is a finite mapping with unique keys of type K and values of type V. `SlotName` is Name within a pattern.

Every field below uses the value states in the next section. A missing required field produces a finding.

An optional field can remain absent only under its declared absence rule. Absence never supplies a default implicitly.

For payload tables, the named owner maintains every field in that row. Parent records own their contained field definitions and bindings.

### Common entity fields

| Field | Type and cardinality | Owner | Meaning and absence rule |
| --- | --- | --- | --- |
| `id` | `ID`, 1 | Author | Stable logical identity. Missing is invalid. |
| `revision` | `ID`, 1 | Author | Immutable content identity within this entity. Missing is invalid. |
| `kind` | Declared entity tag, 1 | Author | Selects exactly one payload table row. Missing or unknown tags are invalid. |
| `source` | Set of source references, 1..n | Author | Origin and authority classification. Missing prevents authority review. |
| `lineage` | List of lineage relations, 0..n | Author | Origin after replacement, split, merge, or derivation. Empty means no such origin. |

A source reference contains `artifact:Artifact`, `locator:Text`, and `authority:requirement|observation|proposal`, each exactly once.

The author owns these fields. A missing locator cannot identify the source passage. An artifact digest alone does not prove approval.

A lineage relation contains `kind:revises|splits|merges|replaces|derives` and `target:Ref<Entity>`, each exactly once.

The author owns these fields. A missing target makes the relation invalid. Ordinary edits use `revises` without creating another logical ID.

### Package fields

| Field | Type and cardinality | Owner | Meaning and absence rule |
| --- | --- | --- | --- |
| `contract` | `ID`, 1 | Author | Stable contract identity. Missing is invalid. |
| `revision` | `ID`, 1 | Author | Immutable package identity. Missing is invalid. |
| `parent` | `Ref<Package>`, 0..n | Author | Earlier package revisions. Empty means initial revision. |
| `scope` | Scope entity, 1 | Owner | Included behavior, observables, and exclusions. Missing blocks interpretation. |
| `actors` | Set of Actor entities, 1..n | Author | Responsibilities and decision authority. Missing blocks actor resolution. |
| `terms` | Set of Term entities, 0..n | Author | Domain meanings. Empty permits only declared primitive vocabulary. |
| `types` | Set of Type entities, 1..n | Author | Types used by admitted inputs and observables. Missing blocks type checking. |
| `clauses` | Set of Clause entities, 1..n | Author | Obligations, prohibitions, and permissions. Missing means no contract behavior. |
| `patterns` | Set of PatternInstance entities, 0..n | Author | Selected prepared structures. Empty is valid if no behavior needs one. |
| `relations` | Set of Relation entities, 0..n | Author | Containment, bindings, calls, inheritance, and other dependencies. Empty means none. |
| `decisions` | Set of Decision entities, 0..n | Resolver | Unresolved or resolved meaning. Empty states that none are recorded. |
| `cases` | Set of Case entities, 1..n | Author | Expected inputs and observables. Missing blocks review. |
| `capabilities` | Set of Capability references, 0..n | Checker | Exact capabilities required for declared checks. Empty establishes no tool coverage. |
| `criteria` | Set of Criterion entities, 1..n | Owner | Declared acceptance conditions and evidence requirements. Missing blocks completion. |

Evidence, findings, and acceptance records are append-only assessment records outside the canonical package. They target exact package revisions.

This separation avoids changing the assessed revision when a checker attaches evidence. A requirement change creates a new package revision.

### Entity payloads

Every field listed in this table is required unless its cardinality states otherwise.

| Entity and owner | Fields: type and cardinality | Meaning and absence rule |
| --- | --- | --- |
| Scope, owner | `included:Text[1]`, `excluded:Text[1]`, `inputs:Set<Ref<Type>>[1..n]`, `observables:Set<Ref<Type>>[1..n]`, `assumptions:Set<Predicate>[0..n]` | Defines the behavior boundary. Empty assumptions means none. Exclusions explicitly describe their boundary, including an explicit none. |
| Actor, author | `name:Name[1]`, `responsibilities:Set<Text>[1..n]`, `authority:Set<Text>[0..n]` | Identifies responsibility separately from decision power. Empty authority grants none. |
| Term, author | `name:Name[1]`, `meaning:Text[1]`, `namespace:ID[1]` | One defined meaning in one scope. Missing meaning blocks normative use. |
| Type, author | `name:Name[1]`, `form:TypeForm[1]`, `equality:Predicate[1]` | Defines admitted values and equality. No implicit coercion or equality rule. |
| Clause, author | `modality:obligation\|prohibition\|permission[1]`, `actor:Ref<Actor>[1]`, `sentence:Text[1]`, `guard:Predicate[1]`, `behavior:Expression<ObservableSet>[1]`, `terms:Set<Ref<Term>>[0..n]`, `pattern:Ref<PatternInstance>[0..1]` | Guard selects applicability. Behavior defines required, forbidden, or permitted observables. Absent pattern means direct clause. Empty terms uses primitive vocabulary only. |
| PatternInstance, author | `definition:Ref<PatternDefinition>[1]`, `bindings:Map<SlotName,Binding>[1]` | Selects a prepared definition revision. Required slots need a valid static binding or exactly one explicitly admitted runtime supplier. Undeclared names are invalid. |
| Relation, author | `relationType:Ref<RelationDefinition>[1]`, `from:Ref<Entity>[1]`, `to:Ref<Entity>[1]`, `bindings:Map<Name,Binding>[1]` | Declares typed edges and their parameters. Empty bindings is valid only if the relation definition requires none. |
| Decision, resolver | `question:Text[1]`, `owner:Ref<Actor>[1]`, `targets:Set<Ref<Entity>>[1..n]`, `status:open\|resolved\|withdrawn[1]`, `answer:Cell[0..1]`, `answerSource:source reference[0..1]`, `disposition:Text[0..1]`, `effects:Set<Ref<Entity>>[1..n]` | Open requires Unknown answer. Resolved requires Known answer and authorized answerSource. Withdrawn requires disposition and absent answer. Other optional fields remain absent. |
| Case, author | `targets:Set<Ref<Entity>>[1..n]`, `input:TypedValue[1]`, `preconditions:Set<Predicate>[0..n]`, `expected:Predicate[1]`, `unresolved:Set<Ref<Decision>>[0..n]` | Defines expected behavior. Empty preconditions means unconditional. An unresolved expected result prevents a determinate case verdict. |
| Criterion, owner | `predicate:Predicate[1]`, `targets:Set<Ref<Entity>>[1..n]`, `requiredEvidence:Set<EvidenceRequirement>[1..n]` | States one acceptance condition and required evidence kinds. Missing evidence requirements cannot mean automatic success. |

`ObservableSet` is the type of permitted observable traces under the selected behavioral definitions. A trace includes ordering when ordering affects behavior.

An obligation requires its behavior on admitted traces. A prohibition excludes its behavior. A permission allows only its declared bounded alternatives.

These modalities do not resolve incompatible clauses automatically. Contradictory admitted traces create a conflict finding.

`EvidenceRequirement` contains `kind:syntax|semantic-review|case-evaluation|implementation-test|owner-answer`, `method:Name`, and `coverage:Set<Ref<Entity>>`, each once.

The criterion owner defines those fields. Empty coverage or missing method is invalid. One evidence kind cannot substitute for another.

`PatternDefinition` and `RelationDefinition` are versioned library entities owned by T03 and T04. This model defines their referencing boundary only.

Their required interface includes named slots, admitted contexts, semantic obligations, and checks. An unavailable definition prevents instance interpretation.

### Type forms and slot declarations

| Type form | Required parameters | Meaning |
| --- | --- | --- |
| Unit | None | Exactly one value, written `unit`. It represents a declared payload with no information. It differs from null and empty collections. |
| Null | None | Exactly one value, written `null`. It differs from Unit, Absent, Unknown, false, and zero. |
| Scalar | `primitive:Boolean\|integer\|decimal\|text`, `domain:Predicate`, `unit:Name`, `precision:Name` | Boolean, integer, decimal, or text values. Dimensionless and exact are explicit parameter values. |
| Record | `fields:List<SlotDeclaration>`, `extras:permit\|reject` | Named fields with their own types and requiredness. The policy permits or rejects extra fields. |
| Sequence | `element:Ref<Type>`, `minimum:nonnegative integer`, `maximum:nonnegative integer\|unbounded`, `order:Text` | A finite ordered collection. Zero minimum explicitly permits empty. |
| Set | `element:Ref<Type>`, `minimum:nonnegative integer`, `maximum:nonnegative integer\|unbounded` | A finite collection under the element equality rule. Duplicate values are invalid. |
| Variant | `alternatives:Map<Name,Ref<Type>>` | Exactly one declared alternative from a nonempty map. A no-payload tag declares a unit payload type. |
| Reference | `targetKind:entity tag`, `revisions:Predicate` | A resolvable exact target after binding. Floating latest references are not valid bound values. |
| Choice | `valueType:Ref<Type>`, `domain:Predicate`, `chooser:Ref<Actor>`, `constraints:Set<Predicate>` | Explicit bounded permission with a nonempty admitted domain. |

All parameters have cardinality 1 and belong to the type author. Missing parameters create type-definition findings.

Unbounded mathematical domains use an explicit bound value `unbounded`. Tool execution limits remain separate from semantic bounds.

A field or slot declaration contains these fields. The pattern author owns every field.

`SlotDeclaration` names this declaration. Record types use the same declaration shape for their fields.

| Field | Type and cardinality | Meaning and absence rule |
| --- | --- | --- |
| `name` | `Name`, 1 | Unique within the declaration. Missing is invalid. |
| `type` | `Ref<Type>`, 1 | Admitted known values. Missing prevents validation. |
| `requiredWhen` | `Predicate`, 1 | True requires a value. False permits declared absence. Unknown blocks this determination. |
| `absence` | `Text`, 1 | Meaning when the slot is not required and absent. Missing does not mean no effect. |
| `emptyAllowed` | Boolean, 1 | Whether an empty value of the type is admitted. False differs from an absent flag. |
| `supplierAllowed` | Boolean, 1 | Whether a declared typed runtime supplier can satisfy this evaluated-data slot. False permits only static Binding satisfaction. |
| `default` | `TypedValue`, 0..1 | Explicit proposed value for absent bindings. Absent means no default. |
| `constraints` | Set of predicates, 0..n | Additional restrictions. Empty adds none. |

An always-required slot uses the constant true predicate. An optional slot uses false. Conditional slots bind their predicate inputs explicitly.

A Binding contains `cell:Cell[1]`, `origin:explicit|default[1]`, and `definition:Ref<PatternDefinition|Type>[0..1]`. The binding author owns these fields.

Default origin requires its exact definition revision. Explicit origin permits absent definition. Missing cell or origin invalidates the binding.

Pattern defaults use their PatternDefinition revision. Record defaults use their Type revision. The enclosing context selects the admitted definition kind.

Choice<T> is a distinct slot type. It does not implicitly bind a T slot. A T binding supplies one admitted concrete value.

A Choice<T> binding supplies explicit discretion over T values. Its owner declares that slot type before implementation can exercise the permission.

### Static values and runtime suppliers

Binding.cell remains Known, Unknown, Absent, or NotApplicable. A Known value has its declared type T, not Expression<T> masquerading as T.

A runtime supplier is an exact Relation reference using T04's declared binds semantics. It identifies the source endpoint and consuming instance/slot.

Package.relations is its only index. The relationType is binds-supplier. Source endpoints are declaration-location or runtime-port; the target is a DataSlot descriptor.

RuntimePort and DataSlot are contained typed descriptors, not canonical entity kinds. FormalParameter declarations remain within their containing ContextBinding record.

Its known declaration establishes source type, target type T, mapping Expression<T>, invocation phase, availability, and rejection behavior before consumption.

The source selects a FormalParameter declaration-location or a typed argument/result runtime port. Static activation selectors identify the eventual current frame.

For an applicable required slot, authoring satisfaction requires exactly one valid static Binding or one valid allowed supplier. Both together create a conflict.

Unknown owner decisions, invalid literals, or arbitrary absent values cannot be overridden by adding a supplier. Missing both creates a missing-required finding.

Definitions mark evaluated-data slots individually. Type references, schemas, owner policies, mode choices and permission bounds remain authoring-resolved by default.

A source-backed bounded contract can explicitly parameterize another slot only after its admitted values and guarantees are declared and reviewed.

At invocation, T04 materializes the actual TypedValue<T> after source/mapping/type checks. Invalid or unavailable arguments follow declared rejection before dependent effects.

A same-activation cyclic or late producer cannot satisfy availability. Cross-iteration state transitions need their separately declared ordering and progress semantics.

Authoring completion checks the complete supplier declaration and required evidence. It does not claim a future argument value already exists.

MODEL-035 | The checker must require exactly one valid source for every applicable required slot.

MODEL-036 | The checker must reject runtime suppliers for slots whose definitions forbid them.

MODEL-037 | The checker must reject conflicting, mistyped, unavailable, late, or cyclic same-activation suppliers.

MODEL-038 | The controller must not treat future runtime arguments as unanswered owner decisions or known literal values.

MODEL-005 | The checker must reject bindings whose known values violate their declared types.

MODEL-006 | The checker must report each undeclared slot name as a structure finding.

MODEL-007 | The author must materialize each applied default as a binding with its definition revision.

MODEL-008 | The checker must preserve false, zero, empty, null, and absence as distinct states.

MODEL-009 | The checker must report unresolved requiredness without guessing its Boolean result.

MODEL-010 | The author must bind each behavior-affecting term to a definition revision.

## Value states and errors

`Cell` is the following tagged conceptual state. A slot's value state does not redefine its type.

| State | Fields | Meaning | Completion consequence |
| --- | --- | --- | --- |
| `Known` | `value:TypedValue[1]` | A supplied value, including admitted false, zero, empty, or null. | Check the type and constraints. |
| `Unknown` | `decision:Ref<Decision>[1]` | A required answer is not known. | Blocks any criterion that needs that answer. |
| `Absent` | None | No static Binding exists. This is the semantic view of omission. | Required without a valid allowed supplier means finding. Optional uses declared absence meaning. |
| `NotApplicable` | `reason:Text[1]`, `proof:Predicate[1]` | Requiredness is false under explicit inputs. | Valid only after the predicate establishes false requiredness. |

The binding author owns these fields. Missing required fields invalidate the state. Empty reasons do not justify NotApplicable.

Null is a known value only under the Null type or a declared variant arm containing Null. Null never means Absent or Unknown.

A variant value identifies its tag and payload. A Null arm contains null. A no-information arm contains Unit's value `unit`.

An empty list, set, map, or string is a known value. It needs type permission and `emptyAllowed=true`.

A choice is Known with a Choice type. It identifies permitted alternatives and a chooser. Unknown grants no choice.

An invalid representation or value produces a finding. A tool execution error produces assessment evidence of failure to check.

Neither error is a valid slot value unless the domain type explicitly defines an error result as a known variant.

`Error(ZeroDivisor)` in the batch example is such a domain result. A checker crash is not that result.

MODEL-011 | The checker must not replace Unknown, Absent, or invalid values with zero or empty values.

MODEL-012 | The author must express implementation discretion as a bounded Choice value or permission clause.

MODEL-013 | The checker must reject NotApplicable when requiredness is true or unresolved.

## D03: Canonical representation proposal

Decision owner: repository owner. State: proposed, unresolved. Drafting and review are authorized. Acceptance is not recorded.

| Candidate | Preserved strength | Failure exposed by cases | Disposition |
| --- | --- | --- | --- |
| Controlled prose only | Human intent, actor, modality, readable conditions | Missing slot structure and ambiguous conversions remain difficult to locate mechanically. | Insufficient alone for typed prepared patterns. |
| Structured clauses only | Types, stable references, slot checks | A schema can admit null while the intended clause rejects null. Structural validity does not settle intent. | Insufficient alone for human meaning review. |
| One combined semantic package | Paired controlled sentences and typed semantics share identities and revisions | Disagreement remains explicit. Review can inspect the exact conflict. | Proposed choice. |

The combined package has one canonical location per entity. It contains the controlled clause and the structured behavioral meaning together.

Neither half overrides the other. Their disagreement invalidates agreement for that revision. The resolver supplies missing meaning before acceptance.

Generated prose, diagrams, summaries, schemas, code, and tool indexes are derived views unless explicitly included in the canonical package.

A conversion records `source:Ref<Package>[1]`, `converter:Name[1]`, `version:ID[1]`, `target:Artifact[1]`, `mapping:Map<Name,Ref<Clause>>[1]`, `preserved:Set<Ref<Entity>>[0..n]`, `losses:List<Text>[0..n]`, and `limits:Text[1]`.

The converter owns these fields. Missing required fields invalidates the record. Empty mapping prevents a claim of clause coverage.

An empty loss list states only that this conversion reports no loss. It does not establish semantic equivalence without the declared review.

Mechanical comparison can check slot values, tags, references, ranges, and supported predicate forms. It cannot establish arbitrary prose equivalence.

An unsupported comparison returns indeterminate with the affected meaning. An owner or reviewer cannot turn that limit into mechanical proof.

MODEL-014 | The converter must report every detected condition loss as a mismatch.

MODEL-015 | The controller must reject conflicting canonical text and structured meaning at the completion gate.

MODEL-016 | Derived artifacts must identify their source package revision and conversion limits.

MODEL-017 | Derived artifacts must not change canonical obligations or supply missing owner decisions.

MODEL-018 | The reviewer must record semantic comparison limits separately from structural check results.

## D04: Identity and lineage proposal

Decision owner: repository owner. State: proposed, unresolved. Logical identity tracks one responsibility, not a title or physical line.

| Change | Logical ID | Required relation | Effect on earlier evidence |
| --- | --- | --- | --- |
| Spelling or formatting only | Preserve | New entity revision revises prior revision | Retain historical record. Revalidate equivalence before carrying evidence. |
| Bound, condition, actor, guarantee, or permitted effect changes | Preserve for the same responsibility | New entity revision revises prior revision | Invalidate affected semantic and case evidence. |
| One clause splits into two independent clauses | Retire original. Allocate two new IDs. | Each new clause splits the exact original revision. | Old coverage does not establish either new clause. |
| Several clauses merge | Retire originals. Allocate one new ID. | New clause merges every original revision. | Review each original obligation's disposition. |
| Another responsibility replaces a clause | Retire original. Allocate new ID. | New clause replaces the exact original revision. | Require fresh requirement and case review. |
| A clause retires without replacement | Retain retired record and reason | No live replacement relation | Record authorized scope removal or unresolved coverage. |

A retirement record contains `target:Ref<Entity>`, `reason:Text`, and `authority:source reference`, each once. The owner authorizes retirement.

The author records it. Missing authority leaves the removal unresolved. A retired ID remains reserved across the contract's lineage.

A package revision manifest maps every live ID to one entity revision. It also retains retirement and dependency identities.

The manifest's owner is the author. Missing entities or dependency revisions make that package revision incomplete.

MODEL-019 | The author must preserve logical IDs through edits to the same responsibility.

MODEL-020 | The author must allocate new IDs for split, merged, or replacement responsibilities.

MODEL-021 | The author must retain every retired ID and its authorized disposition.

MODEL-022 | The author must not reuse a retired ID for another entity.

MODEL-023 | The author must record exact predecessor revisions for every lineage relation.

MODEL-024 | The author must create a new package revision after any canonical content or dependency change.

## Assessments, links, and freshness

Assessment records use the common entity fields. They append results without changing the assessed package.

| Entity and owner | Fields: type and cardinality | Meaning and absence rule |
| --- | --- | --- |
| Capability, checker | `provider:Name[1]`, `version:ID[1]`, `method:Name[1]`, `supported:Set<Ref<Entity>>[1..n]`, `assumptions:Set<Predicate>[0..n]`, `limits:Text[1]`, `state:specified\|implemented\|tested\|unavailable[1]` | A bounded checking capability. Empty assumptions adds none. Missing limits or version prevents evidence use. |
| Evidence, checker | `package:Ref<Package>[1]`, `targets:Set<Ref<Entity>>[1..n]`, `dependencies:Set<Ref<Entity>>[0..n]`, `cases:Set<Ref<Case>>[0..n]`, `capability:Ref<Capability>[1]`, `kind:EvidenceKind[1]`, `result:CheckResult[1]`, `artifact:Artifact[1]`, `coverage:Text[1]`, `limits:Text[1]` | Exact check inputs, results, and retained output. Empty cases means no case evaluation. Empty dependencies asserts none, subject to impact review. |
| Finding, checker or reviewer | `package:Ref<Package>[1]`, `targets:Set<Ref<Entity>>[1..n]`, `slotPath:Text[0..1]`, `kind:structure\|missing\|conflict\|authority\|capability\|stale[1]`, `detail:Text[1]`, `blocking:Boolean[1]` | Locates a defect or checking gap. Absent slotPath targets the entity. Missing blocking status is unresolved. |
| Acceptance, owner or delegated reviewer | `package:Ref<Package>[1]`, `criteria:Set<Ref<Criterion>>[1..n]`, `evidence:Set<Ref<Evidence>>[1..n]`, `actor:Ref<Actor>[1]`, `result:accepted\|rejected[1]`, `authority:source reference[1]` | Records acceptance under declared authority. Missing authority cannot establish acceptance. |

`EvidenceKind` uses the five kinds declared in EvidenceRequirement. `CheckResult` is passed, failed, unsupported, indeterminate, or execution-error.

Passed means the named method established its declared claim under its assumptions. It does not establish an unrestricted contract-completion claim.

Freshness is a relation between evidence and a target revision. It is not another CheckResult. A passed record can be stale.

Any changed target revision makes evidence inapplicable to that new target until impact review records a fresh result.

Changes to predicates, bounds, terms, types, effects, bindings, inheritance, language capabilities, assumptions, and decisions invalidate affected dependent evidence.

Impact follows declared dependency links transitively. A missing dependency map creates an indeterminate impact finding and blocks evidence reuse.

A changed inherited precondition affects descendants, their bindings, cases, and evidence. An unchanged leaf file does not preserve semantic freshness.

Decision answers remain historical. An edited question, target, assumption, or authority invalidates the answer's applicability until its resolver checks it.

Editorial equivalence is a recorded review result for exact revisions. It permits a new evidence record that references the earlier evidence.

It never rewrites the old result or silently reassigns its target. Unaffected records retain their historical validity and applicability.

MODEL-025 | The checker must bind every assessment to exact package and target revisions.

MODEL-026 | The checker must record its capability revision, method, coverage, assumptions, and limits.

MODEL-027 | The controller must reject stale evidence for the current target revision.

MODEL-028 | The reviewer must invalidate dependent evidence after a behavior-affecting change.

MODEL-029 | The resolver must recheck decision applicability after its question, targets, authority, or assumptions change.

MODEL-030 | The reviewer must preserve unaffected evidence instead of invalidating unrelated records.

MODEL-031 | The controller must not treat unsupported, indeterminate, or execution-error results as passed evidence.

## Acceptance predicate boundary

A criterion evaluates over named package elements and assessment references. True, false, unknown, and checker error remain distinct outcomes during checking.

The predicate itself has Boolean meaning. Unknown means inputs or meaning remain unresolved. Checker error means no valid evaluation result exists.

An explicit human review can satisfy a declared semantic-review requirement. It cannot satisfy an implementation-test requirement by changing its label.

Sufficient evidence depends on declared criteria and supported methods. The model cannot prove all behavior or all prose equivalence automatically.

T11 owns the controller state machine, repair cycle, and final gate. This model supplies its revision, criterion, decision, and evidence references.

MODEL-032 | The controller must require evidence for every declared acceptance criterion.

MODEL-033 | The controller must keep syntax, semantic review, case evaluation, and implementation tests separate.

MODEL-034 | The controller must preserve unresolved criteria instead of declaring completion.

## Worked instances and challenge cases

These are conceptual model fixtures. Their expected results do not assert executed semantic checks or approved product behavior.

All fixture slot declarations explicitly have supplierAllowed=false unless their case enables it. MC19–MC21 enable only items. This is no product default.

### Filled retry fragment

Fixture `F-RETRY@r1` defines one illustrative prepared retry pattern. T03 will own any accepted library counterpart.

Its operation executes once before retries. `retryCount` counts additional attempts. Zero means no additional attempt.

The type is an exact dimensionless integer from zero through three. Equality compares mathematical integer values.

| Slot | Declaration | Filled binding |
| --- | --- | --- |
| operation | Required operation reference | Known `Send@r1`, returning Success or TemporaryFailure |
| retryCount | Required bounded integer | Known `0` |
| eligibleFailure | Required result predicate | Known `result=TemporaryFailure` |
| delay | Required nonnegative duration | Known `0 milliseconds` |
| deadline | Required positive duration | Known `1000 milliseconds`, measured from initial attempt start |
| duplicatePolicy | Required declared behavior | Known `reuse request identity for every attempt` |
| exhaustion | Required observable | Known `return the last TemporaryFailure` |

The fixture admits no concurrent calls. Send terminates within the remaining deadline. Deadline expiry returns Timeout without further attempts.

Each slot has true requiredness, no default, `emptyAllowed=false`, and absence meaning `missing required binding`.

Each retry-fragment slot has supplierAllowed=false. Its count/policy fields cannot be deferred to an unreviewed runtime source.

No term is unresolved. The one initial attempt, zero retries, and final failure are determinate for a TemporaryFailure input.

Result: this fragment is filled under its fixture definition. Whole-contract acceptance still requires scope, authority, composition, cases, and evidence.

### Required and adversarial cases

| Case | Input and target | Expected model result | Supporting rules |
| --- | --- | --- | --- |
| MC01 | `F-RETRY@r1.retryCount=Absent` versus Known `0` | Absent creates a missing-required finding. Zero permits one initial attempt and no retry. | MODEL-005, MODEL-008, MODEL-011 |
| MC02 | Clause `C-A@r1` splits into `C-B@r1` and `C-C@r1` | Retire C-A. Both new IDs link to C-A@r1. Review original coverage against both children. | MODEL-020 through MODEL-023 |
| MC03 | Canonical sentence rejects null. Paired structure admits null. | Conflict blocks agreement and completion. No precedence rule selects the easier half. | MODEL-014, MODEL-015 |
| MC04 | Base precondition changes from `x>=0` to `x>0` in r2 | Descendant evidence for r1 remains historical. Evidence for r2 needs new impact and substitution review. | MODEL-024 through MODEL-029 |
| MC05 | A deadline uses undefined `quickly` | Unresolved term finding targets the clause and deadline binding. No invented duration. | MODEL-001, MODEL-004, MODEL-010 |
| MC06 | Retry count Unknown points to open D-COUNT | Decision blocks affected criterion. It grants no permission to select three. | MODEL-011, MODEL-012, MODEL-034 |
| MC07 | `F-CHOICE@r1.retryCount` explicitly declares Choice<Integer[0..3]>, chooser Implementer. Known Choice admits zero through three. | Bounded discretion exists in this separate fixture. Every permitted value remains subject to its declared deadline and duplicate rules. | MODEL-005, MODEL-012 |
| MC08 | Optional empty label, absent label, false flag, zero counter | Four distinct states. Empty needs permission. Absent uses declared absence. False and zero retain their values. | MODEL-005, MODEL-008 |
| MC09 | Unknown requiredness with NotApplicable binding | Reject applicability claim. Resolve its predicate inputs before assessing absence. | MODEL-009, MODEL-013 |
| MC10 | Tool timeout while checking a known domain error result | Execution-error evidence. Domain Error remains an ordinary typed result. No completion evidence follows. | MODEL-026, MODEL-031 |
| MC11 | Generated schema drops a deadline condition | Mismatch identifies source clause, target field, and lost condition. Derived schema has no replacement authority. | MODEL-014, MODEL-016, MODEL-017 |
| MC12 | Code accepts null but approved clause rejects it | Observation records discrepancy. It does not revise the requirement. | MODEL-003 |
| MC13 | Arbitrary prose equivalence exceeds the checker's method | Indeterminate with exact coverage and limit. Declared semantic review remains necessary. | MODEL-018, MODEL-026, MODEL-031 |
| MC14 | Pure heading edit changes package bytes | New revision. Historical evidence stays unchanged. Recorded equivalence supports a fresh scoped assessment. | MODEL-024, MODEL-025, MODEL-030 |
| MC15 | Split clause lacks a disposition for one old guarantee | Lineage exists but requirement coverage remains unresolved. No acceptance follows from new IDs alone. | MODEL-004, MODEL-020, MODEL-032 |
| MC16 | Known null under integer retryCount type | Type finding. Null cannot become absence, zero, or Unknown. | MODEL-005, MODEL-008, MODEL-011 |
| MC17 | Required `ack` slot declares Null, with Known null, no default, and false emptyAllowed | Valid known value. Null is the declared singleton value, not an empty value or omission. | MODEL-005, MODEL-008 |
| MC18 | Known Choice<Integer[0..3]> binds `F-RETRY@r1.retryCount`, whose type is Integer[0..3] | Type finding. No implicit lifting grants discretion to a concrete-value slot. | MODEL-005, MODEL-012 |
| MC19 | Required items:List<Int> allows suppliers. One typed relation maps a declared invocation argument List<Int> before loop entry. No static value is supplied. | Source-bound authoring slot is valid after declaration/type/phase checks. Actual list exists only at invocation. | MODEL-035 through MODEL-038 |
| MC20 | MC19 additionally supplies static Known [] or a second supplier. | Conflict identifies both sources. The checker does not choose precedence. | MODEL-035, MODEL-037 |
| MC21 | MC19 uses a producer available only after loop entry, or a same-activation cyclic producer. | Availability/phase finding blocks consumption and authoring evidence. | MODEL-037 |
| MC22 | Retry policy count is Unknown with supplierAllowed=false, then a runtime source is attached. | Missing owner meaning and forbidden-supplier findings remain. No owner-policy evasion. | MODEL-036, MODEL-038 |
| MC23 | A literal List<Int> slot receives Known Expression<List<Int>> instead of a declared supplier. | Type finding. Expression is not implicitly a List. | MODEL-005, MODEL-038 |

All results above are designer-derived expectations. Independent review records the observed result for this exact artifact revision.

## T02 coverage and handoff

| T02 obligation | Model sections and rules | Challenge cases |
| --- | --- | --- |
| T02-001 | Meaning, field conventions, entity payloads, MODEL-001 through MODEL-004 | MC03, MC05, MC08, MC17 |
| T02-002 | Type forms, slot declarations, value states, MODEL-005 through MODEL-013, MODEL-035 through MODEL-038 | MC01, MC06 through MC09, MC16 through MC23 |
| T02-003 | D04, MODEL-019 through MODEL-024 | MC02, MC14, MC15 |
| T02-004 | D03 comparison and combined package, MODEL-014 through MODEL-018 | MC03, MC11, MC13 |
| T02-005 | Derived-view authority, MODEL-015 through MODEL-018 | MC03, MC11 |
| T02-006 | Unknown versus Choice and runtime arguments, MODEL-011, MODEL-012, MODEL-038 | MC06, MC07, MC18, MC22 |
| T02-007 | Common references and assessments, MODEL-025, MODEL-026 | MC04, MC10, MC14 |
| T02-008 | Freshness and applicability, MODEL-024 through MODEL-030 | MC04, MC14 |
| T02-009 | Terms and binding rules, MODEL-001, MODEL-010 | MC05 |
| T02-010 | Source authority, MODEL-003, MODEL-017 | MC12 |

T03 consumes pattern-instance slots, types, and definition references. T04 consumes relation identity and dependency references.

T06 consumes assessment targets, check results, capability limits, and freshness. T11 consumes criteria, unresolved states, and evidence boundaries.

The downstream tasks define their own semantics. This model does not supply a pattern inventory, composition calculus, language profile, or tool transport.

| Open decision or dependency | Owner | Behavioral effect and required next evidence |
| --- | --- | --- |
| T01 D01 and accepted product boundary | Repository owner | Confirm scope and controller authority. Recheck affected model assumptions. |
| T01 D02 and document authority | Repository owner | Reconcile the method draft and proposed canonical document responsibilities. |
| D03 combined semantic package | Repository owner | Select canonical representation after reviewing MC03, MC11, and MC13. |
| D04 identity and lineage policy | Repository owner | Accept split, merge, retirement, and evidence-carry rules after MC02, MC14, and MC15. |
| T03/T04 definition interfaces | Assigned downstream designers | Reconcile prepared definition fields and relation semantics without silently changing these references. |
| T06/T11 checking and completion methods | Assigned downstream designers | Specify exact evidence production, determinism, and controller gate behavior. |

Only T01 is T02's upstream close gate. Downstream interface reconciliation does not create a reverse close-gate cycle.

The owner decisions and independent review remain open. This artifact is provisional and cannot complete T02 or conceptual 0.1.0.

## See also

- [concept.md](../concept.md): product concept.
- [composition-semantics.md](composition-semantics.md): composition rules.
- [CONTRACT.md](../../CONTRACT.md): method rules.
