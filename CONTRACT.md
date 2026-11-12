# Method contract

Status: draft. Scope: authoring and reviewing CAAC contracts. Authority for this draft: this file.

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

A contract needs additional files only when their distinct purpose justifies separate maintenance. A fixed file count is not an acceptance criterion.

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

CAAC-040 | The author must choose the smallest contract unit that captures the required behavior.

CAAC-041 | The author must use T-shirt sizing (XS, S, M, L, XL) to estimate contract scope before writing.

CAAC-042 | A size-XS contract must target one function or expression. A size-XL contract must target one system or platform.

CAAC-043 | The author must split a size-XL contract into multiple files by module.

Acceptance establishes a reviewed implementation target. It does not establish exhaustive correctness or legal enforceability.

## Contract cases

These cases test CAAC's own rules. They are expected results derived from the
contract. They are not records of executed tests.

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

## Review records

No independent review has been completed. This contract remains draft.

| Field | Value |
| --- | --- |
| Contract revision | 0.1.1 |
| Reviewer | None |
| Cases checked | 0 of 8 |
| Findings | None recorded |
| Remaining limits | All cases unverified. No owner acceptance. |
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
