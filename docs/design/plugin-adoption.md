# Plugin adoption in an active project

Revision: `adoption-draft-3`. Status: provisional design. Designer: `the designer`. Reviewer: `the designer`.

This draft answers T09 r2, SHA-256 `25f4f594934d45f0e68125cfd631bf8356455b00bfe7e43eaeeef970cf816c1a`.

## Inputs and boundary

The workflow attaches contract work to a selected existing slice. It preserves observation, requirement authority, inventory coverage, and execution evidence separately.

The owner authorized this conceptual design. No source change, test execution, runtime migration, or live-service action occurs through it.

Actual design inputs are provisional T05 `profiles-draft-2`, SHA-256 `e297021536f7227f3044d3d55b7f7928bd4c18f2629d65f5f0d98bb35527648f`,
and T07 `skill-draft-2`, SHA-256 `69199f3410755d810f507e93d5a4f49ec9119b8cf3efc8aa28b0c500dd46a9ae`.

The specification observed earlier profile draft 1 at hash `19953adaf9e9f7d6c3f61a76ed3dc3cdefb4fb4a1fc2523696cd1765a6ec2377` without a retained copy.

That historical observation is not reconstructed evidence. This design instead binds the newly retained profile draft 2 and requires upstream reconciliation.

T05/T07 are inputs and close gates. Their owner and upstream acceptance remain absent. D14 belongs to the repository owner.

Typed phase reconciliation uses model-draft-5 `6bfc4bd3a430d5481e020c812de1e47b451715da0ef1be787791b4b2835d9b4c`,
patterns-draft-7 manifest `863f1afd57a13a1246352e52aa6791ed212486bdbec20066eb02372d17c990e3`,
and composition-draft-8 `ca5f3170e6577d542a5eb0c7c9fd4cfa90c2d79abf8c0d3fe149862311b36a1b`.

This changes candidate representation, not observed source facts or proposed requirements. Static values and runtime source relations remain distinct.

The consumed T07 handoff/instruction interface is unchanged. Its earlier loop illustration is not a canonical binding input to this normalized fragment.

## Baseline model

A Baseline identifies exact selected bytes and their crossing dependencies. It does not assert whole-application discovery, behavior correctness, or approval.

| Field | Type and cardinality | Owner | Meaning and missing rule |
| --- | --- | --- | --- |
| identity | ID and revision, 1 each | Analyst | Immutable baseline identity. Missing prevents derived evidence. |
| boundary | Scope with selected paths/declarations/observables, 1 | Owner/analyst | Known selection and explicit exclusions. Unknown selection blocks dependent discovery. |
| files | Set<FileRecord>, 1..n | Collector | Exact source/test/config inputs. Missing paths or digests block inspection. |
| sourceRevision | Commit ID or explicit Unknown, 1 | Collector | Optional repository provenance cannot replace complete file identities. |
| dependencies | Set<BoundaryEdge>, 0..n | Analyst | Imports, callbacks, state, effects and environment crossings. Empty requires positive coverage evidence. |
| profile | Ref<Profile> or explicit language-neutral observation state, 1 | Analyst | Language meaning and limitations. Missing mapping prevents semantic coverage claims. |
| inventory | ResponsibilityInventory, 1 | Analyst | Declared units, observations, mappings and gaps. Missing cannot mean complete. |
| methods | Set<InspectionMethod>, 1..n | Reviewer | How discovery established its coverage. Name-only search is insufficient. |
| currentness | Exact file/dependency comparison record, 1 | Analyst | Inputs checked before submission. Missing prevents current-evidence claims. |

A FileRecord contains canonical selected path, file role, retained Artifact, selection reason, and access mode, each once.

The collector owns these fields. File role is source, test, configuration, requirement, or dependency. Access mode is read-only for this workflow.

A BoundaryEdge contains source unit, target declaration or explicit Unknown, direction, argument/result types, effects/state, guarantees, and evidence locator.

Every field occurs once. The analyst owns the edge. Unknown dynamic target remains an unresolved boundary responsibility, not a broad module label.

An InspectionMethod contains method identity/version, declared input set, coverage domain, output artifact, and limits, each once.

Manual source inspection is a named method with retained declaration/case notes. It is not an implemented parser or exhaustive semantic proof.

ADOPT-001 | The analyst must retain the selected boundary and exact source identities before deriving candidate obligations.

ADOPT-002 | The analyst must record every discovered callback, state, import, and effect crossing the selected boundary.

ADOPT-003 | The collector must preserve unrelated files and runtime configuration during discovery.

## Responsibility and authority records

| Record | Required fields and meaning |
| --- | --- |
| DeclarationEvidence | Exact file Artifact, declaration locator/span, declaration kind, enclosing scope, symbol identity, profile, and discovery method. A substring supplies none of these by itself. |
| Observation | Exact baseline, source locator, observed statement or behavior claim, method, assumptions, and authority=observation. Runtime claims additionally require executed evidence. |
| TestExpectation | Exact test source locator, input, asserted observable, method, execution state, and interpretation limit. Present assertions are not passed tests. |
| CandidateObligation | Proposed clause/instance, source links, affected responsibility, required owner decision, exact base package, and checking impact. It remains proposed. |
| CoverageItem | Responsibility ID, declaration/branch/call/effect locus, observed evidence, mapped pattern/clauses, gap state, and completeness limit. |
| OwnerDisposition | Decision ID, candidate target, keep/change/reject/unknown answer, exact answer source, applicable revision, and authority. Unknown is not approval. |

The analyst owns these records except OwnerDisposition, whose answer belongs to the authorized owner. Every listed field is required.

An empty mapping set means unmapped responsibility. A missing locator prevents traceability. An unexecuted test record uses execution state not-run.

Coverage states are inventoried-mapped, inventoried-unmapped, unresolved-target, outside-selected-boundary, and uninspected.

Coverage completeness states are bounded-method-complete or partial. A partial inventory names its uncovered declarations, dynamic paths, and assumptions.

Mapped does not mean approved, implemented, or tested. Outside-selected-boundary does not mean irrelevant: crossing edges still require declared guarantees.

ADOPT-004 | The analyst must separate code observations, test expectations, proposed requirements, and approved requirements.

ADOPT-005 | The analyst must preserve exact source locations for every observation and test expectation.

ADOPT-006 | The analyst must expose each unmapped or unknown responsibility within the declared inventory.

ADOPT-007 | The reviewer must require declaration evidence instead of function-name substring matches.

## D14 proposal: baseline granularity and coverage

Proposed granularity: selected declarations plus their reachable responsibility edges, bound to exact source files and language/profile settings.

For every selected callable declaration, inventory admitted inputs, branches, exits, loops, calls/callbacks, reads/writes, effects, errors and lifecycle responsibilities.

For each state or type declaration, inventory its admitted values, initialization, visibility, ownership, mutation and references crossing the boundary.

| Profile/method state | Coverage permitted | Required limit |
| --- | --- | --- |
| Verified declaration/flow adapter for an exact profile | Bounded-method-complete for its declared syntax and static path domain | Retain parsed declarations, edge coverage, supported constructs and unresolved dynamic behavior. |
| Manual review of exact selected bytes | Explicitly partial semantic inventory; complete textual declaration list only when independently checked | Retain the declaration list and branch/call observations. No automatic grammar or dynamic-target claim. |
| Unsupported syntax, generated code, macros, reflection or foreign boundary | Unknown or unsupported affected responsibility | Do not skip the construct or infer behavior from its name. |
| Missing dependency or profile assumption | Partial inventory and affected compatibility gap | Do not label resulting candidates ready for completion. |

This draft does not register a language adapter. T05's incomplete Rust/archive/runtime coverage remains visible in the actual slice below.

D14 remains proposed. Its owner decision controls accepted granularity and evidence requirements, not permission for routine read-only design work.

## Workflow and source changes

1. Record selection, retained file identities, source revision, profile and known requirement sources.
2. Identify declarations and responsibility edges through the declared method.
3. Record source observations and test expectations without promoting their authority.
4. Select exact prepared pattern counterparts for supported responsibilities.
5. Preserve unmapped responsibilities and ask only surviving owner-only behavioral questions.
6. Recheck file and dependency identities before submitting revision-bound candidates.
7. Send candidates, findings, decisions, coverage and limits to T11 through the T07 handoff.

A changed file or dependency creates a new baseline. Affected observations, candidates, tests and coverage become stale for the new baseline.

Unchanged file bytes do not preserve semantics when a selected profile, generated input, external schema, callback binding, or environment assumption changes.

Missing impact edges block narrow reuse. The workflow reinspects the affected boundary and records the comparison before proposing completion.

Code and tests can disagree without either becoming requirement authority. A passing executed test still cannot approve unwanted behavior.

A change requiring data/state migration, cutover, compatibility or rollback invokes T10 planning obligations. Adoption itself does not perform those actions.

ADOPT-008 | The analyst must mark affected interpretation records stale after source or dependency identities change.

ADOPT-009 | The analyst must recheck current baseline identities before submitting contract candidates.

ADOPT-010 | The workflow must hand unresolved choices and unmapped responsibilities to the shared completion loop.

## Actual bounded source inspection

Selected implementation: `src/batch.rs`, SHA-256 `42ddb283820de7d086a9ca546ed3154e035c0bcafb68091004b7358770b1fded`.

Selected test expectations: `tests/batch.rs`, SHA-256 `4de7418587bc2d5d431c4c2e0b8221dc64f2a4fc0eecf139318132be1d1aae61`.

The coordinator retained these inputs under RUN-002. Repository commit identity is unknown here. File hashes establish the inspected bytes.

Boundary: the public run wrapper, its Divide selection, default batch body, input/value declarations, and selected tests. No whole-project inventory is claimed.

Excluded from inspected behavior: unrelated CLI/authoring modules, other trait implementations, serde_json internals, runtime resource exhaustion and all live services.

Method: manual full-file reading and declaration/call/branch notes below. Tests were read, not run. No Rust parser or coverage adapter was executed.

| Unit/locator within retained source | Inspected fact | Candidate counterpart and limit |
| --- | --- | --- |
| BatchResult and ItemResult declarations | The file declares InvalidInput/Completed and Ok/ZeroDivisor variants. | VAL-VARIANT p1. These are source declarations, not accepted output requirements. |
| BatchProcessor::apply declaration | The trait declares numerator/denominator parameters and ItemResult output. | ABS-INTERFACE/FN-OPERATION. Other bindings and implementations remain outside this inspection. |
| BatchProcessor::run initial checks | The body contains array conversion, length >100 rejection, object/field checks, integer conversions and [-1000,1000] checks. | FLOW-BRANCH, VAL-RECORD/COLLECTION, EXP-EVAL. serde_json conversion semantics are a crossing dependency. |
| BatchProcessor::run pairs collection | Validation pushes pairs before the later map/collect pipeline calls self.apply. | FLOW-SEQUENCE/LOOP and FN-CALLBACK. Exact language/library iteration semantics need the selected profile and dependency evidence. |
| Divide::apply | Explicit denominator==0 branch returns ZeroDivisor. Other branch contains numerator/denominator expression. | FLOW-BRANCH, FAIL-RESULT, EXP-EVAL. Division semantics are profile-bound, not inferred from the slash alone. |
| Public run wrapper | The source creates Divide and calls processor.run(input). | ABS-COMPOSE/FN-OPERATION. This selection does not cover arbitrary BatchProcessor implementations. |
| Test processor_validates_before_apply_and_calls_once_in_order | Assertions expect no calls after invalid later input, and ordered calls including duplicates. | TestExpectation, execution=not-run. Existing assertions cannot approve the behavior. |
| Test signed_division_and_boundaries | Asserted cases include (-7,2)→-3 and declared numeric boundaries. | TestExpectation plus a candidate EXP-EVAL mapping. Not compiler evidence. |
| Remaining named tests | Assertions concern empty/repeated calls, order/duplicates/zero divisors, invalid inputs, maximum length and subtraction-oracle comparisons. | Explicit finite test-expectation inventory. No execution, line coverage, or exhaustive correctness claim. |

The source declaration list also includes struct Divide. The tests declare pair helper, RecordingProcessor state, its apply body, and seven test functions.

Those test helpers are evidence machinery within the selected test boundary. They do not create production state or callback guarantees automatically.

| Crossing responsibility | Recorded boundary state |
| --- | --- |
| serde_json::Value conversion methods | External library semantics not collected by this ticket. Selected source/type identity and version need baseline completion. |
| BatchProcessor::apply dispatch | Selected Divide body inspected. Generalized trait claims and other implementations remain uninspected. |
| Rust integer evaluation and iteration | T05 candidate mapping/archive/settings gap remains. No whole-language support claim. |
| Allocation/process failure | Outside the current example's execution assumptions. Proposed scope exclusion requires approved requirement authority. |
| Existing example contract | Separate draft requirement source, acceptance pending. Code/test agreement cannot accept it. |

Coverage result: partial semantic inventory, explicit source/test declarations and listed local responsibilities. No language-adapter certificate establishes completeness.

Candidate draft fragment: an operation with explicit input validation, ordered processing, a selected division binding, and tagged per-item errors.

Its proposed slots retain source links above. Retry, persistence, concurrent tasks and live effects are not inserted because this boundary supplies no such requirement.

Owner decisions remain open: retain input bounds, signed division rule, continuation after ZeroDivisor, output shapes, assumptions and public trait scope.

### Populated candidate contract fragment

Package `batch-adoption@candidate-r1` is proposed, incomplete, and not an implementation target. Its source authority remains observation or draft requirement.

Scope is the selected public wrapper and Divide path. Actor BatchProcessor is a proposed role. Pair has required numerator/denominator fields.

Proposed field values are Int[-1000,1000]. Proposed batch length is 0..100. These known candidate values still need D-BOUNDS owner disposition.

| Instance and definition | Populated typed bindings | Explicit missing meaning |
| --- | --- | --- |
| batch-items@candidate-r1, FLOW-LOOP p3 | kind=Known collection, itemType=Known Pair Type, order=Known ascending, continuation=Known bounded, body=Known divide-item reference, resultPolicy=Known collect, termination=Known remaining count reaches zero. Static items key absent. | S-items is its proposed typed binds-supplier. Validation availability and child continuation remain unresolved. failure=Unknown D-CONTINUE. |
| zero-branch@candidate-r1, FLOW-BRANCH p1 | condition=Known denominator equals zero; then=Known ZeroDivisor result constructor; otherwise=Known child; else=Known divide-expression reference | Source/profile-dependent input/result bindings remain provisional. |
| divide-expression@candidate-r1, EXP-EVAL p3 | kind=Known operator, operator=Known divide, arguments=Known list of numerator/denominator Expr references, resultType=Known Int, order=Known left-to-right, divisionRule=Known truncate-zero, zero=Known reject under nonzero branch | D-DIVISION owner acceptance and versioned language mapping remain open. Expr references are static syntax, not raw future Int values. |

Package relations explicitly map validated pair fields to the branch/expression, branch result to the loop, and loop results to Completed.

S-items is indexed in package.relations with relationType=binds-supplier, source=RuntimePort(validate-input,result), target=DataSlot(batch-items,items),
mapping=validated-success-payload of exact List<Pair> type, phase=before-consumption, and failure=return InvalidInput without item effects.

Its explicit ordering constraint validates/materializes items before loop input consumption. This uses T04's declared EvaluationPhase value, not a new phase tag.

Validate-input is a proposed FN-OPERATION with declared Result<List<Pair>,InvalidInput> output. Its raw-input domain and validation guarantee remain Unknown D-VALIDATION.

Therefore S-items has a declared target type but unresolved producer availability. It is not claimed to satisfy the completed-contract gate yet.

Numerator/denominator Expr instances resolve separate typed FormalParameter declarations sourced from the body role-input Pair's fields.

Their canonical name locations end at those declarations. RuntimePort field paths select the Pair fields at invocation, not fields inside a static Type reference.

The body signature supplies the loop's output/failure element types. Its incomplete signature is an explicit typing/checking gap, not inferred from an example result.

These are normalized candidate records. Earlier input-list mapping shorthand translates to S-items, never to a Known Expression<List<Pair>> in a List slot.

Each relation targets exact candidate instance revisions. It remains proposed and requires T04 typing, failure, and effect review.

The following candidate clauses have paired typed meanings in the table below. They do not establish approved requirements.

ADOPTIONEX-001 | BatchProcessor must return InvalidInput when proposed validation rejects the entire batch.

ADOPTIONEX-002 | BatchProcessor must process valid batch items in ascending input-index order.

ADOPTIONEX-003 | BatchProcessor must preserve ZeroDivisor as a per-item result when the owner selects continuation.

| Clause | Modality/actor/guard | Typed observable meaning and source |
| --- | --- | --- |
| ADOPTIONEX-001 | obligation, BatchProcessor, proposed validation predicate=false | Exactly InvalidInput. Candidate input type and initial source checks provide its proposal source. |
| ADOPTIONEX-002 | obligation, BatchProcessor, proposed validation predicate=true | Loop invokes each selected item in ascending index order. Ordered-call test expectation and draft BATCH clauses provide proposal sources. |
| ADOPTIONEX-003 | obligation, BatchProcessor, D-CONTINUE selects continuation and denominator=0 | Per-item Error(ZeroDivisor) is appended without aborting later items. Source test expectation provides proposal source. |

Undefined acceptance of a candidate guard is not an implicit permission. D-CONTINUE remains Unknown and blocks affected expected results.

| Decision | Owner question and exact affected target |
| --- | --- |
| D-BOUNDS | Retain proposed numeric/batch limits? Pair and batch type revisions. |
| D-DIVISION | Retain truncate-zero and explicit zero branch? divide-expression and zero-branch. |
| D-CONTINUE | Continue later items after ZeroDivisor? batch-items.failure and ADOPTIONEX-003. |
| D-OUTPUT | Retain the proposed tagged output shapes and result list? Result type and batch operation guarantees. |
| D-SCOPE | Limit adoption to Divide wrapper or include arbitrary trait implementations? Scope and boundary edges. |
| D-VALIDATION | Resolve raw-input domain, validation guarantee and producer availability? Validate-input and S-items. |

Every decision is open, owned by the contract owner, with answer Unknown and source references to this inspected candidate revision.

This is an actual populated draft fragment, rather than an assertion that a draft exists. A complete T02 package still needs its remaining fields and evidence.

## Adversarial cases

These cases are declared hypothetical changes to a baseline, not accusations about the current inspected files.

| Case | Input and observed fact | Authority/coverage result | Required next action |
| --- | --- | --- | --- |
| AD-C01 | A selected body has an undocumented Error branch. | Source observation, inventoried-unmapped responsibility. No requirement is inferred. | Preserve the exact branch locator and ask whether its outcome is required. |
| AD-C02 | A test asserts behavior the owner rejects. | TestExpectation remains historical, even if an executed run passed. Owner answer controls the proposed target requirement. | Record conflict, decision source and affected candidate/test change. Do not rewrite history. |
| AD-C03 | A callback enters an unselected handler with unknown effects. | Unresolved-target BoundaryEdge, including input/result/state guarantees. | Expand the authorized boundary or declare external assumptions. Do not hide callback under module coverage. |
| AD-C04 | Source file changes from hash A to B during inspection. | A-derived records become stale for B. | Retain both baselines and reinspect affected declarations/edges before submission. |
| AD-C05 | Function-name search finds processPaymentHelper but no declaration evidence. | No declaration or responsibility coverage established. | Use exact parsed/manual declaration evidence under D14. |
| AD-C06 | Code returns Error while a test expects Ok. | Two distinct observations, no approved requirement. | Record both locators and a behavioral decision. Test presence supplies no precedence. |
| AD-C07 | Imported schema/profile changes while selected source bytes remain identical. | Affected interpretation and mapping evidence become stale. | New dependency baseline and compatibility review. |
| AD-C08 | Desired adoption includes a stored-state cutover. | T10 migration obligations arise before implementation. | Handoff existing/target state and unresolved transition decisions. No automatic migration occurs. |
| AD-C09 | Selected files are known, but generated/dynamic paths remain uninspected. | Partial inventory with named gaps. | Preserve those gaps in T11 input. Do not claim whole-project completion. |

## T09 handoff

The handoff contains exact baseline and package references, selected profile, proposed instances/clauses, source/test links, coverage records, open decisions and currentness evidence.

T07 carries it as proposed/observed records. T11 checks completion only after required meaning, coverage, source, profile, and owner gates resolve.

| T09 obligation | Evidence and cases |
| --- | --- |
| T09-001 | Baseline boundary/edges, actual crossing table, ADOPT-002, AD-C03/09. |
| T09-002 | Exact retained FileRecords and sourceRevision rules, ADOPT-001. |
| T09-003 | Observation/TestExpectation locators, actual source/test notes, ADOPT-005, AD-C01/02. |
| T09-004 | Authority record types, ADOPT-004, AD-C02/06. |
| T09-005 | CoverageItem states and unmapped gaps, ADOPT-006, AD-C01/03. |
| T09-006 | D14 method granularity, partial actual inventory, AD-C05/09. |
| T09-007 | Source/dependency comparison and staleness, ADOPT-008/009, AD-C04/07. |
| T09-008 | Read-only baseline, explicit exclusions, ADOPT-003. |
| T09-009 | Shared-loop handoff, ADOPT-010, AD-C08/09. |

Remaining gates: accepted T05/T07, D14, finalized source/profile dependencies, and independent review. This design establishes no automatic adoption tool.

## See also

- [scratch-workflow.md](scratch-workflow.md): scratch workflow.
- [migration-workflow.md](migration-workflow.md): migration workflow.
- [CONTRACT.md](../../CONTRACT.md): method rules.
