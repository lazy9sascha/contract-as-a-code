# Local tooling contract

Revision: draft-2. Status: draft. Scope: local CLI authoring support and example execution.

The user requested implementation in this session. That request authorizes prototype work, not approval of generated contract requirements.

The actor is `CLI`. The Rust executable is named `caac`.

Commands are `scratch`, `plugin`, `migration`, `check`, `watch`, `interview`, `resume`, `run`, `gen`, `md`, `parse`, `extract`, `sync`, and `suggest`.

## Authoring modes

Each authoring command takes an output directory and a required `--title`. The title defines the selected scope.

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

`suggest` is a mode of `interview` that reads source files and generates
heuristic suggestions for open decisions.

TOOL-057 | `CLI` must read source files when `--source` is provided.

TOOL-058 | `CLI` must generate suggestions based on keyword matching.

TOOL-059 | `CLI` must present suggestions as "Suggested:" lines before each question.

TOOL-060 | `CLI` must not auto-apply suggestions. The owner decides.

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
