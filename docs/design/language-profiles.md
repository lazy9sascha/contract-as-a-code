# Language profiles and coverage

Revision: `profiles-draft-2`. Status: provisional design. Designer: `the designer`. Reviewer: `the designer`.

This draft answers T05 r2, SHA-256 `0f83104003badf9d62b91750a6eb011f5f4e024779fb4a5fc57dde6a3183cc88`.

## Scope and authority

A profile maps exact language semantics to prepared contract definitions. Similar syntax or familiar feature names do not establish compatibility.

Provisional inputs are the reviewed T02 model, T06 protocol, T03 library, and T04 composition design. Their acceptance remains open.

The source manifest and exact submitted T03/T04 identities are retained with this ticket's evidence bundle. Changed inputs require impact review.

The inspected T03 bundle is patterns-draft-3, manifest SHA-256 `511d87d650ce20624da7ec1b796b79540e12cc8fd57b91839fc3375242e8124a`.

The inspected T04 composition-draft-4 has SHA-256 `c747405b0d6a86ce1bcb1593308899c7102bf74d5cfd7fa69f0754f57d958909`.

The owner's broad language direction remains in B23. D08 selects a finite initial baseline without declaring other languages impossible or supported.

Specific language claims require primary-source release identities and retained collection evidence. Missing versions or source evidence remain explicit gaps.

The collected source pack, SHA-256 `d7fbba3f03a4c45ed5529c6ac485161d865e5866ac2ec643a234c640b9541010`, supplies R1–R7 and E1 locators.

It retains source summaries and links, not complete primary page bytes. Its retention limit does not establish independently archived page content.

## Profile schema

Every Profile and FeatureMapping uses T02 common identity, revision, source, and lineage fields.

| Profile field | Type and cardinality | Owner | Meaning and missing rule |
| --- | --- | --- | --- |
| language | Name, 1 | Profile author | Language identity. Missing prevents selection. |
| languageVersion | NonemptyText, 1 | Profile author | Exact specification release. An edition alone does not identify all semantics. |
| edition | Name, 0..1 | Profile author | Required when the language distinguishes editions. Otherwise absent means not applicable. |
| sources | Set<SemanticSource>, 1..n | Collector/author | Primary authority for every claimed mapping. Missing source creates a gap. |
| execution | ExecutionAssumptions, 1 | Profile author | Runtime/target/settings that affect semantics. Missing required assumptions block compatibility. |
| mappings | Map<FeatureID,Ref<FeatureMapping>>, 1 | Profile author | Exact feature meanings and pattern bindings. Empty means no mapped features. |
| capability | Map<FeatureID,CapabilityState>, 1 | Maintainer | Declared specification, implementation, and test coverage. Missing entries cannot mean support. |
| dependencies | Set<Ref<Entity>>, 1..n | Profile author | Pattern, composition, source, and method revisions. Missing prevents impact analysis. |
| changes | Set<ImpactRecord>, 0..n | Profile author | Revision effects on contracts and evidence. Empty only for an initial profile revision. |

A SemanticSource contains URL or local authority identifier, retained collection Artifact, release identity, locator, claimed feature IDs, collection time, and retention scope.

Each field occurs once except feature IDs, a nonempty set. The collector owns provenance. The profile author owns the claim-to-locator mapping.

A collection Artifact can retain claim summaries instead of source bytes when retention scope says so. It cannot masquerade as a raw-source snapshot.

A fetched page does not establish all language semantics. An unavailable archive or ambiguous source version leaves that versioned claim unresolved.

| FeatureMapping field | Type and cardinality | Meaning and missing rule |
| --- | --- | --- |
| feature | Name, 1 | Exact language construct or semantic rule. Missing prevents mapping. |
| sourceClaims | Set<SemanticSourceRef>, 1..n | Versioned support for every language-specific assertion. Missing is source-pending. |
| patterns | Set<Ref<PatternDefinition>>, 1..n | Exact common meanings being mapped. Missing prevents compatibility checking. |
| settingsWhen | Predicate, 1 | Conditions that enable this rule. Unknown conditions create an assumption gap. |
| inputDomain | Ref<Type>, 1 | Admitted values with units, ranges, null and missing distinctions. Missing blocks comparison. |
| observable | Defined typed relation, 1 | Result, error, order, state and effects. Missing is unmapped semantics. |
| bindings | Map<SlotPath,Binding>, 1 | Concrete profile choices for common pattern slots. Empty does not invent defaults. |
| divergences | Set<Divergence>, 0..n | Named differences from other profiles or common requirements. Empty means no difference claimed within this row. |
| methods | Set<MethodRequirement>, 1..n | Category, exact method/version or explicit unavailable method, scope and limits. Missing is a checking gap. |
| positive | Case, 1 | Matching behavior under stated settings. Missing prevents conceptual review. |
| conflict | Case, 1 | A requirement or setting that changes the result. Missing prevents boundary review. |

The profile author owns all FeatureMapping fields. Unknown values remain Unknown decisions rather than inferred language defaults.

ExecutionAssumptions contains target architecture, numeric widths, compiler/interpreter identity, semantic settings, host, resource policy, and scheduler policy.

Each field has cardinality 1 and a typed value or explicit NotApplicable proof. Missing is different from an implementation-chosen setting.

Semantic settings are Map<Name,TypedValue>. An explicit empty map claims no relevant settings within this profile's covered feature scope.

An ImpactRecord names prior/new profile revisions, changed feature IDs, affected contracts/criteria/evidence, dependency traversal, and unresolved impact scope.

All fields are required, except unresolved impact scope can be an explicit empty set. The author owns these records.

PROFILE-001 | The author must bind every claimed language rule to an exact primary-source release and locator.

PROFILE-002 | Each selected profile must identify all execution assumptions that affect its mapped guarantees.

PROFILE-003 | The checker must preserve language-specific results, errors, evaluation order, effects, and lifetime rules.

PROFILE-004 | The controller must reject an unmapped required feature as a capability gap.

PROFILE-005 | The controller must not substitute another profile without an explicit contract decision.

## Compatibility and coverage

A compatible mapping admits the contract's entire required input domain and preserves every required observable guarantee.

A conflicting mapping has a concrete admitted case with a forbidden observable, missing guarantee, stronger caller obligation, or unsupported effect boundary.

Unknown support means meaning, assumptions, source, or method cannot establish compatibility. It is not a proven conflict or a pass.

| CapabilityState axis | Values | Meaning |
| --- | --- | --- |
| Specification | source-pending, proposed, reviewed, accepted | Conceptual mapping status for exact source and pattern revisions. |
| Implementation | unavailable, implemented | Whether a named actual adapter/checker exists. Implementation identity is required when implemented. |
| Test evidence | absent, retained-subset, retained-coverage | Exact observed tests and covered feature/settings cases. No blanket tested status. |
| Compatibility | compatible, conflict, unknown, unsupported | Result for one contract guarantee and selected feature/settings tuple. |

These axes are independent. A sourced semantic mapping does not implement its checker or test a compiler.

1. Resolve the package, profile, source, pattern, and method identities.
2. Expand required guarantees into feature/settings tuples and exact slot obligations.
3. Compare admitted inputs and observable relations under those assumptions.
4. Retain a scoped compatibility result, counterexample, or unresolved gap.
5. Bind the result to those exact revisions and checking limits.

T06 `profile.check` owns repeatable compatibility checks. `slots.check` checks profile bindings. `composition.check` checks cross-pattern guarantees.

`cases.evaluate` covers declared finite divergence cases. `revision.impact` identifies changed applicability. Arbitrary semantic equivalence remains unsupported or indeterminate.

PROFILE-006 | The maintainer must separate specification, implementation, and observed test coverage for each feature.

PROFILE-007 | The reviewer must record a concrete counterexample before labeling a mapping as a proven conflict.

PROFILE-008 | The checker must retain unknown source, setting, or method coverage without inventing compatible semantics.

PROFILE-009 | The reviewer must compare guarantees instead of matching syntax or feature names alone.

## Candidate profile baseline

Decision D08 belongs to the repository owner. Proposed candidates are Rust edition 2024 and ECMAScript 2025.

The ECMA-262 16th edition, June 2025, is pinned by E1. The proposed Rust release is 1.85.0 with edition 2024.

R1 establishes edition 2024's stable release. R2–R7 are current unversioned documentation, not verified Rust 1.85.0 semantics.

The local compiler observation does not choose the profile specification release or establish conformance. No compiler test is claimed here.

| Candidate | Planned mapping scope | Current state |
| --- | --- | --- |
| LP-RUST2024 | Rust 1.85.0, edition 2024. Explicit integer/settings and nine divergence groups below. | Release established. Semantic archive verification pending. Implementation unavailable. Tests absent. |
| LP-ECMA2025 | ECMA-262 16.0, June 2025. Number and BigInt remain separate mappings. | Selected source-backed semantics proposed below. Host/runtime gaps remain. Implementation unavailable. Tests absent. |

Neither candidate is owner-selected. The following mapping section distinguishes sourced scoped rules from unresolved profile requirements.

| T03 family and exact entry group | Rust candidate | ECMAScript candidate | Required responsibility |
| --- | --- | --- | --- |
| VAL-LITERAL/RECORD/COLLECTION/VARIANT | Pending | Pending | Numeric domain, representation, equality, optional/null and missing behavior. |
| EXP-EVAL | Pending | Pending | Arithmetic, rounding, conversion, order and short circuit. |
| FLOW-SEQUENCE/BRANCH/MATCH/LOOP/EXIT/RECURSION | Pending | Pending | Flow, exit activation, continuation and bounded/ongoing semantics. |
| FN-OPERATION/CALLBACK | Pending | Pending | Call signature, evaluation, failures, effects and captured lifetimes. |
| FAIL-RESULT/EXCEPTION/RETRY/DEADLINE/CANCEL/CLEANUP | Pending | Pending | Failure representation, unwinding, cancellation, cleanup and external timing boundaries. |
| ABS-INTERFACE/GENERIC/INHERIT/COMPOSE | Pending | Pending | Substitution, dispatch, extension and composition differences. |
| STATE-MACHINE/RESOURCE | Pending | Pending | Ownership, state guarantees and lifetime boundaries. |
| CON-TASK/COORDINATE | Pending | Pending | Async runtime assumptions, ordering, shared state and synchronization. |
| EXT-IO/PERSIST/TRANSACTION/MESSAGE/IDEMPOTENCY/COMPENSATION | Pending | Pending | Host/service guarantees beyond language syntax. No language-only support inference. |
| ORG-MODULE | Pending | Pending | Imports, visibility, initialization and dependency cycles. |

Pending means no asserted support for that entire entry group. Selected rows below specify narrower mappings without implying complete family support.

## Source-backed mapping candidates

Each row inherits the exact source-pack Artifact above, collection date 2026-11-12, summary-only retention, and proposed specification status.

All rows have implementation unavailable and test evidence absent. Methods name T06 categories whose implementations are unavailable for these mappings.

| Feature and pattern targets | Scoped mapping and assumptions | Positive/divergence case | Source and checking limit |
| --- | --- | --- | --- |
| ECM-NUM-DIV: EXP-EVAL p1 divide | Number uses binary64 division. Domain is finite Number inputs with nonzero divisor. Bind resultType=Number, division relation=fractional binary64, not Integer truncate-zero. | -7/2 yields -3.5. A common contract requiring exact integer -3 has a counterexample. | E1 Number type and Number::divide. P/C/E must preserve fractional result and numeric representation. Common EXP-EVAL needs a declared Number-specific relation extension before direct instantiation. |
| ECM-BIG-DIV: EXP-EVAL p1 divide | BigInt division truncates toward zero. Common Int values map exactly to BigInt mathematical values. Denominator must be nonzero for the normal-result scope. | -7n/2n yields -3n. Exact mathematical output maps to Int(-3). Compatible within this scoped relation, with methods still unavailable. | E1 BigInt type and BigInt::divide. Bind divisionRule=truncate-zero. No host resource bound or whole-language compatibility follows. |
| RUST-INT-DIV: EXP-EVAL p1 divide | Current R2 reports primitive integer division toward zero and zero-divisor panic. Proposed i32 input domain excludes zero divisors and signed overflow case. | Current-source rule derives -7_i32/2_i32=-3. Rust 1.85.0 profile result remains unverified until its archive is checked. | R2 arithmetic/overflow, unversioned. P/E cannot pass versioned profile evidence from this note alone. |
| NUM-OVERFLOW: VAL-LITERAL p1, EXP-EVAL p1 | E1 Number has binary64 values including infinity/NaN. E1 BigInt has no fixed language-level bit width. R2 overflow behavior needs exact compiler settings and remains release-unverified. | Number 1/0 permits infinity and conflicts with a finite-only result guarantee. Fixed-width wrap/reject policies cannot be silently assigned to BigInt. | E1 Number/BigInt and division. R2 overflow. Settings, exceptional values and external resource bounds need separate bindings. |
| NULL-MISSING: VAL-VARIANT p1, VAL-RECORD p1 | E1 distinguishes undefined and null. Property absence also depends on object/prototype rules. R4 describes Some/None, but is unversioned. | Known Null maps only from declared null. Undefined uses a separately declared Undefined singleton/variant. Absent own property remains a property-state observation, not automatically null. | E1 Undefined/Null/object methods. R4 current Option. Mapping needs explicit own-property/prototype policy and an Undefined type definition. None is not automatically JavaScript null. |
| EVAL-ORDER: EXP-EVAL p1 | R3 lists left-to-right operand categories and a separate compound-assignment rule. E1 IsLessThan preserves specified conversion order. Neither claim covers every construct. | An effectful comparison records conversion order under its exact selected algorithm. An unlisted operator remains Pending instead of inheriting a universal order. | R3 operand order is release-unverified. E1 IsLessThan is versioned. Operator-specific P/C/E methods remain unavailable. |
| FAILURE: FAIL-RESULT/EXCEPTION p1 | E1 distinguishes normal/abrupt completions. Number division and BigInt division have distinct zero outcomes. R2 reports panic, with delivery settings unresolved. | BigInt 1n/0n throws RangeError. Number 1/0 can produce infinity. A shared ZeroDivisor result needs an explicit adapter, not label equivalence. | E1 Completion Records and Number/BigInt division. R2 arithmetic/overflow. Panic strategy, catching, and external effects remain explicit gaps. |
| LIFETIME: STATE-RESOURCE, FAIL-CLEANUP p1 | R5 describes borrowing/drop scopes but warns against relying on universal destructor execution. E1 FinalizationRegistry cleanup is optional. | A required release-before-return guarantee cannot be justified by JavaScript finalization registration alone. Explicit cleanup paths and failure outcomes are required. | R5 current borrow/destructor docs, release gap. E1 cleanup-finalization-registry. No deterministic cross-language cleanup equivalence claimed. |
| ASYNC: CON-TASK/COORDINATE p2 | R6 describes Future and polling. E1 defines async functions/Promises and host boundaries. Neither supplies one portable cancellation guarantee. | A contract requiring stopped effects after cancellation remains unsupported until executor/host and acknowledgment semantics are named. Ongoing service semantics remain valid requirements. | R6 current docs, release gap. E1 async functions/Promises/hosts. Runtime scheduler/cancellation/interception evidence is unavailable. |
| INHERITANCE: ABS-INTERFACE/INHERIT p1 | R7 describes traits/supertraits. E1 describes prototype lookup/classes. These mechanisms do not establish the same substitution or state model. | A base guarantee still requires T04 behavioral substitution. A supertrait name cannot prove prototype/class inheritance compatibility. | R7 unversioned traits. E1 objects/class definitions. Dispatch/state/visibility and release mappings remain feature-specific gaps. |

Primary E1 locators: [Number division](https://262.ecma-international.org/16.0/#sec-number::divide), [BigInt division](https://262.ecma-international.org/16.0/#sec-bigint::divide),
[types](https://262.ecma-international.org/16.0/#sec-ecmascript-language-types), [IsLessThan](https://262.ecma-international.org/16.0/#sec-islessthan),
[completion records](https://262.ecma-international.org/16.0/#sec-completion-record-specification-type), [finalization cleanup](https://262.ecma-international.org/16.0/#sec-cleanup-finalization-registry),
[async functions](https://262.ecma-international.org/16.0/#sec-async-function-definitions), [Promises](https://262.ecma-international.org/16.0/#sec-promise-objects),
[hosts](https://262.ecma-international.org/16.0/#sec-hosts-and-implementations), and [classes](https://262.ecma-international.org/16.0/#sec-class-definitions).

The Number relation exceeds EXP-EVAL p1's current divisionRule enum. This is a common-pattern capability gap, not permission to invent a slot.

The permitted next step is a versioned numeric-expression counterpart or reviewed extension with binary64 semantics and cases. Until then direct Number division instantiation is unsupported.

BigInt's scoped truncate-zero relation fits the existing rule. A concrete contract still needs exact value representation, failure domain, runtime assumptions and required evidence.

Undefined needs no new model state. Proposed type ECM-Undefined is Variant{undefined:Unit}, with equality comparing tag and unit payload.

It binds VAL-VARIANT's alternatives, tag and payload with optional-form=false and nullable-form=false. Known Undefined(unit) remains different from Known Null(null).

Property presence is a separately declared record field under VAL-RECORD. Missing own properties are not automatically identified with any lookup result.

### Shared division case

Common requirement DIV-C: mathematical Int numerator/denominator in [-1000,1000], denominator nonzero, no effects, result truncated toward zero.

| Mapping | Input | Expected observable and compatibility |
| --- | --- | --- |
| ECM-BIG-DIV | Mathematical (-7,2) represented as (-7n,2n) | Raw -3n maps exactly to Int(-3). Scoped semantics compatible. Checking and complete-profile gaps remain. |
| ECM-NUM-DIV | Mathematical (-7,2) represented as Number values | Raw -3.5 violates DIV-C. Concrete counterexample establishes conflict. Current common Number counterpart is also missing. |
| RUST-INT-DIV candidate | (-7_i32,2_i32) | Current documentation derives -3. Candidate Rust 1.85.0 compatibility stays Unknown because the archived rule was not verified. |

These are derived source-backed expectations, not compiler executions. The two ECMAScript numeric mappings demonstrate distinct semantics within one language.

The cross-language Rust-versus-ECMAScript comparison remains provisional. This draft does not claim that missing Rust archive evidence satisfies final selected-profile acceptance.

## Representation challenge fixtures

These fictional profiles test the schema. They make no statement about Rust, ECMAScript, or another actual language.

| Case | Exact fixture inputs | Expected conceptual result |
| --- | --- | --- |
| LP-C01 | Shared EXP-EVAL divide on Int(-7),Int(2). Fictional P-TRUNC truncates toward zero. Fictional P-FLOOR rounds down. Both map explicit divisionRule. | P-TRUNC returns -3. P-FLOOR returns -4. A contract requiring -3 conflicts with P-FLOOR for this admitted case. |
| LP-C02 | Contract requires unsupported atomic shared update; selected profile has no mapping or method. | Capability gap names CON-COORDINATE.atomicity and the affected guarantee. No alternate profile is selected. |
| LP-C03 | P-TRUNC r2 changes its rounding relation while evidence targets r1. | Invalidate affected division mappings, contracts and evidence. Preserve r1 historical results. |
| LP-C04 | Numeric rule depends on one semantic setting, but execution.semanticSettings omits it. | Unknown assumption finding. No default build behavior is invented. |
| LP-C05 | Language-neutral contract requires exact mathematical integer addition. A candidate mapping admits only a bounded integer domain. | Domain mismatch unless the contract explicitly narrows its admitted inputs. No silent lowering. |
| LP-C06 | Two names look alike but use different failure or cleanup observables. | Compare full guarantees. Equal spelling supplies no compatibility evidence. |
| LP-C07 | Available source covers only division, while all other matrix entries remain Pending. | Division can become proposed within scope. Other entries remain source/method gaps. No whole-language support follows. |

All fixture fields follow the schema with source=local fixture definition, settingsWhen=true, exact pattern references, and implementation unavailable.

Their tests axis remains absent. A reviewer can inspect the relation outcomes without claiming a running checker.

## Revision and discovery rules

A changed source release, compiler semantic setting, host assumption, mapping, or checking method creates a new profile revision.

Impact traverses feature-to-pattern-to-contract-to-case/evidence dependencies. Unknown edges block narrow reuse instead of pretending unaffected coverage.

Source text changes that preserve meaning still require an exact equivalence assessment before new evidence can reference the old result.

An unsupported feature records its source obligation, selected profile, exact feature/settings tuple, affected guarantee, and candidate next collection/design/check action.

PROFILE-010 | The maintainer must identify affected contracts and evidence after any profile semantics or assumptions change.

PROFILE-011 | The controller must reject stale profile evidence for the current contract target.

PROFILE-012 | The author must retain language-neutral guarantees until an explicit decision selects compatible language bindings.

## T05 handoff and open work

| Obligation | Defined evidence |
| --- | --- |
| T05-001 | Profile release and ExecutionAssumptions schema, PROFILE-001/002, LP-C04. |
| T05-002 | FeatureMapping bindings/methods, compatibility algorithm, family matrix, LP-C01/02. |
| T05-003 | Divergence relation and counterexamples, PROFILE-003/007/009, LP-C01/06. |
| T05-004 | Independent capability axes and candidate states, PROFILE-006, LP-C07. |
| T05-005 | Explicit Pending/unknown/unsupported states, PROFILE-004/008, LP-C02/04. |
| T05-006 | No-substitution and neutral guarantees, PROFILE-005/012, LP-C02/05. |
| T05-007 | ImpactRecord and dependency traversal, PROFILE-010/011, LP-C03. |

Open work: Rust archive verification, raw-source retention where required, runtime/settings bindings, missing Number relation counterpart, D08, upstream acceptance, and review.

This draft supplies scoped source-backed mappings and reviewable gaps. It cannot complete T05 while selected-profile source, capability, owner, and upstream gates remain open.

## See also

- [contract-model.md](contract-model.md): contract representation.
- [CONTRACT.md](../../CONTRACT.md): method rules.
