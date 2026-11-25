# Bounded paragraph presentation

This report records the paragraph and item increments before the 0.10.0 release. For the shipped hierarchy, module files, attachments and sentence parts, see the [candidate authoring guide](../docs/candidate-authoring.md#presentation-and-boundaries).

## Scope and contract mapping

This increment implements explicit article-to-paragraph grouping in the existing candidate presentation model. It advances CAAC-044 semantic preservation and the requested CAAC-08/16 structural workflow. It does not complete either ticket, automatic XS–XL solutioning or the full legal hierarchy. Existing method clauses and owner decisions are unchanged.

The explicit experimental paragraph profile separates stable IDs from display ordinals. It permits mixed flat and paragraph-containing articles, but each article has exactly one nonempty placement field. Paragraphs have IDs and entity references only. They introduce no normative text, inherited applicability, precedence or implicit exception relation. Numbering resets at each article. At that stage the paragraph profile had no items, subitems, omitted parent levels, attachments, forms or sentence relationships.

## Implementation and compatibility

The existing candidate validator, renderer, revision manifests, apply, compare-review, body revision, presentation revision, verification and trace paths share the extended model. Paragraph packages select explicit presentation/renderer/manifest profiles. Flat packages retain their earlier serialization and renderer path. Returning to a flat-only package requires omitting the paragraph profile and selects the legacy profile deterministically.

Tests cover S direct authoring, M presentation revision, mixed articles, paragraph reordering, exact trace paths, comparison followed by body revision, tampered mappings/profiles and flat restoration. Negative cases reject null/empty/mixed fields, invalid nesting, duplicate presentation IDs, missing/duplicate semantic placement and stale requests. No new dependency or command was added.

A new-binary roundtrip alone is not historical compatibility evidence. Separately, three retained pre-extension flat packages with provenance v3, v4 and v5 were verified with the new binary. All returned exit 0 and remained byte-identical. Verification regenerates the exact prior review bytes and mappings. Historical candidate hashes were:

- v3: `e06e35f8a0246037fa351233f03fb1b1cd34ace13565c8849d97e4e950f1eccc`
- v4: `8593db28b6318a10a16197e9726bb3395c6f2cbe92db5f220c7d0441a781d7d1`
- v5: `91a38cc9388762d663a602318638252ab62907bba9d6f1c9303d17006a3ce95e`

## Independent real-draft exercise

An independent reviewer used the repaired A candidate from the [neutral authoring review](neutral-authoring-review.md). This is deterministic reorganization of an existing pending draft, not a new source scan or LLM generation run. The reviewer grouped only the API article into three explicitly chosen paragraphs, swapped the second and third groups, and restored the original flat presentation. Other articles remained flat, and titles and normative content were unchanged.

Each of the three stages executed presentation revision, verification and trace. All succeeded. Complete semantic JSON, authority, object revisions and all nine required questions remained unchanged. `paragraph-api-b` retained its stable ID while its local ordinal changed from 2 to 3. Flat restoration reproduced the original candidate and Markdown bytes exactly. Original inputs were not overwritten.

| Stage | Candidate SHA-256 | Package revision |
| --- | --- | --- |
| grouped | `9f156adae5f0d4c40e14835791bccdafecaab3aaa4cb9bcc74de7197d05ae458` | `sha256:beeb515111385b5ee549e792db1ebfac1c56720e3b72fba2f8ac43eb62c510dd` |
| swapped | `50b1247eaeb5f3b7cb3d65729791a1fc5b3aac22f39d8e3f90ebd3a43f54a8f9` | `sha256:49c78a1d617c2b78c2cba88ae91bc429a517d4abae35e46b0703ffadd1116456` |
| flat | `1964c260943170edf545a0b53c66b3c9c479841068890c39a805ac73f94b54f9` | `sha256:fccd71cab8272123a3daf2a34c2499abacdc383985a2f9489be19080884e7254` |

Two independent reviewers found no blocking code issue. The real-draft reviewer also checked headings: no new titles, conditions or scope statements were added. This does not establish that every human reader will interpret any grouping correctly; fresh semantic review remains required for each presentation.

## Verification and limits

The full 268-test suite, formatting and strict Clippy passed before the scoped commit. Independent static review covered profile dispatch, strict shape checks, parent receipt validation, comparison slot boundaries, literal escaping and recomputed trace mappings. No target implementation was built or executed by the itoa exercise.

The output remains a pending review document, not a legally accepted paragraph policy or a completed contract. This increment adds a real missing structural capability, but project relationship discovery, robust XS–XL/partition proposals and the remaining hierarchy were still open at the time of this exercise. Execution evidence and transitive freshness belong to the separately planned 0.11 gate and are not supplied by these artifact checks.

## Follow-up: item placement below a paragraph

The next bounded extension adds an ordered item list below an explicit paragraph. It does not split prose or invent new obligations. An item has only a stable presentation ID and nonempty entity placements. Paragraphs use either their old entities field or items, never both. The item profile selects distinct renderer and revision-manifest versions; packages without items retain the old paths. Public Rust callers with exhaustive `ReviewLocation` matches must handle the new `Item` variant.

The concrete Pigeon use is one common-completion paragraph with three groups: CORE-001/002 for original/route checks, CORE-003/006 for atomic completion and temporary-claim rollback, and CORE-004/005 for conflicts and replay. These groups are analyst-selected placement, not procedure order or precedence. The application definition and its HOST references remain semantic content; their applicability is not created by numbering.

Focused regressions exercise S/M item publication, exact location trace, comparison/body revision, mixed placements and restoration to paragraph-only and flat formats. Invalid shapes, wrong profiles, nested items, duplicate presentation IDs and incomplete/duplicate semantic placement fail. Independent static review found no blocking issue; it requested an explicit body-edit assertion, distinct from successful execution alone. Pre-extension retained itoa paragraph and flat artifacts are also verified with the new binary, separately from new-binary roundtrip tests.

The item increment did not cover subitems, arbitrary legal nesting, omitted-parent rules, annexes or sentence semantics. No parser, dependency, command or inheritance engine was added. This presentation increment does not create implementation links, run Pigeon behavior tests or implement the planned 0.11 evidence lifecycle.

The actual Pigeon draft then completed 19 CLI calls: parent verification followed by item grouping, swapping two items and flat restoration, with verification and four traces at each stage. All semantic objects, applicability definitions and eight questions were identical throughout. The stable atomicity/replay item IDs retained their entities while their ordinals changed. Restored candidate SHA-256 is `329b6bc92c9d2a73cf0fa6ce74f084f77890baf14f3f887ee3adb6b3d872605b`; restored review SHA-256 is `ac1ccd06e39cc0cb950fe413332d6bf1c12138e85896459f981920cf860e1366`, both exactly equal to the pre-item source. The restored revision manifest is also equal; the new transition provenance records the roundtrip rather than pretending to be the original receipt.

An independent semantic reviewer checked all three real triples and reran verification successfully. No new condition, applicability, procedure order or precedence was introduced. This is review of the selected placement, not owner acceptance or Pigeon execution evidence.
