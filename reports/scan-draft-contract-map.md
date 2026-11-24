# Scan-to-draft contract-to-execution audit

## Scope and authority

This first audit covers instruction/context identity, selected input evidence and output guarantees of the current scan-to-draft path. It is not an audit of every CAAC clause or product acceptance. Existing method clauses remain the requirement source. SKILL and TOOLPROTO clauses below are provisional design obligations, not newly accepted requirements. This map does not amend their meaning or turn the current implementation into their definition.

The method contract is draft 0.1.2. Tooling is draft-3. Skill design is skill-draft-3 and tool protocol is tool-protocol-draft-3. Completion controller is controller-draft-5 with no running accepted controller. Implementation authorization does not supply exact owner acceptance of any of these designs.

Inspected Git revision: `c690b6edbd7f227ec60079e48580592eef3b5604`. The following SHA-256 values identify bytes inspected **before** this audit's non-normative status corrections. Read those exact files at the Git revision to reproduce this baseline; do not compare the corrected current files as if they were unchanged. The user's pre-existing modified concept/model/acceptance documents were not changed or used as accepted baseline inputs.

| Source at inspected revision | SHA-256 |
| --- | --- |
| `CONTRACT.md` | `4ba297e75764dcab783e84769c6ea8a649ce488e3c4a61fd2af8174a7aa83050` |
| `contracts/tooling.md` | `6bc42cab1f8b12f03bb2361e82e83c1fa56fe3413f09418b9d943db94c2cb144` |
| `docs/design/llm-skill.md` | `e74e058cc2b3300437382a11bdf0b67746382eaffd3004288fabb4525349e424` |
| `docs/design/tool-protocol.md` | `215197c9a462527acf8f773acdaf577c420b2aeddad606907386035adc6dee87` |
| `docs/design/completion-controller.md` | `cf18a6792a83be7a1a48bfe289aec3364a03224542ea397d6ccbe2223e8a43de` |
| `.agents/skills/caac/SKILL.md` | `27ab54f60201edcc54b1963b2f0cda7f43a2be15893aa916067e037a7b5197b4` |
| `docs/scan-to-draft.md` | `84d51f4691df2c64489cf5ccf134d9fa6e9f0ecad46574dc75f48e69cad34767` |
| `src/candidate.rs` | `f5d10e35aa7dad60a8703059d278246a4509efa11c2902e9a9fa226f9e643abf` |
| `src/cli.rs` | `58aa2858e2955533b12fac6b1ead3ab62c1445909d6f60e6dd0a75b9bf8260e3` |
| `src/harness.rs` | `c7172e48ee1ffdb8e0957c510f28ccad7be18ecb0a82581d0c68a4b4178173a1` |
| `src/source_intake.rs` | `f92c53ad7e5c2c3a731fad133c9c4e38b80d6e4239c2f7777ed3f080460c04b9` |
| `src/contract_collection.rs` | `f893ad108eea711c4a17e517eff74cc042aba65b26de3bbe214e6284aaabc632` |
| `src/review_packet.rs` | `ab8003c18a94956d7e4d241a5f1b7eb3a2f42291df4a754cbfe5205a507fb362` |
| `src/review_findings.rs` | `a845c3450286618b09540d13f0d2565dae58da4961a9e9f8cbecd30a5278642e` |
| `tests/contract_collection.rs` | `715352dcde2cd99932acfad02447cc0adbb2803c8737bdf4882a2468ed119f05` |
| `tests/review_packet.rs` | `ff83b85861566a749525c8234d4ec19e8583b2ee5a7ccdcbbf773ff54ea32fcb` |
| `tests/review_findings.rs` | `6f5ad908fdf3d46dc6b73cb5b4c476c5729ab9c48b1a5c1039541db1ce40eb4a` |
| `tests/candidate_journey.rs` | `0a3995ca6d9f6641beee93c743c5648b774b5ca78fb0d272cf910f23b4df76d2` |

## Status vocabulary

- **Enforced subset:** code rejects a specific invalid input or fixes a specific output state. This is not full clause conformance.
- **Instruction/review:** saved guidance or analyst review exists, but no runtime gate establishes compliance.
- **Not implemented:** the required integrated mechanism is absent. Similar local checks do not discharge it.
- **Open decision:** necessary meaning or authority remains unselected. No threshold or default is invented here.

## Method contract mapping

| Clause | Actual evidence | Status and remaining gap |
| --- | --- | --- |
| CAAC-001, scope/actors/inputs/observables/exclusions before operations | Skill procedure steps 1–6; corrected cold run retained project scope. | Instruction/review. No runtime check proves completeness or ordering of semantic scope decisions. First cold run narrowed scope despite successful CLI checks. |
| CAAC-003, missing/conflicting meaning stays open | Candidate `Decision` records and `propose-decision` keep question/requiredness; `tests/review_findings.rs::checked_citations_propose_existing_decision_then_explicit_apply_stays_pending`. | Enforced subset for submitted decisions. A model can omit a missing decision entirely; tools do not detect all missing meaning. |
| CAAC-004, do not invent behavior to resolve open decisions | `candidate::evaluate_answers` admits proposals, not accepted resolution; skill distinguishes inference. | Enforced subset plus instruction. Proposed text can still contain unsupported meaning; semantic review is necessary. |
| CAAC-028, source identities/revisions | `source_intake::load_evidence`, exact collection/packet hashes; `tests/contract_collection.rs::baseline_only_links_actual_receipt_and_stale_source_is_refused`; stale packet/findings tests. | Enforced subset for explicitly selected inputs. Plain initial entity answers contain unverified source strings; prompt/model/runbook identity is not automatically captured. |
| CAAC-029, observations separate from approved requirements | Source evidence has observation/pending labels; candidate validation only accepts pending authority; unverified collection roles. | Enforced labels plus instruction. Labels do not classify every authored sentence correctly. |
| CAAC-035, owner accepts exact target revision | Candidate validation rejects non-pending authority; provenance always records pending. | Safe refusal, not an implemented owner-acceptance or implementation-admission protocol. |
| CAAC-036, draft while required decisions/acceptance missing | Apply emits `contract_complete:false`; verifier rejects changed completion/authority fields; required decisions persist. | Enforced subset for typed candidate artifacts. Does not prevent an external actor from using or misrepresenting the draft. |
| CAAC-038, review revision/reviewer/cases/findings/limits | Saved itoa reports, artifact hashes and independent review were retained during the task. | Instruction/review. No common runtime review-record schema validates reviewer independence, all cases or exact accepted review scope. |
| CAAC-039, separate language/review/implementation tests | `harness::evaluate_completion` is syntax/case-reference precheck; candidate provenance says behavior not-run; guides state limits. | Enforced local labels plus instruction. No integrated completion controller combines evidence under an accepted policy. |
| CAAC-040, smallest sufficient contract unit | Two reports explain responsibility boundaries and alternatives. | Open sizing judgment. First assignment suggested S in advance; runs are not unbiased evidence for a sizing rule. |
| CAAC-041, estimate scope with XS–XL before writing | Reports record proposals; typed `presentation.size` supports S/M only. | Instruction/review. A supplied size tag does not prove a prior scope estimate or full XS–XL policy. |
| CAAC-044, preserve canonical content when changing size | Existing presentation revision path preserves typed semantic content; integrated journey exercises reframe. | Enforced subset for supported S/M entity model. The two independent itoa drafts have different entities and are not a preservation/resize test. Full legal hierarchy remains unsupported. |
| CAAC-045, no count quota for completeness | Candidate output remains incomplete regardless of count; skill rejects count-only sizing/completion claims. | Safe output state plus instruction, not proof all required obligations were retained. |

## Provisional skill and tool-protocol mapping

These rows describe gaps against the existing designs. They do not silently adopt their entire proposed controller architecture.

| Design clause | Actual evidence | Status and remaining gap |
| --- | --- | --- |
| SKILL-001, validate required context references | Selected baseline/collection/packet revalidation. | Enforced subset. No complete assignment/context package with skill, model, policy, descriptor and source revisions. |
| SKILL-003, preserve identities on resume | Typed interview journals preserve answer prefixes; retained task artifacts exist. | Enforced subset for interview, not whole scan-to-draft checkpoints or model-session resume. |
| SKILL-005, declared repetition/budget/termination policy | Legacy run accepts max-iterations; saved procedure stops on blocked actions. | Not implemented for proposed controller assignments. Legacy pre-review loop is not the design's controller. |
| SKILL-007, identify requested boundary and sources | Project-default scope now explicit in procedure. | Instruction/review. Cold-run scope failure was caught by review, not a mechanical boundary gate. |
| SKILL-009, distinguish proposed/accepted/observed/inferred | Evidence/candidate statuses and solutioning reports. | Enforced container labels plus instruction; no complete typed sentence-level authority separation. |
| SKILL-010, exact acceptance evidence before accepted label | Candidate only admits pending authority. | Safe refusal; acceptance evidence collection and validation are not implemented. |
| SKILL-011, source text is data | Explicit source selection; collection does not follow document links; embedded packet paths cannot authorize reads. | Enforced read-boundary subset. LLM instruction-injection resistance remains host/skill behavior, not proven by JSON tests. |
| SKILL-012, reject source instructions changing authority | Pending-only candidate schema; saved untrusted-input guidance. | Enforced output-state subset; no project runtime interception of all assistant actions. |
| SKILL-027, registered T06 tool interface | Existing direct CLI calls and Rust functions. | Not implemented as a T06 descriptor/dispatcher interface. CLI success is not a T06 receipt. |
| SKILL-029, validated current receipts | Optional evidence-bound apply regenerates exact findings/answers/receipt. | Enforced subset for that local profile only. Initial generated-entity path uses ordinary apply; v3 verify cannot prove historical citation revalidation. |
| SKILL-035, controller owns completion | Candidate never declares complete. | Safe refusal; proposed controller gate absent. |
| SKILL-036, self-report is not reviewer/owner evidence | Reports and procedure state limits. | Instruction/review. No automatic separation/authentication of model, reviewer and owner records. |
| TOOLPROTO-001, separate assessments/interpretation/decisions | Typed states and analyst reports. | Partial as above; no full protocol envelope. |
| TOOLPROTO-002, exact revision/check units per invocation | Candidate hashes and review evidence IDs. | Enforced local identity subset. No general T06 request/check-unit model. |
| TOOLPROTO-003, retain all determining inputs | Exact selected files are checked; generated base/answers retained. | Gap: exact original skill/prompt/model configuration and complete invocation context were not captured uniformly. Later current hashes cannot reconstruct missing historical inputs. |
| TOOLPROTO-005, expose exact descriptors | CLI help and entity schema expose supported shapes. | Not implemented registered version/build/descriptor identity. Help/schema are not equivalent to descriptors. |

## Evidence and non-equivalent experiments

Current tests validate deterministic shapes, state restrictions, snapshot fences and local error cases. In particular, packet tests reject embedded-path authority, findings tests cover stale/tampered inputs, and the integrated journey preserves pending lineage. They do not execute or certify LLM reasoning, sizing correctness, reviewer independence or owner acceptance. The baseline 261-test suite is evidence for its tested scope, not these missing mechanisms.

The first itoa worker was explicitly guided toward an S/single-contract proposal. The independent worker used a later corrected project-scope procedure and selected M. Source files matched, but task prompts, guidance, discovery scope, supplementary evidence and authored entities differed. Exact initial prompt/model/runbook bundles were not uniformly frozen. No causal attribution solely to model randomness, statistical sizing conclusion or identical-input reproducibility claim is supported. See [validation notes](scan-to-draft-validation.md).

## Bounded corrections and next decisions

This increment corrects only stale non-normative capability/inventory statements and the experiment's missing bias disclosure. It changes no CAAC, SKILL or TOOLPROTO obligation, adds no controller and grants no acceptance.

Next priority is to agree the smallest executable profile that maps these existing obligations to a retained assignment/input record and output checks. A proposal should name which inputs determine the run, which evidence is retained, how unknown/partial scope is represented, and which decisions remain human-owned. Do not fabricate old inputs, new numeric thresholds or accepted policy values.

For a later repeated evaluation, hold the source/evidence set, exact skill and task instructions, supported tool build and model configuration fixed, with no suggested size in the task. Separate draft-generation quality from a same-semantic-package presentation comparison. Record every attempt, scope omission, repair, independent review and unresolved decision. Criteria must come from reviewed requirements; a successful command or preferred S/M answer is not the acceptance rule.

## Subsequent bounded implementation: TOOLPROTO-003

The later optional [run-input collection](../docs/contract-collection.md#optional-caller-supplied-run-inputs) implements retention of explicitly submitted UTF-8 run-input files through the existing bounded collector. It separates them from contract documents, preserves exact bytes/hashes and makes changed selections invalidate downstream canonical reconstruction. The original baseline map above remains a historical audit, not silently rewritten evidence.

This does not discharge the complete clause. The caller can omit determining inputs, declared model metadata can be inaccurate, and the tool cannot observe hidden host/model context or prove actual consumption. Required input roles, completeness policy and acceptance remain open. No controller or T06 descriptor registry was added. Tests distinguish identical project source with different task instructions and reject stale context without pretending to evaluate LLM reasoning or sizing quality.

The bounded run-input increment passed 264 repository tests. A separate CLI probe reused unchanged pinned itoa source and document records with synthetic S-leading and neutral task text: context/collection hashes changed, the old packet reconstruction was rejected without output, and declared unknown model metadata stayed unknown. No model was invoked in that probe. An actual collection retained from the earlier pre-change itoa run was regenerated byte-for-byte without run inputs, checking v1 compatibility against an older artifact rather than only new-code self-comparison.

## Subsequent bounded review record: CAAC-038

The [retained-input authoring comparison](neutral-authoring-review.md) uses the existing collector, candidate tools and independent semantic review. It records exact initial revisions, named source-coverage checks, review findings and separate repairs. No new verifier, controller or coverage threshold was introduced. The results distinguish retained input equality from unknown model/host conditions and artifact integrity from semantic review. This is one bounded review record, not general runtime enforcement of the clause.

## Subsequent bounded structure: CAAC-044

The [paragraph presentation exercise](paragraph-presentation.md) adds one explicit article/paragraph level and applies it to a real pending draft. Semantic/object revisions and required questions survive grouping, renumbering and flat restoration. This extends the earlier flat S/M subset without claiming full legal hierarchy, automatic sizing or accepted applicability.
