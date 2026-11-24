# Case expectations without stronger hidden obligations

## Requirement and observed failure

CAAC-025 requires supporting documents to reference canonical clauses without adding or changing their obligations. CAAC-032 requires each case's inputs, applicable clauses, expected observables and unresolved decisions. CAAC-030 requires contract-first review before using implementation to infer meaning. These clauses are unchanged.

The [retained-input comparison](neutral-authoring-review.md) exposed a concrete problem after both drafts passed artifact verification. Their proposed fixed-block cases expected no writes outside a designated block. Their cited clauses clearly required allocation bounds, safety and zero preservation, but did not clearly establish the stronger untouched-region obligation. `NOT RUN` did not resolve that traceability gap. Both repaired plans separated the extra footprint condition as a proposal awaiting adoption.

That exercise used source-first coverage inventory to find possible API omissions. It did not demonstrate full CAAC-030 contract-first review. The procedure now names that distinction instead of treating the inventory as a complete semantic-review protocol.

## Contract-first reasoning case

This example is a synthetic, unapproved review aid, not an itoa policy decision.

| Item | Record |
| --- | --- |
| Inputs/preconditions | A buffer of length 32, a block starting at offset 8, and an admitted value requiring zero-padded output. |
| Cited bounds requirement | Writes remain within the allocated buffer. |
| Cited representation requirement | The designated block contains the required zero-padded digits. |
| Existing expectation | All writes remain in the allocation; the designated block has the required digits. |
| Extra expectation | Bytes outside that block retain their original values. |
| Counterexample to entailment | Changing byte 0 to a different value can remain inside the allocation without changing the designated block's digits. |
| Disposition | Untouched surrounding bytes are an additional proposed requirement, not a consequence of those two requirements alone. |
| Unresolved decision | Should the contract require a block-local write footprint, and under which preconditions? |

This regression concerns final-value preservation, not strict write footprint. A same-value write or write-then-restore can preserve surrounding final values while violating a no-outside-writes requirement. Both conditions need their own contractual basis; they are not interchangeable.

This counterexample concerns logical sufficiency of the stated requirements. It is not a claim that itoa writes byte 0 or contains a bug. Source behavior can inform a proposal but cannot supply missing authority.

## Existing implementation boundary

Candidate validation checks the case operation, complete typed inputs/result, operation-owned clause references and exception-target links. It cannot decide whether arbitrary clause prose entails an expected observable. A new linkage-only verifier would not detect this semantic error merely by adding hashes or IDs.

The bounded implementation therefore reuses the existing decision proposal, apply, verification and trace paths. A related required question retains the additional expectation; the original clauses remain unchanged. No new case-plan wire format, controller, entailment engine or mandatory acceptance threshold is introduced. If no existing decision covers the question, the author must explicitly revise the draft rather than use an unrelated decision.

## Regression and independent review

`tests/candidate_journey.rs::additional_case_observable_stays_a_pending_linked_proposal_not_a_clause_rewrite` executes seven actual CLI calls. It checks explicit additional final-value preservation in a required decision, exact clause/package references, unchanged canonical clause bodies, retained required/pending state and rendered proposal text. Invalid references, forged acceptance and stale base identity fail without publishing outputs.

A positive boundary case deliberately submits unsupported stronger prose. It remains structurally valid and does not rewrite clauses. This records the absence of semantic entailment validation rather than pretending to detect the original error automatically. Independent review checked the test and procedure, and corrected a conflation of write footprint with final-value preservation. The full 265-test suite, formatting and strict Clippy passed before commit. No target project behavior was executed. Deciding whether an expectation follows remains a reviewer responsibility.
