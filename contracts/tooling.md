# Local tooling contract

Revision: draft-3. Status: draft. Scope: local CLI authoring support and example execution.

The user requested implementation in this session. That request authorizes prototype work, not approval of generated contract requirements.

The actor is `CLI`. The Rust executable is named `caac`.

Commands are `scratch`, `plugin`, `migration`, `baseline check`, `check`, `watch`, `interview`, `resume`, `run`, `gen`, `md`, `parse`, `extract`, `sync`, and `pattern`. Source suggestions are an `interview --suggest` mode and part of `run`, not a standalone command.

Current command inventory note (non-normative): the experimental 0.10 line also exposes `discover`, `collect`, `review-packet`, `review-findings`, and `candidate` subcommands. Their bounded profiles are documented in [candidate authoring](../docs/candidate-authoring.md), [discovery](../docs/contract-discovery.md), [collection](../docs/contract-collection.md), [review packets](../docs/review-packet.md), and [findings](../docs/review-findings.md). This inventory is not a normative contract for every new command. The [contract-to-execution audit](../reports/scan-draft-contract-map.md) records that mapping gap.

## Authoring modes

Each authoring command takes an output directory and a required `--title`. The title defines the selected scope.

The optional `--size` records an XS, S or M scope estimate in the Markdown scaffold. A scaffold without `--size` omits the estimate. Select a rendering profile in the [typed candidate workflow](../docs/candidate-authoring.md#presentation-and-boundaries).

`plugin` and `migration` require one or more explicit `--source` file arguments. They do not discover an entire repository.

TOOL-001 | `CLI` must reject an existing output path without changing its contents.

TOOL-002 | `CLI` must reject an empty title or a title containing a line break.

TOOL-003 | `CLI` must create one draft `CONTRACT.md` for each authoring command.

TOOL-004 | `CLI` must include terms, inputs, results, failures, clauses, cases, and open decisions in each generated contract.

TOOL-005 | `CLI` must leave unknown requirements as explicit open decisions.

TOOL-006 | `CLI` must not infer owner approval from source code or a successful command.

TOOL-007 | For `scratch`, `CLI` must not create a baseline or require existing implementation files.

TOOL-008 | For `plugin` and `migration`, `CLI` must reject missing source files before creating the output directory.

TOOL-009 | For `plugin` and `migration`, `CLI` must record each selected source path and its SHA-256 digest in `baseline.json`.

TOOL-010 | `CLI` must not execute or modify selected source files.

TOOL-011 | For `plugin`, `CLI` must include a section for observed behavior and proposed requirement changes.

TOOL-012 | For `migration`, `CLI` must include keep/change/remove decisions, target behavior, cutover conditions, and rollback conditions.

TOOL-013 | `CLI` must label every generated contract as draft with owner acceptance pending.

The baseline identifies selected file contents. It does not establish test success or inventory completeness.

## Source observations and currentness (non-normative)

`plugin` and `migration` record `source-syntax-v1` in `baseline.json`. The analyzer selects a Tree-sitter 0.27.2 grammar by the resolved UTF-8 file's case-sensitive extension.

| Extension | Grammar | Recorded syntax |
| --- | --- | --- |
| `.rs` | `tree-sitter-rust` 0.24.2 | Function declarations and signatures, including enclosing module, trait or implementation scope where present. |
| `.py` | `tree-sitter-python` 0.25.0 | Function declarations, decorators, `raise` and `assert`, with enclosing scope where present. |

The [source-intake CLI tests](../tests/source_intake_regressions.rs) cover selected examples in both languages. The Rust generation profile remains separate and uses Syn.

Each captured file records its SHA-256 digest, byte length, resolved identity, and all selected path aliases. Syntax observations include exact source excerpts, half-open byte ranges, one-based LF line and byte-column locations, enclosing scope, and parser versions. Source bytes and line endings are not normalized. Other extensions record `unsupported-language`. Invalid UTF-8 records `unsupported-encoding`. These statuses, parse errors and exhausted budgets produce no observations for the affected file.

Observations do not establish runtime behavior, reachability, active configuration, expanded macros, resolved types, imports, decorator effects, or whole-project coverage. Source text claiming approval cannot change `authority: observation`, `execution: not-run`, or `owner_acceptance: pending`.

Run `caac baseline check OUTPUT/baseline.json` to compare the recorded files with current source bytes, selected-path resolution, and recomputed syntax evidence. The JSON receipt identifies the exact baseline input by resolved path and SHA-256. It reports stale, retargeted, unavailable, or inconsistent evidence without rewriting the historical baseline. A matching source with unsupported analysis can be identity-current; this does not establish successful source analysis or semantic verification.

A successful check means the selected identities and recorded bounded evidence match at the check, not that requirements or a contract are accepted. Changed files with the same length and timestamp still fail their digest comparison. An alias pointing to a different target is not current, even if its target has identical bytes. Old, unversioned baselines are unsupported; create a new authoring output rather than silently upgrading them.

Intake is limited to 128 selected paths, 1 MiB per source file, and 8 MiB of captured source bytes. Baseline input and output are limited to 16 MiB. Per-file analysis limits include 100,000 visited syntax nodes, depth 128, 2,048 observations, 16 KiB per excerpt, and 1 MiB of emitted names, scopes, and excerpts. Parsing has a one-second cancellation budget; traversal also checks a two-second overall budget. These are work bounds, not a filesystem I/O deadline.

Only regular files are read. Unix reads use nonblocking, no-follow descriptor opening and metadata consistency checks. Obvious special files are rejected before opening. These checks detect sampled concurrent changes; they are not an atomic filesystem snapshot or a lock against arbitrary writers. A source can change after a check returns. External dependency and semantic currentness remain unknown.

This intake does not resolve OPEN decisions, convert answers into canonical obligations, or complete an authoring role. Suggestions and answer-to-schema conversion remain separate work.

## Declared syntax checks

`check` takes one or more Markdown files. Their combined clauses form the checked set.

A clause occupies one physical line: `ID | sentence`. An ID contains uppercase letter groups and ends with three digits.

A sentence uses whitespace-separated words. Its ID does not count toward the 20-word limit.

`must`, `must not`, and `may` are normative markers. A clause needs a marker and text before and after it. Backtick-quoted marker names in explanatory prose do not declare obligations.

A fenced code block is illustrative. An HTML comment is non-normative. Neither defines clauses or references.

A heading containing `(non-normative)` excludes its section and subordinate sections from checks. A peer or ancestor heading ends that exclusion.

A reference uses `@ID` outside excluded content. A bare token is not a reference.

The diagnostic fields are `path`, `line`, `code`, and `message`. Line numbers start at one.

TOOL-014 | `CLI` must report duplicate clause definitions across the checked set.

TOOL-015 | `CLI` must report references without definitions in the checked set.

TOOL-016 | `CLI` must report malformed clause definitions, empty bodies, missing markers, and missing text beside markers.

TOOL-017 | `CLI` must report numbered clauses that exceed 20 words or contain a semicolon.

TOOL-018 | `CLI` must report unnumbered lines containing normative markers outside excluded content and Markdown tables.

TOOL-019 | `CLI` must report each `OPEN:` line as an unresolved decision.

TOOL-020 | `CLI` must report files containing no clause definitions.

TOOL-021 | `CLI` must report unclosed code fences and HTML comments.

TOOL-022 | `CLI` must include file and line information for every diagnostic.

TOOL-023 | `CLI` must support machine-readable diagnostics through `check --json`.

TOOL-024 | `CLI` must exit with code 1 when declared syntax checks find defects.

TOOL-025 | `CLI` must exit with code 2 for argument errors or unreadable input files.

TOOL-026 | `CLI` must exit with code 0 when the requested operation completes without reported defects.

TOOL-027 | Every successful check must state that semantic review remains pending.

These checks do not identify all undefined domain terms, missing branches, conflicting meanings, or invalid inheritance. They do not authenticate approval.

## Markdown admission profile (non-normative)

The checker and Markdown converter share Comrak 0.59.0 structure and source locations. Only the initial YAML frontmatter extension is enabled. GFM table parsing is disabled because its transformed cell positions do not preserve raw source offsets. A valid generated CAAC metadata prefix permits frontmatter immediately after that prefix.

Analysis normalizes the supported physical line separators without changing the source file. Diagnostics retain physical line numbers. Canonical fields retain source spelling, including escapes and entities. Complete HTML comments are omitted only when their spans match the source. Invalid source ranges produce errors. Inline code suppresses explanatory normative markers, but not explicit clause references.

The supported normative profile uses plain ATX headings without closing hashes. Closed, top-level fenced code blocks are illustrative. Setext or formatted headings, block quotes, indented or container code, multiline inline code, and raw or mixed HTML are unsupported in normative content. These structures produce errors instead of silently removing obligations. An unsupported peer heading cannot prolong a non-normative exclusion. Unclosed fences, comments, and frontmatter remain errors.

CAAC admission remains separate from CommonMark recognition. Operation declarations still use the declared dash notation. Case rows still require four explicit outer-pipe cells and the existing escaped-pipe encoding. Library recognition does not expand either domain format. Generated metadata still requires exact body validation for schema recovery.

This profile is not arbitrary Markdown conversion, semantic completeness, or owner acceptance.

## Example binding

The executable example follows [the batch contract](../examples/batch-division.md).

The library uses `serde_json::Value` for its input. Its public API is `batch::run(input: &Value) -> BatchResult`.

`Value::Null` represents absent input. A valid input is an array of zero through 100 pair objects.

Each pair object has exactly two fields: `numerator` and `denominator`.

Each field accepts a JSON integer number from -1000 through 1000, inclusive. A Boolean, float, null, missing field, extra field, or out-of-range value is invalid.

`BatchResult` has two variants: `InvalidInput` and `Completed(Vec<ItemResult>)`.

`ItemResult` has two variants: `Ok(i64)` and `ZeroDivisor`.

The library exposes a Rust `BatchProcessor` trait as the extension point. `Divide` supplies its `apply` operation.

`BatchProcessor::run` provides the validated default operation. A trait implementer can override it.

Rust does not mechanically prove that an arbitrary implementation follows the batch contract.

`BatchProcessor::apply` accepts values in the batch contract's -1000 through 1000 range. `BatchProcessor::run` validates the full batch before the first call.

The input remains unchanged during a call. Division truncates toward zero. A zero divisor returns `ZeroDivisor` and does not stop later items.

TOOL-028 | The Rust library must expose `batch::run`, `BatchProcessor`, `Divide`, `BatchResult`, and `ItemResult`.

TOOL-029 | `batch::run` must follow @BATCH-001 through @BATCH-014 and @DIV-001 through @DIV-005.

TOOL-030 | The Rust implementation must not require a Python runtime or Python bridge.

TOOL-031 | The `caac` executable must display usage and exit with code 0 for top-level and subcommand `--help` requests.

TOOL-032 | `batch::run` must reject JSON null, Booleans, floats, and null or non-integer pair fields as invalid input.

TOOL-033 | The tooling contract must state that Rust does not enforce arbitrary `BatchProcessor` conformance or `run` overrides.

## Repair watch

`watch` takes one or more Markdown files and a polling interval.

TOOL-034 | `CLI` must run declared syntax checks on the selected files immediately.

TOOL-035 | `CLI` must re-run checks when a selected file changes.

TOOL-036 | `CLI` must report PASS or FAIL with finding count on each iteration.

TOOL-037 | `CLI` must accept `--interval SECS` to set the polling interval.

TOOL-038 | `CLI` must exit with code 0 on help requests for `watch`.

The repair watch supports the contract completion loop. It does not repair
files automatically. Repair remains the responsibility of the author or agent.

## Contract schema and code generation

A contract can be expressed as a JSON schema. The schema is the machine-readable
form. Markdown and code are derived from it.

`gen` takes one or more schema files and generates a Rust trait.
`md` takes one or more schema files and generates a Markdown contract.

TOOL-045 | `CLI` must parse a JSON contract schema with name, terms, operations, clauses, and cases.

TOOL-046 | `CLI` must generate a Rust trait with one method per operation.

TOOL-047 | `CLI` must include clause IDs as doc comments on each method.

TOOL-048 | `CLI` must generate a Markdown contract from a schema.

TOOL-049 | `CLI` must support multiple input files for modular contracts.

TOOL-050 | `CLI` must merge multiple schemas into one output.

The schema is the source of truth. Markdown and code are derived artifacts.
Generated artifacts contain versioned schema metadata. Recovery checks their
complete rendered contents against that metadata before restoring the schema.
The metadata preserves authored values; it does not authenticate owner approval.
Rust metadata also carries fields absent from the trait body, including cases and
standalone invariants. Editing those fields can remain body-consistent.
Recovery is not proof of the original revision or its acceptance.
Exact-body recovery uses the current renderer. Preserve canonical JSON and regenerate
views after layout changes. The metadata version identifies schema encoding, not layout compatibility.
Unknown JSON fields, conflicting merge definitions, and unsupported Rust
signatures are rejected instead of silently removed.
Rust generation accepts a bounded identifier and type subset. Function-pointer
types, explicit lifetimes, comments, attributes, and raw identifiers are unsupported.
The Rust parser is `syn`. A schema signature field has a 4096-byte limit.
Each parameter or return field permits at most 64 reference, generic, or parenthesis openings.
This complexity budget applies before parsing. Type-name resolution requires a Rust compiler.
Admission does not establish type correctness or compilation.
The elision guard checks explicit references, not lifetimes hidden in nominal types such as `Cow`.
Unsupported syntax and ambiguous explicit-reference returns fail before output.
Contract inheritance references remain metadata, not Rust supertraits.
Generated traits declare signatures; they do not implement contract behavior.

## Parse and extract

`parse` reads a Markdown contract and produces a JSON schema.
`extract` restores a schema from unchanged CAAC-generated Rust with supported metadata.
Unmarked, body-inconsistent, or unsupported Rust is rejected. Arbitrary Rust import is not supported.

TOOL-051 | `CLI` must parse Markdown contract clauses into JSON schema clauses.

TOOL-052 | `CLI` must parse Markdown operation signatures into JSON operations.

TOOL-053 | `CLI` must parse Markdown contract cases into JSON cases.

TOOL-054 | `CLI` must restore operations from verified CAAC-generated Rust metadata.

TOOL-055 | `CLI` must preserve complete schema obligations when recovering verified generated artifacts.

TOOL-056 | `CLI` must reject generated artifacts whose rendered contents conflict with embedded schema.

These commands preserve the schema through unchanged generated artifacts.
Handwritten Markdown parsing supports explicit clauses across sections, operation
signatures, declared operation sections, standalone invariants, terms, cases,
exclusions, and base references. Examples, comments, frontmatter, and declared
non-normative sections do not supply obligations. Unknown local clause references
and malformed supported structures are rejected. Nested normative headings and
standalone condition markers are unsupported. General prose interpretation and
legal-style structure remain outside this parser's admitted grammar.
Operation lists require `-` entries. Case tables require outer pipe characters.
Other nonblank content in these sections is rejected, except resolved DECISION annotations.
Those annotations do not create canonical obligations or prove that answers were implemented.
Clause reference checks cover actors and obligations in Markdown and JSON admission.
Generation rejects escaping that changes a clause's reference roles.
Arbitrary natural-language prose is not converted into executable semantics.

## Suggest

`interview --suggest` presents review questions linked to bounded source syntax.
It does not generate contract answers. `run` presents the same questions before its interview step.

TOOL-057 | `CLI` must read source files when `--source` is provided.

TOOL-058 | `CLI` must classify pending-review questions by decision keywords without inferring requirements from source syntax.

TOOL-059 | `CLI` must display "Suggested question:" lines before interviewing the corresponding file.

TOOL-060 | `CLI` must not auto-apply suggestions. The owner decides.

### Suggestion evidence profile (non-normative)

Revision draft-3 replaces TOOL-058's unsupported heuristic answers with question-category selection and narrows TOOL-059 to the actual per-file batch display. These IDs are preserved. TOOL-060 is unchanged. This draft revision does not establish owner acceptance or approval of any generated requirement.

`interview --suggest [--source FILE ... | --baseline FILE] FILE ...` accepts either fresh source capture or one historical baseline, not both. Source evidence options on `interview` require `--suggest`. `run` accepts the same mutually exclusive evidence options without that flag. `--` ends option parsing for contract filenames; a source filename beginning with a hyphen can use an explicit relative path such as `./-source.rs`.

Fresh sources use the bounded intake profile above. Missing or unreadable selected files are errors, not silently omitted evidence. Historical evidence is read once for its exact digest, parsing, and verification. Stale, retargeted, unavailable, or inconsistent historical evidence fails without fallback to fresh evidence. Unsupported language, encoding, syntax, and parser-budget statuses remain explicit even when identity matches. Empty evidence is allowed for questions, not as support for an inferred answer.

Each file's question batch contains one evidence context and pending-review question records. The context retains every selected source record, exact hash, source span, parser profile, and available observation. Each question references the SHA-256 of the exact serialized context, rather than choosing the first source or declaration. The link identifies candidate evidence, not an established mapping between source behavior and a requirement. Context authority remains observation, execution is not run, and owner acceptance is pending. Question authority is proposal and status is pending-review.

Keyword categories select questions about inputs, results, actors, failures, or scope. Declaration, return-type spelling, raise, assert, and decorator syntax cannot determine required domains, actors, outcomes, or runtime failures. Ambiguous or conflicting observations remain unresolved. Proposed definitions and exception relationships are explicitly unsupported by these question records; typed canonical relationships and answer-to-schema conversion remain separate work.

All selected contract files receive a read/check preflight. Evidence is captured or rechecked before each file's suggestion batch and each later harness interview step, including explicit evidence validation when no OPEN remains. Snapshots are current only at capture/check, not for the duration of an interactive session. The shared checker determines OPEN locations and excluded content. Excluded-only files do not receive a suggestion interview checkpoint.

Displayed source text, paths, symbols, and decisions are JSON-quoted and ASCII-escaped, including terminal controls, directional controls, and Unicode line separators. Decoding these strings preserves the raw evidence; escaping does not grant it authority. The entire batch is prepared before display, with a 32 MiB serialized-display limit and a 16 KiB decision-text limit. Exceeding either fails rather than silently truncating evidence. An earlier completed file is not rolled back if a later file changes or fails; this is not a multi-file transaction.

Questions are displayed before the ordinary interview. Skipping or pressing Enter does not apply a suggestion. Source text claiming approval has no authority. An explicit owner answer still records only a DECISION annotation under the existing interview workflow, not an accepted obligation or automatic schema change.

## Contract harness

`run` executes a pre-review loop: check, interview, repair, re-check.
A successful pre-review check does not establish semantic completion, executed
case results, independent review, or owner acceptance.

TOOL-061 | `CLI` must run declared syntax checks on the first iteration.

TOOL-062 | `CLI` must interview open decisions when findings include open decisions.

TOOL-063 | `CLI` must re-check after interview answers are recorded.

TOOL-064 | `CLI` must stop when no findings and no open decisions remain.

TOOL-065 | `CLI` must accept `--max-iterations` to bound the loop.

The harness supports the contract completion loop. It does not invent
owner decisions or claim semantic completeness.

## Pattern library

`pattern list` shows available prepared patterns grouped by family.
`pattern gen` generates Rust code from a pattern template.

TOOL-066 | `CLI` must list all prepared patterns with their ID, name, and slots.

TOOL-067 | `CLI` must generate code from a selected pattern template.

TOOL-068 | `CLI` must reject unknown pattern IDs with a list of available IDs.

TOOL-069 | `CLI` must accept `--slots JSON` to fill pattern slots.

TOOL-070 | `CLI` must replace `{slot}` placeholders in the template with provided values.

TOOL-071 | `CLI` must reject missing used slots when the caller supplies `--slots`.

Bound generation rejects unknown slots, unsupported bindings, and values outside the supported identifier or type grammar before creating output.
The list distinguishes bindable slots from declared slots that the scaffold does not implement.
Without `--slots`, generation exports an unbound scaffold. It does not validate compilation or behavior.

The pattern library provides prepared counterparts. Slot filling is the
author's responsibility. Generated code is a scaffold, not a complete implementation.

## Sync

`sync` reads a contract schema and generates both code and documentation.
With `--watch`, it polls for changes and regenerates automatically.

TOOL-072 | `CLI` must read a JSON contract schema as input.

TOOL-073 | `CLI` must generate a Rust trait when `--code` is provided.

TOOL-074 | `CLI` must generate a Markdown contract when `--docs` is provided.

TOOL-075 | `CLI` must support `--watch` for continuous regeneration.

TOOL-076 | `CLI` must support `--interval` for watch polling.

Sync regenerates outputs from the schema only. It is not reverse synchronization.
Initial destination paths are absent. Existing paths are rejected.
Preflight detects exact and resolved-parent collisions, but not every filesystem alias.
Watch updates only outputs created by its current invocation. Detected external
output edits stop regeneration. This check is not a lock against simultaneous writers.
Publication is atomic per file, not transactional across several outputs.
An I/O failure can leave earlier outputs published; the error reports that limit.

## Interview and resume

`interview` takes one or more Markdown files. It finds `OPEN:` lines and prompts
the owner for answers. Answers replace the `OPEN:` line with a `DECISION:` record.

`resume` continues a previous interview from its saved state.

TOOL-039 | `CLI` must find all `OPEN:` lines in the selected files.

TOOL-040 | `CLI` must prompt the owner for each open decision.

TOOL-041 | `CLI` must record answers as `DECISION:` lines in the contract.

TOOL-042 | `CLI` must save interview state to a sidecar file for resumption.

TOOL-043 | `CLI` must support `--auto` mode that skips all prompts.

TOOL-044 | `CLI` must report resolved and skipped counts.

The interview supports the pre-review loop. It does not invent
owner decisions. Unanswered questions remain as `OPEN:` lines.
Answers are single-line text without HTML comment delimiters. The same restriction
applies before pending answers are replayed. This is a Markdown-specific boundary,
not a source-language-neutral sanitizer.


## Prototype limits

No command compiles natural language, generates approved obligations, or switches live traffic.
Interview deliberately edits its selected contract. Watch sync updates its own generated outputs.

Authoring commands generate review scaffolds. Human or agent review supplies requirement text and records unresolved meaning explicitly.

## See also

- [Method contract](../CONTRACT.md): method rules.
- [Batch example](../examples/batch-division.md): complete example.
- [Quickstart](../docs/quickstart.md): five-minute guide.
