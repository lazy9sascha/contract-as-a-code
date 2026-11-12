# Local tooling contract

Revision: draft-1. Status: draft. Scope: local CLI authoring support and example execution.

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

## Parse and extract

`parse` reads a Markdown contract and produces a JSON schema.
`extract` reads a Rust source file and produces a JSON schema.

TOOL-051 | `CLI` must parse Markdown contract clauses into JSON schema clauses.

TOOL-052 | `CLI` must parse Markdown operation signatures into JSON operations.

TOOL-053 | `CLI` must parse Markdown contract cases into JSON cases.

TOOL-054 | `CLI` must extract Rust trait methods into JSON operations.

TOOL-055 | `CLI` must extract doc comments as clause obligations.

TOOL-056 | `CLI` must preserve clause IDs found in doc comments during extraction.

Together, `gen`, `md`, `parse`, and `extract` enable bidirectional conversion
between schema, Markdown, and Rust code.

## Suggest

`suggest` is a mode of `interview` that reads source files and generates
heuristic suggestions for open decisions.

TOOL-057 | `CLI` must read source files when `--source` is provided.

TOOL-058 | `CLI` must generate suggestions based on keyword matching.

TOOL-059 | `CLI` must present suggestions as "Suggested:" lines before each question.

TOOL-060 | `CLI` must not auto-apply suggestions. The owner decides.

## Contract harness

`run` executes the contract completion loop: check, interview, repair, re-check.

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

TOOL-071 | `CLI` must warn about unfilled slots.

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

The sync command enables bidirectional conversion. It does not
auto-apply changes to source files. Repair remains the author's responsibility.

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

The interview supports the contract completion loop. It does not invent
owner decisions. Unanswered questions remain as `OPEN:` lines.


## Prototype limits

No command compiles natural language, generates approved obligations, switches live traffic, or edits an existing project.

Authoring commands generate review scaffolds. Human or agent review supplies requirement text and records unresolved meaning explicitly.

## See also

- [Method contract](../CONTRACT.md): method rules.
- [Batch example](../examples/batch-division.md): complete example.
- [Quickstart](../docs/quickstart.md): five-minute guide.
