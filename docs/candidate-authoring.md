# Experimental candidate authoring

`caac candidate apply` is the official typed draft entry point. It applies explicit, structured answers to a versioned semantic draft and creates a new reviewable candidate. It does not interpret prose, run an interview, execute behavior, approve requirements, or admit implementation. This is a bounded authoring slice, not completion of CAAC-12 or the new harness/controller.

## Supported end-to-end path

The current flow is: inspect the Entity input schema → supply explicit interview answers → verify the generated candidate and document → edit a separate review copy → compare the edited copy against the original generated review to produce a pending proposal → explicitly apply that proposal to a new pending candidate → optionally reorganize its presentation → verify and trace the resulting exact revision. Interview publication, comparison output and each revision use a new output directory; verification, tracing and schema export do not publish packages. Keep earlier artifacts together; a proposal or presentation request for an earlier package cannot be reused against a later one.

The [integrated CLI journey](../tests/candidate_journey.rs) exercises this path using the supplied synthetic, unapproved Echo fixture. From the repository root, run `cargo test --locked --offline --test candidate_journey`. It drives the actual stdin interview and generated Markdown rather than constructing transition results directly. It checks retained inputs and stale-artifact rejection. Owner acceptance and implementation admission remain pending even after an explicit revision; artifact verification establishes consistency, while tracing reports only declared relationships. This is a regression check of the supported authoring flow, not a real contract acceptance, implementation execution or proof of full model coverage. The sections below give each command and its boundaries.

## Run the supplied example

From the repository root, choose an output directory that does not exist. Its parent must already exist.

```sh
cargo run -- candidate apply \
  --base examples/candidate/draft.json \
  --answers examples/candidate/answers.json \
  --output /tmp/caac-candidate-example
```

The example proposes six synthetic entities: a definition, obligation, operation, result-replacement exception, and two cases. It is not a real owner's approved business policy. A required scope question is deliberately skipped and remains unresolved.

The command creates exactly three files under the new directory:

- `candidate.json`: the canonical candidate, including semantic entities, unresolved decisions, separate presentation, and pending authority.
- `review.md`: a deterministic projection from that same validated candidate. Preserve this original; edit a separate copy only for the [comparison workflow](#compare-an-edited-review-without-applying-it). Labeled human-readable fields accompany exact fenced JSON projections; user text is never interpreted as Markdown.
- `provenance.json`: versioned `caac-candidate-provenance-v3` input/output SHA-256 hashes, per-field answer provenance, a derived `revision_manifest`, and the explicit `renderer_profile: "caac-candidate-legal-review-v1"`. This file is written last and marks successful publication, not contract completion. Its claims are local records, not authenticated evidence or owner signatures.

Input files and existing outputs are never overwritten. Validation errors occur before output creation. Publication I/O failures may leave a partial directory without the final provenance file. Do not treat that directory as a successful result or overwrite it on retry. Select a new output path after inspecting the error.

## Explicit input contract

The public `caac::candidate::Candidate` type is parsed with Serde. `validate` and `render` use the same validator as the CLI. The [draft example](../examples/candidate/draft.json) and [answer example](../examples/candidate/answers.json) are required companion inputs: this overview is not an exhaustive field-by-field schema reference. They show the admitted fields. Unknown fields, duplicate object fields and duplicate input-map keys are rejected.

Draft and answer inputs must be regular files of at most 1 MiB each; final-component symlinks, FIFOs and other special files are refused. On Unix, the shared input reader uses nonblocking/no-follow open flags. This bounds input size and avoids waiting for a FIFO writer; it is not a filesystem I/O deadline.

`expected_base` is the lowercase SHA-256 of the exact draft file bytes, including whitespace. Calculate it with `shasum -a 256 DRAFT`, then copy its first field into the answer file. Do not hash parsed/reformatted JSON. A mismatch fails without publishing. Both input byte sequences are reread immediately before publication and compared with their exact initial bytes; any change fails publication. This detects intervening edits, but is not a lock against simultaneous writers or filesystem alias replacement.

Each answer has a unique stable `answer_id` and nonblank `source`. A source string is unverified provenance, never authority. The actions are:

- `propose-entity`: an explicit `target` with `kind` and `id`, plus the complete typed `value`. Target and value must agree. It adds a new entity; replacements are deliberately rejected, including an identical entity with an existing ID.
- `propose-decision`: an existing `decision_id` and `value` containing nonblank `text` plus nonempty `entity_refs` to actual candidate entities. The question and required flag remain unchanged. The proposal remains pending within `unresolved`; it is not accepted or automatically resolved. Existing proposals cannot be replaced through this action.
- `skip`: an existing `decision_id`. The question and required flag remain unchanged. Duplicate targets and nonexistent skips are rejected.

For another iteration, copy the generated `candidate.json` to a new draft file. Preserve existing entities, add explicit presentation references for the new entities, and prepare a new answer file containing only new entity proposals, decision proposals, or skips. Calculate `expected_base` from that new draft. The tool does not edit the prior candidate or merge an old answer file. Changes made directly while preparing the new draft belong to that new input snapshot; the provenance manifest records only the current answer application, not an authenticated history or a diff of those manual edits.

This slice does not resolve or delete existing decisions. A candidate can be structurally valid while required decisions remain open. No `contract_complete` or acceptance flag can be set through an answer. The [project/domain proposal exercise](../reports/project-domain-proposals.md) shows concrete ordinary-use proposals retained under required questions and a separately reviewed project-root relationship; neither establishes adopted applicability.

## Entity input schema

From the repository root, run `cargo run --locked --offline -- candidate schema` to print a JSON Schema for one complete Entity answer. You can also run `caac candidate schema` with the built CLI. The schema covers the entity kinds below. Use it with a standalone Entity JSON file in an editor that supports JSON Schema draft 2020-12. When the [typed interview](#typed-non-auto-interview) requests an entity, supply its complete JSON on one stdin line. The entity accepts only its declared fields.

The schema is generated from the existing Rust/Serde types with [Schemars](https://docs.rs/schemars/1.2.3/schemars/), rather than maintained as a second handwritten field specification. It exposes tags, required fields, nested shapes, unknown-field restrictions and the signed 32-bit integer range. Regenerate it with the CAAC version whose inputs you are preparing; a saved schema is not a version-independent contract.

This is input-shape assistance, not candidate validation or acceptance. JSON Schema cannot detect duplicate keys in raw JSON after a parser has discarded them. Its integer type is mathematical: a validator may accept `1.0` or `1e0`, while the runtime integer parser rejects those floating-point spellings. A schema-valid entity can still have blank text, invalid or mismatched IDs, unresolved references, inconsistent operation/case/exception types or an ineligible target. Exact-revision checks, pending authority and package-wide rules still belong to the existing parser and runtime validator. Neither schema generation nor schema validation runs a case, approves a requirement or establishes completion.

## Admitted semantic profile

The version is `caac-candidate-v1`. Entity IDs and package-local references are explicit and stable, independent of display numbering. All references resolve inside this candidate; external references and inherited contracts are unsupported.

- `definition`: nonblank text.
- `clause`: actor, modality (`obligation`, `prohibition`, or `permission`), text, typed definition references, and optional annex and form references.
- `operation`: unique name, a named input map, output type, one or more clause references, and explicit exclusions.
- `expression`: nonblank authored `text`, a named operand type map in `inputs`, an `output` type, and one or more `clause_refs`.
- `exception`: a clause target, optional sentence part, `input-equals` applicability and a `replace-result` effect. The input field and replacement result must match every operation sharing the target clause.
- `case`: exactly one of `operation` or `expression`, with a non-null ID of the corresponding kind. Supply every declared input with its type, a typed expected result, and one or more of the target's clause references. For an operation, `exception_refs` lists any exceptions targeting those clauses. For an expression, supply an empty `exception_refs` list.
- `annex`: status, title, ordered column names and rows of typed values.
- `form`: status, title and ordered fields with an ID, label, value type and required flag.
- `sentence-part`: an explicit first or second part of a named parent clause.

Admitted value types are `i32`, `bool`, and `string`. Validation checks the declared structure and references. Cases record expected behavior. Exception review remains pending.

The manifest's `semantic_sha256` is a serialized semantic-content hash: SHA-256 of the module's deterministic pretty-JSON serialization of the complete `semantic` object (`namespace`, `entities`, and `unresolved`). It includes unresolved questions and every entity field; it excludes `presentation`, `authority`, and provenance. This checksum is distinct from the domain-separated revision identifiers below. It is a versioned implementation encoding, not a claim of universal JSON canonicalization or semantic equivalence. Presentation changes preserve semantic identity; semantic changes produce a different semantic hash. Old evidence is not imported or declared current.

## Presentation and boundaries

Explicit `XS` requires exactly one target entity across all operations and expressions. Targets count even without a case. Set `presentation.size` to `XS` and supply one flat article with direct placements. Omit `presentation.profile`. Place any annexes or forms in the attachment list. Trace uses the `unnumbered-article` location and stable node ID. Operation-only XS uses renderer and revision format v4, or v8 with attachments or sentence parts.

Explicit `S` contains articles. Explicit `M` can group articles under chapters, sections and subsections. Each upper node and article has a stable ID and title. Each semantic entity must appear exactly once. Operations can reference the same clause. Missing, repeated or wrongly typed references fail before publication.

The candidate format is separate from the existing narrower `ContractSchema` conversion format. Candidate input is rejected by legacy `md`/`gen`; legacy input is rejected by `candidate apply`. There is no automatic conversion, equivalence claim, reverse parsing, or dual-source synchronization. `candidate.json` alone is the source for its own derived view. The old Python demonstration is not called by this command and is not its wire format.

The existing harness/controller and conceptual acceptance gate are unchanged. Both `authority.owner_acceptance` and `authority.implementation_admission` must be `pending`; supplied acceptance claims are rejected. Unsupported capabilities remain gaps, not successful checks.

### Typed expression example

An expression package can use XS or flat S. In flat S, each article places entities directly and the package omits `presentation.profile`. Flat S can contain both operations and expressions. Expression packages use renderer `caac-candidate-legal-review-v9` and manifest `caac-candidate-revisions-v9`. Operation-only packages retain their versioned artifacts.

Run the [less-than draft](../examples/candidate/expression-draft.json) with its [bound answers](../examples/candidate/expression-answers.json).

```sh
cargo run --locked -- candidate apply \
  --base examples/candidate/expression-draft.json \
  --answers examples/candidate/expression-answers.json \
  --output /tmp/caac-less-than
cargo run --locked -- candidate verify \
  --candidate /tmp/caac-less-than/candidate.json \
  --provenance /tmp/caac-less-than/provenance.json \
  --review /tmp/caac-less-than/review.md \
  --trace expression-less-than
```

The review quotes `left < right` as authored text. It shows the two `i32` operand types, the `bool` result type, the rule ID and the case with inputs `left=2`, `right=3` and expected `true`. The CLI checks the declared types and bindings. Provenance records `behavior_evidence: "not-run"`. The expression has an incoming `expression` edge from the case and an outgoing `clause_refs` edge to the rule. Rust callers matching every variant of `ObjectKind` or `ReferenceField` can handle the new `Expression` variant.

Use [presentation revision](#explicit-presentation-revision) to change this draft to flat S. It receives Article 1 and keeps its semantic and object revisions. Reversing the size change restores the original candidate, review and manifest. Each transition issues a new receipt and requires fresh review.

### XL module files

Set `presentation.size` to `XL`. Each top-level module has `kind: module`, `id`, `title`, `file`, and `decisions`. Supply one nonempty `articles` or `children` list. Supply 2 to 64 modules and at least one article in each. Use a distinct lowercase ASCII Markdown basename for each module. Reserve `review.md` for the index. Use an empty `decisions` list when the module owns no unresolved decisions.

Place each semantic entity once across the module articles. Assign each unresolved decision to one module. References resolve against the package, including when the referenced clause is in a different module file. Article numbers follow module order across the package.

`candidate apply` writes `candidate.json`, the `review.md` index, the module files, and `provenance.json`. Keep all module files beside the index. `candidate verify` checks the declared module files against the candidate. `candidate verify --trace` reports the module file and display location. An I/O failure during publication can leave a partial directory. The last file, `provenance.json`, marks completion.

Use `candidate revise --presentation` to move an article between modules. The new revision preserves its semantic revision and object IDs. Apply a fresh source draft to change obligations. Markdown edit comparison on an XL index returns an error.

### M, L and nested XL hierarchy

M and L each produce one `review.md`. The M upper levels are `chapter`, `section`, and `subsection` in that order. L also permits `part` above those levels. Use the L hierarchy inside an XL module's `children` list.

Each upper node has an `id`, a `title`, and a nonempty `children` list. A branch can skip upper levels. Each branch ends at an article. A chapter can also use `articles` to list articles directly.

Number upper nodes among siblings of the same kind. Number articles across the whole presentation, including across XL files. A trace records the upper ancestor IDs and ordinals when the article has them. Deep review headings stop at Markdown level six. Trace paths preserve the logical hierarchy.

Use `candidate compare-review` to compare supported field edits in a single M or L review.

M with direct articles or upper nodes that use `children` selects renderer and revision format v7. M with only article-list chapters keeps its previous format unless it uses subitems. L and nested XL without subitems use v6. Article-only XL without subitems keeps v5.

### Attachments and sentence parts

Place each annex or form once in `presentation.attachments` for a single-file review. In XL, place each in one module's `attachments` list. The review puts these entities in a separate Attachments section. Numbering starts at 1 in each file. Trace records the attachment ID, file and number. Clause references can cross module boundaries.

Use `annex_refs` and `form_refs` on clauses. Each reference contains `id` and `usage`. Set the usage to the target's `status`, either `binding` or `illustrative`. Supply distinct column names and at least one row in each annex. Each row has one value per column. Supply distinct field IDs in each form. A field declares `id`, `label`, `value_type` and `required`.

Place a `sentence-part` in the main hierarchy. Set `clause` to the parent ID and `portion` to `first` or `second`. Its text must match a proper prefix or suffix of the parent text, respectively. A clause can have one of each, with no overlap. The author supplies the text boundaries. An exception can set `sentence` to one of its target clause's parts. It retains the parent `target` and the same typed applicability and effect.

Candidates that use these entities or references select renderer and revision format v8, or v9 if they also contain an expression. Trace resolves each reference to an exact object revision and reports `attachment_usage` on attachment references. To change a table cell or a form field, edit the typed draft and run `candidate apply` with answers bound to that draft.


## Explicit paragraph grouping

Set `presentation.profile` to `caac-paragraph-presentation-v1` when at least one article uses paragraphs and no paragraph uses items. A package can mix flat articles and articles with paragraphs. Omit the profile for a flat-only package.

Each article has exactly one nonempty `entities` or `paragraphs` field, never both. Under the paragraph-only profile, a paragraph contains only a stable `id` and a nonempty ordered `entities` list. IDs are unique across all presentation nodes. Nulls, empty groups, unknown fields and nested paragraphs are rejected. Every semantic entity still appears exactly once in the package.

```json
{
  "size": "S",
  "profile": "caac-paragraph-presentation-v1",
  "nodes": [{
    "kind": "article",
    "id": "article-public-api",
    "title": "Public API",
    "paragraphs": [
      {"id": "paragraph-input", "entities": ["TERM-INPUT", "RULE-001"]},
      {"id": "paragraph-output", "entities": ["RULE-002"]}
    ]
  }]
}
```

The referenced semantic entities must already exist. Paragraph numbering starts at 1 in each article. Stable IDs survive numbering changes. Groups record presentation only.

Use the existing `candidate revise --presentation` path to regroup an exact candidate. All semantic content, unresolved questions and object revisions must remain unchanged. Package/presentation revisions and exact location mappings change. A paragraph location records the paragraph ID/ordinal, article ID/ordinal and optional chapter ID. Trace reports the recomputed location, not a user-supplied address.

S and legacy M paragraph-only packages use renderer and revision format v2. The size-specific formats above apply to the other profiles. Remove the paragraph profile when you remove all paragraph groups. See the [bounded paragraph exercise](../reports/paragraph-presentation.md).

## Explicit item placement below paragraphs

Set `presentation.profile` to `caac-item-presentation-v1` when at least one paragraph uses items and no item uses subitems. Each paragraph has exactly one nonempty `entities` or `items` list. In the item profile, each item has a stable `id` and nonempty `entities` list.

For example, a paragraph may contain `{"id":"paragraph-core","items":[{"id":"item-validation","entities":["CORE-001","CORE-002"]},{"id":"item-commit","entities":["CORE-003"]}]}`. This is a placement fragment, not a complete candidate; all referenced entities must exist. Mixed flat articles, flat paragraphs and item-containing paragraphs are allowed under the item profile.

Item numbers restart at 1 in each paragraph; stable IDs survive moves. Order is display order, not execution sequence, applicability or precedence. A trace location retains the item, paragraph, article and optional chapter IDs plus their ordinals. A presentation revision preserves the full semantic object and pending authority. Meaning changes must use a separate semantic proposal.

S and legacy M item packages use renderer and revision format v3. The size-specific formats above apply to the other profiles. The public Rust `ReviewLocation` enum provides the `Item` variant.

### Subitems

Set `presentation.profile` to `caac-subitem-presentation-v1` when at least one item uses subitems. Each item has exactly one nonempty `entities` or `subitems` list. Each subitem has a stable `id` and a nonempty ordered `entities` list. Nulls, empty lists, mixed fields, duplicate presentation IDs and missing or repeated entity placements fail validation.

Subitem numbers start at 1 in each item. Trace reports the `subitem` location with its item, paragraph, and article IDs and ordinals. The renderer and revision manifest use v7. Rust callers with exhaustive `ReviewLocation` matches must handle the `Subitem` variant.

## Typed non-auto interview

`candidate interview` is a stdin adapter for the same candidate `apply`, validation and rendering path. It does not use or expand the legacy interview/harness controller. The base's explicit presentation determines the missing entity IDs and question order. Existing semantic entities are not re-entered. Decisions without a proposal follow those entities in their existing order.

```sh
cargo run -- candidate interview \
  --base examples/candidate/draft.json \
  --answers /tmp/echo-answers.json \
  --output /tmp/echo-candidate \
  < examples/candidate/interview-input.jsonl
```

The [scripted stdin example](../examples/candidate/interview-input.jsonl) supplies complete typed Entity JSON values one per line, then explicitly skips the required scope question. It is synthetic input, not inferred business requirements. For a decision, enter either `skip` or one JSON object such as `{"text":"Proposed scope for review","entity_refs":["operation-echo"]}`. Proposals preserve the question and required flag and remain unresolved pending review. Type `cancel` to stop. The command never silently skips a missing answer or interprets free prose.

For `candidate interview`, `--answers` names the noncanonical journal path, not a prefilled answer file. The journal path must not exist for a new interview. Invalid profiles, duplicate presentation IDs, non-pending authority, overlapping base/journal/output paths and existing output paths are refused before prompts or journal creation. The base is frozen by exact bytes. New entity references can be incomplete while answers are being collected; full reference/type validation occurs before publication, without placeholder entities.

Each shape-valid typed answer that matches the current question is saved before asking the next question. Field/reference consistency and semantic-profile validation run only at final apply, so a saved prefix is not a fully validated semantic candidate. For example, a string with blank clause text has the correct JSON shape but is refused at final validation. Each line is limited to 64 KiB and the complete journal to 1 MiB. EOF, cancellation, malformed JSON, a mismatched entity ID or an entity `skip` ends with a nonzero result and preserves previously saved answers. An entity skip does not advance past that missing entity. A decision skip is recorded but leaves the decision unresolved. Final reference/type validation or publication failure also preserves already-journaled answers; a malformed or rejected input line itself is not saved. Collected answers do not mean a candidate was published. The command exits zero only after all three output files were created successfully; this still is not contract completion. Required decisions may remain unresolved while a review-only candidate is published; that does not permit contract completion or acceptance.

Resume explicitly against the same base. Remove the scripted stdin redirection for this interactive example so you can review the saved answers and enter `reuse` yourself:

```sh
cargo run -- candidate interview \
  --base examples/candidate/draft.json \
  --answers /tmp/echo-answers.json \
  --output /tmp/echo-candidate \
  --resume
```

The journal has `format: "caac-interview-answers-v1"`, the exact `expected_base`, and a typed ordered answer prefix. It is noncanonical input data that `candidate apply` can also consume, not an accepted contract. Resume checks its format, exact base, field types, target sequence and answer IDs, then displays its values through terminal-safe JSON quoting. Enter `reuse` to select the displayed saved answers explicitly before continuing, or `cancel`. `reuse` is not authentication of the journal, an owner signature, or approval of requirements/implementation. Never infer authority from an answer's source string.

After reuse, the displayed journal bytes remain the expected snapshot. Outside edits are refused rather than overwritten or silently consumed by final apply. The base is rechecked before each save and publication. These are conflict checks, not concurrent-writer locks. A valid-shaped malicious journal cannot be proven authentic by this file format; review what is displayed before choosing reuse. Authority fields cannot be changed through journal answers.

If a final semantic check fails, the journal retains the offending explicit input for inspection. A deliberate manual correction remains untrusted input and requires another explicit `--resume`/`reuse` selection. If output partially exists or another actor created it, inspect it and select a fresh output directory; do not overwrite it. A changed base cannot resume the old journal: prepare a new draft/journal and deliberately reconsider the answers. This workflow does not silently rebase or transfer acceptance/evidence.


## Revision identities and exact derived references

`caac-candidate-v1` remains the experimental canonical input format. The additive `caac-candidate-revisions-v1` manifest lives inside authoring provenance v3, entity-revision provenance v4 or presentation-revision provenance v5 and is derived from a validated candidate by `caac::candidate::revisions`. It is not another source of requirements. `verify_revisions` compares a supplied typed manifest against that candidate without repairing stale values.

The manifest separates these identities:

- `namespace` and stable object `id` identify an entity or unresolved decision. Display numbering and file locations are separate.
- An object's `revision` hashes its direct typed body and namespace. The body includes its own ID, its entity kind where applicable, and its complete typed payload. A change to its text, values or reference IDs changes that revision. Decisions include their question, required flag and optional pending proposal. This is not a transitive effective-meaning hash.
- `semantic_revision` hashes the candidate's entire `semantic` section, including namespace, all entities and unresolved decisions. It excludes presentation and authority.
- `presentation_revision` hashes the complete presentation tree, including size, node IDs, titles, ordering and entity placements. It is interpreted within the enclosing package, not as a globally named document.
- `package_revision` hashes the complete typed candidate, including format, semantic content, presentation and pending authority. Moving an unchanged clause can therefore preserve its object and semantic revisions while changing the package and presentation revisions.

Revision IDs use `sha256:` plus a lowercase digest. The versioned encoding is `caac-candidate-v1/serde-json-pretty-v1`. Each digest covers the UTF-8 domain name, one zero byte, then deterministic pretty-JSON bytes without a trailing newline. Domains are `caac.package.v1`, `caac.semantic.v1`, `caac.presentation.v1`, `caac.entity.v1`, and `caac.decision.v1`. Entity/decision hash input is an object containing `namespace` and `body`; the other inputs are their complete typed objects. This paragraph describes the encoding; it is not a standalone interoperable serialization specification. The normative implementation and regression cases are [the revision module](../src/candidate/revisions.rs) and [revision tests](../tests/candidate_revisions.rs). This encoding is tied to that implementation version, not RFC 8785, arbitrary JSON equivalence, or a semantic proof.

Typed map keys have deterministic order. JSON whitespace and source object-key order do not change typed revision IDs. Array order is retained: reordering semantic objects or reference lists changes the semantic/package revision even if a human considers the ordering immaterial. String content, including whitespace within strings, is not normalized. Raw `candidate_sha256`, `review_sha256`, and `semantic_sha256` checksums retain their separately documented byte/serialization meanings; they are not the domain-separated revision IDs.

Every derived relationship records its source and target as exact addresses containing namespace, full package revision, kind, stable ID and direct object revision. The relationship field and zero-based index distinguish ordered list references; scalar references use a null index. Decision-proposal references are included. Presentation mappings separately identify the exact target and its generated `review.md` article node, chapter if present, display number, or unresolved-decision array index. Node and chapter IDs are presentation IDs, not Markdown anchors or source byte spans. The review includes the same manifest's identities and mappings.

A referenced definition may change while a referring clause's direct body hash remains unchanged. The semantic/package revision and exact target bindings still change. Likewise, changing an exception does not prove the unchanged target clause has unchanged effective meaning. An unchanged object hash never grants evidence reuse or owner acceptance. Full package references avoid silently treating the old dependency snapshot as current. No recursive package-hash dependency is introduced because the derived manifest is not part of the canonical candidate being hashed.

Apply/interview authoring inputs use package-local IDs rather than per-target expected revisions. The [explicit revision workflow](#explicit-pending-revision-with-exact-preconditions) enforces the complete expected parent snapshot and each existing target revision, and records a bounded one-step transition. A general exact-reference authoring language, cross-package resolver, retirement policy and evidence-impact engine remain unsupported; CAAC-05 is not closed.

## Verify the three generated artifacts

```sh
cargo run -- candidate verify \
  --candidate /tmp/echo-candidate/candidate.json \
  --provenance /tmp/echo-candidate/provenance.json \
  --review /tmp/echo-candidate/review.md
```

The command is read-only. It validates the candidate, requires supported authoring provenance v3, entity-revision provenance v4 or presentation-revision provenance v5, the supported renderer profile and manifest version/encoding, compares the complete typed revision manifest (including arrays in exact generated order, not as sets), checks raw candidate/review checksums and the semantic serialization checksum, and requires the review bytes to equal the deterministic projection of that candidate. Missing or additional manifest records, duplicate records, wrong kinds, mismatched namespaces, stale revisions, wrong relation indices and changed location mappings fail. Unknown fields and duplicate JSON object fields are refused. Nullable `index` and `chapter_id` keys admit omission as equivalent to explicit null; other required structure must be present. Inputs are reread before success and compared with their exact initial bytes; any change fails verification. This is not a concurrent-writer lock.

Changing a review and merely updating its checksum does not pass: the canonical candidate must generate that exact review. Reformatting a candidate can preserve every typed revision but changes its raw checksum, so the old provenance fails. Regenerate the artifacts through the supported authoring workflow rather than manually repairing checksums; matching a checksum alone does not establish the remaining consistency checks. Unsupported versions fail without a legacy fallback. Old provenance v1/v2 artifacts are not silently migrated or rewritten.

Verification establishes unsigned artifact consistency only. The original base/answer files are not supplied, so their hashes and field-change history are not authenticated or proven truthful. Co-edited, consistently regenerated unsigned artifacts can pass. The command does not authenticate authorship, establish permission, approve requirements, run behavior cases, transfer evidence, or verify a historical acceptance record. All supported authority and completion fields must remain pending/false; v3 field-change authority remains unverified proposal data. V4 and v5 receipt-specific checks and historical limits are described [below](#revision-provenance-v4).

Publication and verification use coherent individual artifact limits (1 MiB = 1,048,576 bytes): generated candidate at most 1 MiB, embedded revision manifest at most 16 MiB measured by typed reserialization, and review/provenance at most 32 MiB each. Oversized generated artifacts fail before creating the output directory. Verify uses the same regular-file and size boundaries and does not open referenced external files or paths from the manifest.


## Bounded legal-style review profile

`caac-candidate-legal-review-v1` (flat) and `caac-candidate-legal-review-v2` (explicit paragraphs) are versioned renderers of the experimental candidate, not a finished legal contract or completion of CAAC-08. The public `render` API, apply/interview publication and read-only verifier use this same renderer. Provenance v3, v4 and v5 record the profile; earlier provenance v1/v2 and unknown profiles are refused rather than silently interpreting an earlier review through new rules. No automatic migration is supplied. The verifier refuses old provenance; an existing candidate can be explicitly used as a new draft to regenerate fresh output through the [apply workflow](#explicit-input-contract). Candidate format and semantic revision encoding remain unchanged; paragraph packages use the explicit v2 location manifest described above.

The review shows the supplied presentation titles, IDs and numbering. Each [admitted entity](#admitted-semantic-profile) shows its stable ID and typed body fields as labeled JSON literals. Clause actor, modality and authored text appear in separate fields. Shared clauses appear once at their designated location. Operations retain references to those clauses.

Every entity also has a complete exact JSON projection from the same source. Here exact means preservation of typed values, not original input formatting or object-key order. These repeated values are not independent obligations or a second editable source. Full article/chapter presentation JSON and decisions remain available. All decisions remain unresolved, with optional proposals labeled pending; mapping uses zero-based decision-list indexes. Proposals never resolve a decision automatically. Package metadata and the exact revision manifest bind this view to the source package. Mapping identifies article/chapter presentation IDs, display numbering and decision indexes, not byte spans or sentence anchors.

Dynamic text is JSON-quoted inside centrally fenced Markdown code literals whose backtick delimiters are longer than every backtick run in the data. Escaped data and delimiters are tested with Comrak, not promised compatible with every Markdown viewer. Ordinary Unicode text, including Korean, remains readable. Embedded string newlines and control/directional-formatting characters are escaped; Markdown punctuation and HTML are displayed as data, not links or executable markup. Strings retain quotes so `false`, `0`, `"0"`, `""` and `" "` remain distinguishable. Some viewers may visually collapse spaces in inline code; the exact fenced JSON remains available for inspection. Exact JSON projections decode to the same typed values, including escaped characters. This is field-preserving display, not universal legal-language interpretation or semantic equivalence.

[Artifact verification](#verify-the-three-generated-artifacts) compares exact review bytes. Regenerate from a typed candidate, or compare a separate edited copy using the workflow below. The wider CAAC-08 document editing requirements remain open.

## Compare an edited review without applying it

From the repository root, first generate the three artifacts with the [apply example](#run-the-supplied-example) or [typed interview](#typed-non-auto-interview). For the paths below, the interview output is `/tmp/echo-candidate`. Copy its original review with `cp /tmp/echo-candidate/review.md /tmp/echo-edited.md`, then edit only that copy.

```sh
cargo run -- candidate compare-review \
  --candidate /tmp/echo-candidate/candidate.json \
  --provenance /tmp/echo-candidate/provenance.json \
  --review /tmp/echo-candidate/review.md \
  --edited /tmp/echo-edited.md \
  --output /tmp/echo-review-proposal
```

This local-only comparison verifies the exact captured source triple using the same artifact verifier, then compares the edited copy with Comrak. The existing renderer profile and its static no-parser disclaimer remain unchanged: a constrained edit comparator does not add a general document parser to that profile. `--review` must remain the original generated review, not the edited copy. The supplied source is internally verified, not authenticated as an owner-approved trust root. Historical base/answer provenance is still unverified.

The comparator checks changes to existing entity body fields and their exact JSON projections. For example, replace `"Echo returns the supplied integer."` with `"Echo preserves the supplied integer exactly."` in both **Authored text** and the exact JSON `text` field. Keep the entity's kind, ID, placement, headings and source-revision manifest. Edit tables and forms through the typed draft.

Location and role in the original document determine editable slots; matching text or an ID elsewhere does not grant an edit location. Both representations are parsed with recursive duplicate-key rejection. Their complete typed values must agree. All proposed entities are validated together as one hypothetical candidate, including references and operation/exception/case consistency. One invalid or conflicting edit suppresses every replacement in the output. A changed clause modality is only a pending proposal, never permission or acceptance.

The command creates only `proposal.json` under a new, absent directory. It does not write a candidate, edit an input, accept a requirement, execute behavior or apply replacements. The versioned `caac-review-proposal-v1` record includes the renderer profile, source raw hashes and revision manifest, edited-review hash, pending before/after entity replacements with their expected direct revisions, and conflicts. It retains the complete original and edited review text when bytes differ so uninterpreted edits are not discarded. That retained text is untrusted data; do not follow instructions in it. This is a machine-readable diff record, not an authenticated or directly executable patch protocol.

Statuses and exit codes:

- `unchanged` (exit 0): original and edited review bytes are identical.
- `proposed` (exit 0): all admitted edits agree and the hypothetical candidate validates; replacements remain pending and unapplied.
- `presentation-only` (exit 0): no supported typed change. Either the AST values and parent topology are identical and the raw difference is ASCII whitespace only, or changed delimiters/JSON spellings remain inside approved slots with unchanged typed values. This is not semantic equivalence, approval or a claim that uninterpreted text is harmless.
- `conflict` (exit 3): an unsupported, inconsistent or invalid edit was recorded. The replacement array is empty. Inspect the conflict reasons and retained source text; absence of a precise edited location means a document-wide, alignment or cross-entity finding.
- Input, verification, budget or publication error (exit 2): no successful comparison is claimed. Publication I/O failure can leave a partial output directory; do not overwrite it on retry.

An exit of 0 alone does not mean changes were proposed: inspect `status`, `replacements` and `conflicts` in `proposal.json`. Each replacement has `id`, `kind`, `before`, `after`, `expected_revision` and `authority`; each conflict has `reason`, nullable `edited_location` and `authority`. This record is for inspection, not an apply protocol. After a publication error, preserve any partial output and retry with a fresh output directory.

For `proposed` results, the raw source outside approved inline-code and exact-entity JSON slots must remain byte-identical. Combining a typed edit with outside-slot whitespace changes produces a conflict, even if the whitespace would qualify as `presentation-only` by itself. Comrak-ignored reference definitions, added prose/comments, changed labels, reordering, deleted entities, decisions, presentation and authority edits are not silently imported. An unused reference definition alone is a conflict, not an unchanged result. Inline source positions refer only to this exact edited snapshot; they are not stable sentence anchors. There is no general edited-contract parser, arbitrary Markdown import, insertion/deletion/move workflow, reverse synchronization or full CAAC-08 closure.

Although rendering and artifact verification admit reviews up to 32 MiB, this narrower command admits each original/edited review up to 4 MiB, the candidate up to 1 MiB and provenance up to 32 MiB. It rejects a comparison AST above 100,000 nodes after parsing the bounded input. Generated proposal JSON is limited to 32 MiB before output creation. These are finite input/output limits, not a parser-time deadline. Inputs must be regular nonsymlink files and are reread against their exact captured bytes before publication. Output cannot overlap any input path and must not exist. These checks do not provide a concurrent-writer lock or authenticate historical provenance.


## Explicit pending revision with exact preconditions

Receiving or producing a proposal does not apply it. First run [candidate compare-review](#compare-an-edited-review-without-applying-it) and inspect its `proposal.json`: only status `proposed`, no conflicts and a nonempty replacement array are eligible. The [admitted entity profile](#admitted-semantic-profile) and comparator define the editable fields; [revision identities](#revision-identities-and-exact-derived-references) define the hashes. An explicit invocation can then create a new pending candidate:

```sh
cargo run -- candidate revise \
  --candidate /tmp/echo-candidate/candidate.json \
  --provenance /tmp/echo-candidate/provenance.json \
  --review /tmp/echo-candidate/review.md \
  --proposal /tmp/echo-review-proposal/proposal.json \
  --output /tmp/echo-revised-candidate
```

The output directory must be new. The command creates `candidate.json`, `review.md` and `provenance.json`, with provenance written last. Source artifacts and the proposal remain unchanged. This is an explicit transition between pending candidates, not owner acceptance, implementation admission, automatic application on message arrival or permission to execute the contract.

The command verifies the captured source triple, recursively rejects duplicate proposal keys, and recomputes the entire comparison using the proposal's retained edited-review snapshot. The supplied proposal must equal that recomputed record, including all source hashes, the complete expected package manifest, every target ID/kind, before value and expected direct object revision. Unknown fields, altered replacements, stale raw source bytes, changed package context (including presentation-only changes), and mismatched dependency snapshots fail without publication. A proposal with `conflict`, `unchanged` or `presentation-only` status is not revisable. At least one supported typed body change is required.

For `--proposal`, the finite transition rules preserve namespace, stable IDs, entity kinds/order, presentation, unresolved decisions and pending authority. Only existing entity bodies may change. All replacements are validated together using the ordinary candidate validator. There is no insertion, deletion, rename, move, retirement, cross-package lookup or implicit “latest revision” target resolution. Reference targets must exist with the expected kinds in the exact parent snapshot; if a target is also changed in this batch, its precondition still names its parent revision, while the output manifest names its resulting revision.

### Revision provenance v4

Apply/interview continue emitting authoring provenance v3. Revise emits a distinct strict `caac-candidate-provenance-v4` envelope; it does not invent answer IDs, answer history, `base_sha256`, `answers_sha256` or `field_changes` to impersonate an authoring run. It records current artifact hashes, the current revision manifest, the renderer profile, pending authority/evidence statuses and a `revision_transition` receipt containing:

- The exact bounded parent candidate JSON snapshot, its raw hash and full package revision.
- Parent provenance/review hashes and the supplied proposal hash.
- The exact nonempty changed-object set, with stable kind/ID and before/after direct revisions.
- `reference_preconditions`: every resulting relationship, including unchanged ones, with source and target addresses pinned to the parent package/object versions. These are not a claim that those relationships or list indices existed in the parent; a proposed relationship may select a different existing target.
- `evidence_policy: "fresh-review-required-for-whole-package"`. No inherited acceptance or executed evidence is granted, even for an unchanged direct-content hash. This conservative policy is not a transitive impact-analysis engine.

The receipt stays outside the canonical candidate, so lineage does not recursively enter the package hash. All supported transitions produce pending authority, `contract_complete: false`, `behavior_evidence: "not-run"` and pending exception review. Direct-content hashes remain distinct from effective meaning and do not authorize evidence reuse.

The ordinary [artifact verifier](#verify-the-three-generated-artifacts) accepts v4 and recomputes all current artifact fields plus the receipt's parent candidate/review hashes, parent package identity, preservation rules, exact changed-object set and complete resulting-reference preconditions. It checks the embedded parent snapshot's structural validity. It does not authenticate that snapshot as historical truth: parent-provenance and proposal hashes are only shape-checked opaque claims in standalone verification because those files are not supplied. A fully consistent co-edited unsigned history can pass. Revise itself does check the actual supplied source/proposal bytes and recomputed comparison before producing the record; that still is not a signature or owner acceptance. This is one-step lineage consistency, not a verified historical chain. For example, a new clause-to-definition relationship uses a path/index in the resulting candidate, but its precondition pins that definition's parent revision. If the definition changes in the same batch, the resulting manifest instead binds its new revision.

Successful revise exits 0; failed verification/preconditions/validation/publication exit 2. Verify a new result with `candidate verify --candidate NEW/candidate.json --provenance NEW/provenance.json --review NEW/review.md` before consuming it. It can be used for another comparison or `--proposal` revision only if its review fits their narrower 4 MiB input limit; the separate `--presentation` mode admits a 32 MiB source review. Inputs are regular nonsymlink files: candidate up to 1 MiB, source review up to 4 MiB, provenance/proposal up to 32 MiB, and embedded edited review up to 4 MiB. The embedded parent candidate is at most 1 MiB. Generated candidate is at most 1 MiB and review/provenance at most 32 MiB each. Every captured input is reread before publication; output overlap and existing paths are refused. These checks are not locks and do not exclude changes after the last reread. Require command success and verify all three artifacts; provenance-file presence alone does not prove success or authenticity. I/O failure can leave a partial output directory without final provenance; inspect it and choose a fresh directory rather than overwriting it.

This closes only a bounded expected-revision/transition step for the experimental candidate API. Broader canonical model, retirement, cross-file references, accepted-state lifecycle and evidence invalidation remain open.


## Explicit presentation revision

The same `candidate revise` command accepts exactly one of `--proposal` (existing entity bodies) or `--presentation` (a complete presentation request). Presentation revision uses the same [size and layout validation](#presentation-and-boundaries) as apply. For an expression package, select XS or flat S. Supply the complete new layout with all existing entity placements.

Prepare a strict JSON request with four required fields:

```json
{
  "format": "caac-presentation-request-v1",
  "expected_candidate_sha256": "COPY_THE_SOURCE_CANDIDATE_RAW_SHA256",
  "expected_package_revision": "COPY_THE_SOURCE_FULL_PACKAGE_REVISION",
  "presentation": {
    "size": "M",
    "nodes": [
      {
        "kind": "chapter",
        "id": "chapter-one",
        "title": "Explicit chapter",
        "articles": []
      }
    ]
  }
}
```

This is a request-shape illustration, not a runnable fixture: replace the two placeholders and populate `articles` with the complete article objects from your source, arranged as intended. Empty or incomplete placement fails. The verified source provenance exposes `candidate_sha256` and `revision_manifest.package_revision`; the first is the SHA-256 of exact candidate file bytes, the second is its full content revision. Copy both for the same source snapshot. For S, supply article nodes directly; for M, supply chapter nodes containing articles. Explicitly retain every entity exactly once. Titles, article/chapter placement and order may change; no missing content is filled in automatically. Unknown fields, duplicate fields or node IDs, missing entities, repeated placements, unknown targets, unsupported sizes and nesting are rejected.

For a complete S-to-M example, start with the S output of the [typed interview](#typed-non-auto-interview), verify it with [candidate verify](#verify-the-three-generated-artifacts), then create this request without overwriting an existing file:

```sh
python3 - /tmp/echo-candidate /tmp/echo-presentation-request.json <<'PY'
import hashlib, json, pathlib, sys
source = pathlib.Path(sys.argv[1])
raw = (source / "candidate.json").read_bytes()
candidate = json.loads(raw)
provenance = json.loads((source / "provenance.json").read_bytes())
assert candidate["presentation"]["size"] == "S", "This example requires an S source"
request = {
    "format": "caac-presentation-request-v1",
    "expected_candidate_sha256": hashlib.sha256(raw).hexdigest(),
    "expected_package_revision": provenance["revision_manifest"]["package_revision"],
    "presentation": {"size": "M", "nodes": [{
        "kind": "chapter", "id": "chapter-one", "title": "Explicit chapter",
        "articles": candidate["presentation"]["nodes"]
    }]}
}
if "profile" in candidate["presentation"]:
    request["presentation"]["profile"] = candidate["presentation"]["profile"]
with pathlib.Path(sys.argv[2]).open("x") as output:
    json.dump(request, output, indent=2)
PY
```

This wraps every existing S article under one explicit chapter without reconstructing or dropping article contents. Choose a chapter ID distinct from the source's presentation node IDs. The candidate still owns each semantic entity once; article `entities` lists are placements by ID, while operation/exception/case references do not create additional placements. Permitted title edits are article/chapter presentation labels, not semantic authored text. Display order may change while the semantic entity-array order remains identical. To return to S, use the complete article objects directly as `presentation.nodes` with `size: "S"`, preserving the applicable paragraph/item profile when those groups remain and pinning the new source hashes.

```sh
cargo run -- candidate revise \
  --candidate /tmp/echo-candidate/candidate.json \
  --provenance /tmp/echo-candidate/provenance.json \
  --review /tmp/echo-candidate/review.md \
  --presentation /tmp/echo-presentation-request.json \
  --output /tmp/echo-reframed-candidate
```

The source triple is verified against captured bytes before either precondition is checked. Even reformatted source candidate bytes with unchanged typed/package revisions make an old raw-hash request stale. A typed no-op presentation is rejected. A valid transition preserves the complete semantic section, including entity order, unresolved decisions and proposals, plus format and pending authority. The semantic revision and every entity/decision direct revision remain identical. A shared clause is still one semantic entity; moving it cannot change its exception or case relationships. The new full package/presentation revisions and exact reference addresses are regenerated, as are article/chapter numbering and mappings. In the wrapping example, each article retains its node ID and global number; its mapping gains `chapter_id: "chapter-one"`. Each target address uses the resulting package revision while its entity revision stays unchanged. A typed no-op means the parsed presentation is unchanged, regardless of request JSON whitespace.

### Presentation provenance v5

The three output artifacts remain candidate, derived review and provenance. This operation emits a distinct strict `caac-candidate-provenance-v5` with a `revision_transition` whose operation is `replace-presentation`; v4 body-change rules are not widened. Its receipt contains the bounded exact parent candidate snapshot/raw hash/package revision, parent provenance/review hashes, request hash, before/after presentation revisions, preserved semantic revision and object-revision list, complete before/after review mappings and the whole-package fresh-review policy.

Standalone artifact verification recomputes the current artifacts and every derived receipt field from the embedded parent and resulting candidate. It rejects any non-presentation change, no-op transition, missing/extra mapping or object, altered revision, unsupported operation, or authority/evidence claim. Parent-provenance and input-request hashes remain shape-checked historical claims because their files are not supplied to standalone verification; they are not signatures or proof that the parent existed historically. The actual revise invocation checks the supplied source and request bytes before generating that record.

All authority remains pending; behavior evidence remains not-run and exception review pending. Preserved semantic identity is not authorization to reuse evidence or acceptance. A later S→M→original-S transition can return to the original content-addressed package identity even though its new transition receipt differs. Revision hashes identify content, not a never-repeating chronological event number. The explicit operation still requires fresh whole-package review and confers no inherited authority.

Presentation requests and parent/current candidates are limited to 1 MiB each; source review/provenance and generated review/provenance are limited to 32 MiB each. Unlike `--proposal`, this mode does not parse an edited-review snapshot, so its source-review limit is 32 MiB rather than the comparator's 4 MiB. Existing nonsymlink regular-file, input/output overlap, exact reread and absent output-directory protections apply through the shared publication path. Failure does not mutate source; partial I/O publication requires inspection and a fresh destination. For supported L and XL views and module files, see [M, L and nested XL hierarchy](#m-l-and-nested-xl-hierarchy). For attachment and exception bindings, see [Attachments and sentence parts](#attachments-and-sentence-parts). Push, acceptance workflows and general move imports remain outside this operation.

## Read-only exact declared-graph trace

Add `--trace ID` to the existing verifier to inspect one exact entity or unresolved decision:

```sh
cargo run -- candidate verify \
  --candidate /tmp/echo-candidate/candidate.json \
  --provenance /tmp/echo-candidate/provenance.json \
  --review /tmp/echo-candidate/review.md \
  --trace rule-echo
```

Without the option, verification behaves as before. With it, a successful command emits exactly one terminal-safe JSON record on stdout, format `caac-candidate-trace-v1`, without success prose. Unknown IDs, incomplete arguments, stale/tampered artifacts or other errors exit 2 with no partial report on stdout. A report is an observation, not a new canonical file, transition, acceptance record or implementation-readiness gate. No input or history is changed. The shared verifier supports v3/v4/v5 provenance and the same unsigned-consistency limitations.

Successful trace exits 0 only for the consistency check and lookup, not for conformance. To inspect the one-line record, pipe stdout through `python3 -m json.tool`. The source triple comes from the [authoring workflow](#run-the-supplied-example); hash and address meanings follow [revision identities](#revision-identities-and-exact-derived-references).

IDs are unique across entities and decisions within the one supplied package namespace. They contain only ASCII letters, digits, `_` and `-`, so lexicographic ID ordering is unambiguous. All targets must resolve in the validated manifest. Traversal follows typed references from each source to its target. A case contributes one scalar `operation` or `expression` edge. Authored prose, paths and external links are data.

Top-level keys are `format`, `source`, `artifact_consistency`, `selected`, `direct_incoming`, `direct_outgoing`, `forward_reachable`, `reverse_reachable`, `reachable_declared_cases`, `unresolved_decisions`, `evidence_scope` and `limits`. `selected.reference` is an exact address; `selected.body` is the existing typed entity/decision; `selected.mapping` is an array of manifest presentation bindings. Direct edges reuse the [revision manifest](#revision-identities-and-exact-derived-references) without inventing another edge encoding. Cases and decisions contain `reference` and `body`, with explicit evidence/status labels. The [trace regression fixtures](../tests/candidate_trace.rs) exercise the concrete record.

The record binds raw source hashes and the full package revision. `selected` includes the exact namespace/package/kind/ID/direct-revision address, typed body and derived document mapping. Both entity IDs and unresolved-decision IDs are supported. Direct incoming/outgoing relationships retain the manifest's field/index and exact addresses. Forward and reverse reachability use finite visited-set traversals of those declared directed edges, exclude the starting object, and return exact references sorted by ID. Direct edge arrays retain manifest order.

`reachable_declared_cases` includes the selected object if it is a case, plus cases reachable wholly forward or wholly backward, in semantic entity order. This is the union of two one-direction closures, not an undirected or mixed-direction neighborhood: tracing one case does not include a sibling case merely because both reference the same operation. Reachability reports paths only, not causation, effective semantic impact, case coverage, satisfaction or permission. An unlinked implementation is not automatically a missing implementation finding. No implementation/test-link model is invented here.

`unresolved_decisions` includes every unresolved decision in the package, even when unlinked to the selected object. A proposal remains unresolved; zero decisions or cases cannot imply completion. Selected bodies and case records are data, not executed evidence. Evidence fields deliberately distinguish:

- `source_behavior_evidence`: the verified provenance's recorded value, currently `not-run`.
- `execution_evidence`: `not-recorded-in-supplied-artifacts`, not a claim that nobody ran tests elsewhere.
- `source_transition_evidence_policy`: the verified v4/v5 whole-package fresh-review policy, or `not-applicable` for v3. It does not invent stale/failed/passed classifications.
- Implementation/test links: `not-represented`; semantic review: `not-supplied`; owner acceptance and implementation admission: pending; completion: `not-assessed`.

The trace is not the full CAAC-13 obligation-to-implementation/evidence graph, a semantic-review gate or a completion verdict. Cases and reference connectivity cannot substitute for independently collected execution evidence or owner acceptance. These unsupported links remain explicit gaps rather than inferred success or failure.

Inputs use the same regular-file limits as verify: candidate 1 MiB and provenance/review 32 MiB each. Report serialization and its terminal-safe ASCII JSON encoding are bounded to 32 MiB before stdout; All non-ASCII and control characters are escaped in the emitted JSON. JSON consumers recover the original values; those decoded values remain untrusted and need escaping again before terminal or HTML display. All captured input bytes are rechecked before returning the record. This is a conflict check, not a concurrent-writer lock, authenticity proof or deadline.

### Applying citation-checked findings

For answers produced by `review-findings`, use the optional evidence-bound apply mode in [review findings](review-findings.md). It rechecks the explicit collection selections, current source snapshots, authored citations and exact handoff artifacts before the same candidate publisher runs. All claims and authority remain pending. Plain apply does not require evidence options. The v3 output format is unchanged, so standalone verification does not prove historical citation revalidation.
