# Method contract

Revision: 0.1.4. Status: draft.

Scope: authoring and reviewing CAAC contracts. Authority for this draft: this file.

## Scope

In scope: method rules for authoring and reviewing CAAC contracts.
Out of scope: implementation, runtime, publication, legal enforceability.
Actors: author, reviewer, owner.
Inputs: source requirements, existing contracts, owner decisions.
Observables: clauses, cases, findings, review results, completion state.
Exclusions: runtime correctness, performance, security, and deployment.

## Meaning of terms

| Term | Meaning |
| --- | --- |
| Contract | A set of clauses that defines permitted behavior within a stated scope. |
| Clause | One independently reviewable obligation, prohibition, or permission. |
| Actor | The entity responsible for an operation. |
| Observable | An output, error, state change, or external effect that the contract exposes. |
| Condition | A predicate with defined inputs and a Boolean result. |
| Invariant | A predicate that must hold at every state boundary named by the contract. |
| Extension | A contract that inherits obligations from a named base contract. |
| Open decision | Missing meaning that prevents a reviewer from determining permitted behavior. |
| Owner | The person authorized to accept or change contract requirements. |
| Reviewer | The person who checks clauses and cases against the declared acceptance criteria. |
| Size profile | A presentation and partitioning policy for the same scope and required semantic content. |
| Upper level | A grouping level above an article. The order from outermost to innermost is part (편), chapter (장), section (절), subsection (관). |
| Article (조) | The main unit for placing contract content. |
| Lower level | A grouping level within an article. The order from outermost to innermost is paragraph (항), item (호), subitem (목). |
| Direct content | Contract content placed in an article or lower group without an intervening group. |
| Module | A named division of an XL contract's scope with its own article hierarchy. |
| Descendant | A node reached by following one or more child links from another node. |
| Siblings | Nodes that share a parent or occur in the same root list. |
| Display number | An ordinal assigned to a presentation node from its position in the hierarchy. |
| Article order | The order of articles in a depth-first traversal of the hierarchy in declared sibling order. For XL, traverse each module in declared module order. |
| Semantic entity | An identified object in the semantic source, such as a clause, definition, operation, exception, or case. |
| Semantic identity | The stable package-local ID of a semantic entity or open decision. |
| Semantic reference | An authored link to a target's semantic identity. |
| Semantic content | Clauses, definitions, operations, exceptions, cases, evidence requirements, open decisions, other semantic entities, and their semantic references. |
| Semantic revision | The revision of the semantic content of a package or of an individual entity or open decision. |
| Primary placement | The location in a review file that presents a semantic entity as authored content. |
| Entity placement | A reference to a semantic entity for its primary placement. |
| Review location | The location in a review file that presents an open decision. |

`must` states an obligation. `must not` states a prohibition. `may` states a permission with explicit bounds.

An implementation choice is permitted only when the contract leaves that choice open without changing its observable guarantees.

## Core rules

CAAC-001 | The author must define the scope, actors, inputs, observables, and exclusions before defining operations.

CAAC-002 | The author must state every decision that changes permitted observable behavior within that scope.

CAAC-003 | The author must record missing or conflicting meaning as an open decision.

CAAC-004 | The implementer must not resolve an open decision by inventing behavior.

CAAC-005 | The author must define each domain term before its first normative use.

CAAC-006 | The author must use each defined term with one meaning throughout the contract.

CAAC-007 | Each clause must identify its responsible actor and one obligation, prohibition, or permission.

CAAC-008 | Each clause must have an ID that is unique within the complete contract set.

CAAC-009 | The author must preserve each clause ID until the clause retires.

CAAC-010 | The author must not assign a retired ID to another clause.

CAAC-011 | Each normative sentence must contain no more than 20 words, excluding its clause ID.

CAAC-012 | The author must replace subjective qualifiers with defined predicates, values, or bounded choices.

CAAC-013 | The author must not use a word or phrase that permits conflicting interpretations of an obligation.

Words such as "appropriate", "normally", "quickly", and "as needed" require explicit definitions when they affect behavior. Short sentences alone do not establish precise meaning.

## Programming patterns

CAAC-014 | The author must express every programming pattern that the required behavior uses through the rules below.

CAAC-015 | The author must omit patterns that the required behavior does not use.

The following table defines the required elements for @CAAC-014. A required but undefined element is an open decision.

| Pattern | Required meaning |
| --- | --- |
| Types and values | Define valid values, units, ranges, precision, defaults, missing values, and equality. |
| Operation | Define the actor, inputs, preconditions, results, failures, side effects, and completion condition. |
| Branch | Define predicates and precedence. Cover every admitted case. State whether several branches can execute. |
| Loop | Define the collection or counter, order, updates, empty case, termination, and response to an item failure. |
| State | Define the initial state, allowed transitions, forbidden transitions, and the boundaries where each invariant holds. |
| Exception | Define each failure trigger, error result, propagation, partial effects, and recovery action. |
| Retry | Define eligible failures, attempt count, delay, deadline, duplicate effects, and the final result. |
| Abstraction | Define the common obligations and the exact operations or values that remain abstract. |
| Inheritance | Name the base revision, inherited clauses, extension points, and restrictions on replacement. |
| Composition | Define each dependency, call order, input and result mapping, and failure propagation. |
| Recursion | Define the base case, progress measure, depth bound, and failure at that bound. |
| Concurrency | Define permitted ordering, shared state, atomic operations, cancellation, and duplicate execution. |
| Resources | Define acquisition, ownership, release, limits, and cleanup after failure. |

CAAC-016 | The author must separate input obligations from guarantees that apply after those obligations hold.

CAAC-017 | The author must define rejection behavior for inputs outside an operation's admitted domain.

CAAC-018 | The author must define dependency failures or explicitly exclude them through named environmental assumptions.

CAAC-019 | Each executable profile must bind every abstract operation and parameter before implementation starts.

CAAC-020 | An extension must preserve every inherited guarantee for every input admitted by its base contract.

CAAC-021 | An extension must not strengthen inherited caller obligations or weaken inherited result guarantees.

CAAC-022 | An extension must replace behavior only at an extension point declared by its base contract.

CAAC-023 | The author must resolve conflicting inherited obligations before accepting an extension.

An abstract contract can remain incomplete at its declared extension points. A concrete profile cannot.

## Size and authority

CAAC-024 | The author must give each normative clause one canonical location.

CAAC-025 | Supporting documents must reference canonical clauses without adding or changing their obligations.

CAAC-026 | Each new clause must serve a current requirement or a necessary constraint within the declared scope.

CAAC-027 | The author must remove redundant clauses without removing distinct obligations or evidence.

CAAC-028 | The author must preserve supplied source identities and revisions when deriving obligations from external material.

CAAC-029 | The author must distinguish observed implementation behavior from owner-approved requirements.

A contract needs additional files only when their distinct purpose justifies separate maintenance. File count alone cannot establish completeness.

## Contract review

CAAC-030 | The reviewer must inspect the contract before using implementation code to infer its meaning.

CAAC-031 | The reviewer must construct cases for each branch, boundary value, loop termination rule, and declared failure.

CAAC-032 | The reviewer must record each case's inputs, applicable clauses, expected observables, and unresolved decisions.

CAAC-033 | The reviewer must reject conflicting expected outcomes unless an explicit permission allows those outcomes.

CAAC-034 | The reviewer must check extensions against the admitted inputs and guarantees of their base contracts.

CAAC-035 | The owner must accept the exact contract revision before the project treats that revision as an implementation target.

CAAC-036 | A contract must remain draft while any required decision or owner acceptance is missing.

CAAC-037 | A behavior change must trigger review of affected clauses, cases, extensions, and dependent contracts.

CAAC-038 | Review records must identify the contract revision, reviewer, cases, findings, and remaining limits.

CAAC-039 | The reviewer must keep language checks, contract review, and implementation test results separate.

## Contract sizing

CAAC-040 | The author must choose the smallest contract unit that captures the required behavior.

CAAC-041 | The author must use T-shirt sizing (XS, S, M, L, XL) to estimate contract scope before writing.

CAAC-042 | A size-XS contract must target one function or expression. A size-XL contract must target one system or platform.

CAAC-043 | The author must split a size-XL contract into multiple files by module.

CAAC-044 | The author must preserve canonical clauses, definitions, exceptions, cases, and evidence requirements when changing a size profile.

CAAC-045 | The author must not use clause-count quotas to determine completeness.

### Size profiles

The file column counts files that contain articles. Indexes and other supporting files are outside that count.

| Size | Permitted upper levels | Article structure | Files containing articles |
| --- | --- | --- | --- |
| XS | None | One unnumbered article with direct content | One |
| S | None | Articles with direct content or lower levels | One |
| M | Chapter, section, subsection | Articles with direct content or lower levels | One |
| L | Part, chapter, section, subsection | Articles with direct content or lower levels | One |
| XL | Part, chapter, section, subsection within each module | Articles with direct content or lower levels | One distinct file per module, with at least two modules |

CAAC-046 | The author must use the hierarchy and file boundaries in the selected size profile.

CAAC-047 | The author may omit any permitted upper level.

CAAC-048 | The author must place retained upper levels in their defined order on each branch.

CAAC-049 | Each upper-level group and XL module must have at least one descendant article at any permitted depth.

CAAC-050 | Each lower level must have the immediately preceding level as its parent, with the article preceding paragraphs.

CAAC-051 | The author must place the articles of each XL module together in that module's file.

CAAC-052 | The author must number S through XL articles consecutively from 1 in article order across the contract.

CAAC-053 | The author must number each kind of upper or lower level consecutively from 1 among siblings in declared order.

CAAC-054 | The author must preserve semantic content when changing only presentation.

CAAC-055 | The author must preserve applicability, inheritance, precedence, and execution order when changing only presentation.

CAAC-056 | The author must preserve semantic identities and semantic revisions when changing only presentation.

CAAC-057 | The author may mix articles and permitted upper-level groups among siblings in declared order.

CAAC-058 | The author must give each article and lower group exactly one nonempty list of permitted contents.

Each alternative in the table specifies the permitted contents of a whole list.

| Group | Permitted contents |
| --- | --- |
| Article | Entity placements or paragraphs |
| Paragraph | Entity placements or items |
| Item | Entity placements or subitems |
| Subitem | Entity placements |

CAAC-059 | The author must give each semantic entity exactly one primary placement across the review files.

CAAC-060 | The author must give each open decision exactly one review location.

CAAC-061 | The author must place each XL open decision in the review file of its assigned module.

Display numbers and file paths identify presentation locations. Clause IDs identify obligations and remain subject to @CAAC-009.
Derived trace addresses use the resulting package revision.

Acceptance establishes a reviewed implementation target. It does not establish exhaustive correctness or legal enforceability.

## Sizing implementation status (non-normative)

The typed candidate CLI supports an XS article with one operation or one typed expression. An expression case binds directly to its expression and declares the expected result. Flat S also supports expression targets. S through XL support subitems, M and L support upper-level hierarchies, and XL writes one review file per module. The author selects the scope and size. See [candidate authoring](docs/candidate-authoring.md) for supported shapes and CLI limits.

## Case status (non-normative)

These cases test CAAC's own rules. They are expected results derived from the
contract. They are not records of executed tests.

## Contract cases

| Case | Input | Expected result | Clauses |
| --- | --- | --- | --- |
| Missing scope | Contract without scope section | Rejected by CAAC-001 | CAAC-001 |
| Missing actor | Clause without actor | Rejected by CAAC-007 | CAAC-007 |
| Long clause | Clause with 21 words | Rejected by CAAC-011 | CAAC-011 |
| Duplicate ID | Two clauses with same ID | Rejected by CAAC-008 | CAAC-008 |
| Open decision | Clause with missing meaning | Recorded as OPEN: | CAAC-003 |
| No cases | Contract with clauses but no cases | Rejected by CAAC-031 | CAAC-031 |
| Owner not accepted | Contract treated as target without acceptance | Rejected by CAAC-035 | CAAC-035 |
| Behavior change | Clause changed without review | Review triggered by CAAC-037 | CAAC-037 |
| Smaller profile | Same semantic package with a smaller permitted profile | Clauses, definitions, exceptions, cases, and evidence requirements remain unchanged | CAAC-044 |
| Quota-only completeness | Clause count meets a quota but a required obligation is missing | Rejected despite the clause count | CAAC-044, CAAC-045 |
| Resize identity | A clause moves between permitted view groups | Same clause ID and semantic revision with a changed display location | CAAC-009, CAAC-044, CAAC-056 |
| XS scope | One function or one expression in a flat article | Permitted in one file with no article number | CAAC-042, CAAC-046 |
| XS invalid grouping | One function in an article with a paragraph | Rejected for the XS profile | CAAC-046 |
| S lower levels | An article lists a paragraph with an item. The item has a subitem that places `rule-check` | Permitted in one file. Each lower level starts at 1 under its parent | CAAC-046, CAAC-050, CAAC-053, CAAC-058 |
| M optional groups | A direct article followed by a sibling chapter → section → subsection → article branch | Permitted in one file as Articles 1 and 2. The chapter and section have a descendant article | CAAC-046, CAAC-047, CAAC-049, CAAC-052, CAAC-057 |
| L skips a chapter | A part with a chapter containing Articles 1 and 2, followed by a sibling section containing an article | Permitted. The section is numbered 1 under the part and contains Article 3 | CAAC-046, CAAC-047, CAAC-048, CAAC-049, CAAC-052, CAAC-053 |
| Inverted levels | An L section contains a chapter | Rejected for the upper-level order | CAAC-048 |
| Empty upper branch | An L part has one valid chapter and one section without an article anywhere below it | Rejected for the section without a descendant article | CAAC-049 |
| Skipped lower level | An S article places an item directly under the article | Rejected for the missing paragraph parent | CAAC-050, CAAC-058 |
| Empty lower group | An S paragraph lists zero entity placements and has no items | Rejected for the empty paragraph | CAAC-058 |
| Mixed lower content | An S paragraph lists an entity placement and an item | Rejected because the paragraph combines two alternatives | CAAC-058 |
| Missing placement | An XL module loses its only article or a required clause loses its only placement | Rejected for the missing article or required placement | CAAC-024, CAAC-049, CAAC-059 |
| Repeated entity placement | Both XL module review files give `rule-retry-limit` a primary placement | Rejected for the duplicate primary placement | CAAC-059 |
| Missing or repeated decision | An XL package omits `decision-policy-owner` from every review file or presents it in both module files | Rejected for the missing or repeated review location | CAAC-060 |
| Decision in the wrong file | `decision-policy-owner` is assigned to Intake and appears only in the Replay review file | Rejected for the file mismatch | CAAC-061 |
| XL shared file | Two XL modules use the same article file | Rejected for the module file boundary | CAAC-043, CAAC-046, CAAC-051 |
| XL article move | `intake.md` has policy Article 1, Intake Article 2 and the open `decision-policy-owner`. `replay.md` has Article 3. Move `article-policy` before the Replay article | Intake becomes Article 1. The policy moves to `replay.md` as Article 2. Replay stays Article 3. Keep `rule-retry-limit`, its obligation, `except-priority`, their references, and all semantic identities and revisions. The decision remains open in `intake.md` | CAAC-009, CAAC-043, CAAC-051, CAAC-052, CAAC-054, CAAC-056, CAAC-059, CAAC-060, CAAC-061 |
| Display reorder | Two S articles contain inherited rules A and B with declared applicability. A precedes B in precedence and execution order. Place B's article before A's | Permitted with new display numbers. Both rules keep their applicability and inheritance. A still precedes B in precedence and execution | CAAC-052, CAAC-055 |
| Changed evidence requirement | An XL move also drops the named evidence requirement for the priority Replay case | Rejected as a presentation-only change | CAAC-054 |

## Review records

Grok re-reviewed the hierarchy and placement rules after the three findings on draft 0.1.3. Hermes reproduced the CLI checks for the corrected hierarchy. On 2026-11-24 the owner accepted the hierarchy, file boundaries and numbering in the exact 0.1.4 document with SHA-256 `7d3a29551daaed3d73460900711571f716b8a03b2fa72cf452112966395b7108`. The implementation status and review record below describe subsequent verification.

| Field | Value |
| --- | --- |
| Contract revision | 0.1.4 |
| Reviewers | Grok (rules and case mapping), Hermes (CLI replay) |
| Cases checked in independent review | CAAC-36 re-review covered the three findings on descendant articles, preserved semantics and unique placement. |
| Findings | Grok confirmed all three corrections within the reviewed scope. |
| Remaining limits | The approval covers the reviewed sizing rules. Runtime execution evidence and acceptance of generated candidates are separate. |
| Reviewer must differ from designer | Yes (CAAC-030) |

## First acceptance exercise

Use [the example](examples/batch-division.md) to challenge these rules. Find a permitted input whose outcome needs an unstated decision.

Record that decision before expanding the method. Add tooling only when a repeated defect establishes a concrete checking requirement.

## Executable prototype

The [local tooling contract](contracts/tooling.md) defines the prototype CLI, declared syntax checks, and Rust example binding.

## See also

- [Tooling contract](contracts/tooling.md): CLI and checker rules.
- [Batch example](examples/batch-division.md): complete contract example.
- [Quickstart](docs/quickstart.md): five-minute guide.
- [Roadmap](docs/ROADMAP.md): version plan.
- [Concept](docs/concept.md): product concept and authority.
