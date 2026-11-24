---
title: "CAAC roadmap"
doc_type: "guide"
audience: "contributors"
status: "maintained"
publication: "review-required"
---

# CAAC roadmap

**1.0.0 goal.** A team can review a contract, trace an accepted clause to the code and tests that implement it, and reproduce the evidence from a clean checkout.

| Version | What the user gets | Status |
| --- | --- | --- |
| [0.10.0 Contract drafts](#0100-contract-drafts) | A project becomes a reviewable contract with sources and open questions. | Verified scope complete |
| [0.11.0 Execution evidence](#0110-execution-evidence) | Contract lint and an execution harness enforce selected CAAC rules in CI and report the evidence for each result. | Planned after 0.10.0 |

[Backlog by milestone](#backlog)

## 0.10.0 Contract drafts

Given a project and scope, the authoring assistant produces `candidate.json`, a readable `review.md` and a record of the inputs and decisions. The owner reviews the proposed obligations. [Authoring guide](candidate-authoring.md)

- **G1–G3.** Collect source and existing contracts at exact revisions. Record missing roots, coverage gaps, suspected mismatches and unknowns. A scoped search and an owner's report of absence remain distinct. [Collection guide](contract-collection.md)
- **G4–G5.** Use the project's dependencies and responsibilities to propose one contract or a split, and explain the XS–XL size choice. For APIs, identify which common clauses apply to each route. [Scan-to-draft procedure](scan-to-draft.md)
- **G6–G7.** Render the selected hierarchy, referenced tables and forms, and sentence-level exceptions. Preserve clause IDs, references, cases and open questions when a clause moves or a contract splits.

**Working today.** Selected real projects produced S and M drafts. The function and expression fixtures round-trip between XS and flat S with identical candidate and review bytes on restoration. M and L render optional upper levels in one review. XL permits the L hierarchy inside each module file. S through XL support subitem placements. Candidates can show typed attachments and trace exceptions to an explicit sentence part. [Project exercises](../reports/scan-to-draft-validation.md) · [Typed expression](candidate-authoring.md#typed-expression-example) · [Hierarchy](candidate-authoring.md#m-l-and-nested-xl-hierarchy) · [Attachments and sentence parts](candidate-authoring.md#attachments-and-sentence-parts)

**Done when** an independent reviewer can reproduce a draft from a pinned project, and the XS–XL fixtures preserve its meaning and reject broken references and lossy splits.

**2026-11-24 milestone audit · partially satisfied.** Hermes reported 316 passing tests, bounded pinned-project runs for itoa and itsdangerous, and lossless Echo S → XS → S and shared retry M → XL → M round trips. The exercised broken references and lossy placements failed before publication. A subsequent independent replay reproduced a pending M draft for a pinned Redis GET and optionless SET scope spanning command dispatch, string handling and storage. Source review confirmed the corrected readback and transaction boundaries. Redis behavior remains unexecuted. A corrected function fixture subsequently passed independent source review and CLI replay through XS, S, M, L, XL, a module move and restoration to XS. Its original clause and open decision were preserved, all six negative controls were rejected, and the full suite passed 317 tests. The owner selected typed expression-only XS for 0.10.0 under CAAC-03. Grok verified the schema and Hermes reproduced the CLI results. The expression package survived an XS → flat S → XS round trip with identical candidate and review bytes. All 324 tests passed. The wider CAAC-16 evidence criteria remain open. On November 24 the owner approved the reviewed hierarchy and numbering rules in method revision 0.1.4 (SHA-256 `7d3a29551daaed3d73460900711571f716b8a03b2fa72cf452112966395b7108`) and adopted `caac candidate apply` as the official typed draft entry point. CAAC-31, CAAC-04 and CAAC-33 track shared-obligation rules, attachment policy and language profiles. CAAC-02 and CAAC-09 track further completeness and reference-validation expectations.

### Closure review

On 2026-11-25 the owner approved 0.10.0 for independently reproducible draft generation, meaning-preserving XS–XL resizing and rejection of broken references and lossy splits. The tickets below remain open for the broader requirements.

CAAC-07 is closed. The owner adopted `caac candidate apply` as the typed entry point. On November 24, 47 focused tests passed. Direct apply and verify succeeded at each size from XS through XL. The complete semantic object and reference bindings survived each change of size. Five invalid inputs exited 2 before writing output.

CAAC-01 distinguishes the scope of the product from the acceptance of a generated contract. Echo, shared-retry, Redis and the expression example retain pending authority. They can serve as draft-generation fixtures in this state. Using one as an implementation target requires acceptance of its exact revision.

| Open work | Verified scope and remaining decision |
| --- | --- |
| CAAC-02 and CAAC-16 | The Echo fixture preserved its declared content across all sizes. Separately typed evidence requirements and checks against an external inventory of mandatory obligations are open. |
| CAAC-09 | The tested packages reject broken, duplicate and wrong-type references. Authorization, stale target revisions, ambiguous namespaces and forbidden cycles remain open for the wider resolver. |
| CAAC-04 and CAAC-31 | The CLI carries typed attachments, sentence exceptions and shared clause references. Adoption of the attachment policy and the shared-obligation rules remains open. |
| CAAC-17 | Independent reviewers reproduced the corrected Redis draft. It covers direct execution outside MULTI and retains open integer and pointer assumptions. |

The owner approved the reviewed hierarchy and numbering in method revision 0.1.4. Hermes verified that subsequent edits updated the implementation status and review record. The normative clauses and mapped cases match the approved revision. Expression exceptions remain outside the reviewed XS declaration feature. Runtime execution and evidence freshness are scheduled for 0.11.0.

## 0.11.0 Execution evidence

Implementation starts after the 0.10.0 draft-generation milestone closes. The first increment turns one reviewed rule into a reproducible pass or failure locally and in CI. Contract lint checks the declared clause and case bindings. The harness runs the bound case, compares the observed and expected results, and reports the clause and revision. A reviewer can follow the result back to the clause.

When two readings of a clause predict different results, retain the smallest input that distinguishes them. The intended outcome stays open until resolved. Once resolved, the same input becomes a regression case. A syntax pass alone leaves semantic review pending.

For example, `BATCH-006` requires processing to continue after an item fails. The existing Rust test sends a zero divisor followed by two valid pairs and expects both later results to be `Ok(4)`. The proposed feature would show which code and test revisions produced the observed result. [Clause](../examples/batch-division.md) · [Rust test](../tests/batch.rs)

| Work | First completion check |
| --- | --- |
| E1–E2 · Bind | Link one reviewed clause to the exact code and test revisions, inputs and expected result. |
| E3–E4 · Run and trace | Execute the case. Retain the command, environment, actual result, exit status and artifact digest. Verify lookup in both directions. |
| E5–E6 · Change and rerun | Change a bound dependency. Mark the affected evidence stale, retain the original run and record a fresh result. |
| Local and CI enforcement | Run the same check on a passing fixture, a broken binding and a behavior regression. Report the affected clause and reason. Missing or stale execution evidence cannot pass the check. |

Current candidate traces cover declared relationships and document locations. The experimental comparator identifies fixed and regressed cases in [caller-supplied observations](observation-comparison.md). Implementation bindings, execution records and freshness are planned.

## Backlog

Ticket IDs refer to files in `.dev/tickets/` in a development checkout. Assign individual items to a release during planning. Before writing general-purpose tooling, try a maintained library on a passing case and a failing case.

### Draft-generation follow-up

| Deliverable | Tickets |
| --- | --- |
| Extend the verified resize fixtures to cover separately typed evidence requirements and an external inventory of mandatory obligations. | CAAC-02, CAAC-16 |
| Review the experimental attachment and sentence-part semantics before adoption. | CAAC-04, CAAC-05, CAAC-06, CAAC-09, CAAC-16, CAAC-32 |
| Declare which source languages each analyzer supports. Turn gaps into review questions and identify who owns each shared obligation. | CAAC-10, CAAC-11, CAAC-31, CAAC-33 |
| Turn resolved decisions into a draft, carry reviewed edits back to the schema and independently check the workflow on public projects. | CAAC-01, CAAC-08, CAAC-12, CAAC-17 |

### 0.11.0 planned work

| Deliverable | Tickets |
| --- | --- |
| Enforce one reviewed rule through contract lint and an executable case, with reproducible local and CI outcomes. | CAAC-40 |
| Bind a reviewed clause to its code and test, run one case and recover the clause from the recorded result. | CAAC-13 |
| Change a bound dependency, identify which results are stale and retain both the original run and the rerun. | CAAC-32 |

### Awaiting version assignment

| Item | Observable result | Tickets |
| --- | --- | --- |
| Review and admission | Bind approval to an exact revision, reject unauthorized changes and retry an interrupted action without duplicating its effect. | CAAC-14, CAAC-15 |
| Audit and product claims | Complete the blocked user-role review. Test scaffold limits, link capability claims to results and rerun affected checks after a fix. | CAAC-23, CAAC-28, CAAC-29, CAAC-30 |
| Output recovery | Define what concurrent writers can replace and recover when only part of a multi-file write succeeds. | CAAC-24, CAAC-25 |
| Release tooling | Stage only selected files and pin the CI toolchain. | CAAC-26, CAAC-27 |
| CLI library reuse | Decide whether a shared argument parser preserves the four commands' inputs, diagnostics and outputs. | CAAC-35 |
| PigeonMQ validation | Trace one selected operation from contract to test and reproduce a failure case. | Open proposal |
| CooCoo pilot | After PigeonMQ validation, evaluate one integration against agreed success and failure criteria. Its role in 1.0.0 remains open. | Open proposal |
| Rust MCP adapter | Return the same results and errors as the CLI through the shared core. | Open proposal |
| Coop knowledge base | Resolve an exact contract revision and flag stale or inaccessible copies. | Open proposal |
| Skill distribution | Package the CAAC authoring skill for installation after the core workflow is validated. | Deferred |

**1.0.0 release check.** Independently replay the supported workflow from a clean checkout, including installation, recovery and concurrent writes. Resolve required findings, obtain owner acceptance, check outgoing history for private material and run CI on the release commit.
