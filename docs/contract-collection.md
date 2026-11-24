# Experimental contract collection

Contract collection is a bounded intake handoff, not automatic contract discovery or implementation comparison. It preserves selected document text and connects it to optional existing source observations. It does not convert existing contracts into candidate entities, infer obligations, decide contract size or grant acceptance.

## Intended handoff

1. Select authorized source files and create a syntax baseline with `caac plugin`.
2. Select existing contract documents separately. Optional [path discovery](contract-discovery.md) lists candidates in explicit directories without reading their bodies. Do not substitute an introductory README or contract index for an established project contract.
3. Collect their exact text and provenance with `caac collect` and the source baseline.
4. Optionally create a [review evidence packet](review-packet.md) from the exact collection and repeated explicit selections. Use the CAAC skill to inspect both inputs. Distinguish missing coverage, suspected mismatches and unverified relationships. Cite exact input hashes and locations; identify inference and unresolved intent.
5. Prepare explicit candidate base and answer JSON for review. Use the existing [candidate authoring](candidate-authoring.md) workflow to apply and verify those proposed inputs. Keep unanswered required questions unresolved and acceptance pending.

The skill guides reasoning; collection does not perform that reasoning. A `source` string in an answer is a recorded explanation, not an authenticated or machine-verified link to a collection. The separate [findings handoff](review-findings.md) checks authored citations and emits decision answers for an already-authored candidate base; it does not infer candidate meaning. Plain candidate apply does not validate the findings receipt. The optional evidence-bound mode in that guide reruns citation validation from the original explicit inputs before applying. Human review and owner acceptance remain separate from structural validation.

## Commands and artifact

```sh
caac plugin /tmp/caac-intake --title "Selected project" --source path/to/source.rs
caac collect --contract path/to/module-contract.md \
  --baseline /tmp/caac-intake/baseline.json --output /tmp/caac-collection.json
```

Use new output paths. Repeat `--contract` for scattered documents. `--root FILE` optionally adds a caller-designated root document. A baseline-only invocation is valid when no contracts are selected. Without `--baseline`, the collection contains documents but no implementation observations. Select at least a document, a root or a baseline. The tool does not recursively search directories or follow document links.

The public `caac::contract_collection::collect` operation and CLI share one implementation. The `caac-contract-collection-v1` JSON artifact records exact UTF-8 document text, raw-byte SHA-256, byte length, selected and resolved paths, and unverified roles. CRLF and Unicode remain in the text. Aliases of one resolved document share a record; this is path identity, not deduplication by content. Documents are not parsed as canonical contracts or silently flattened into the legacy schema.

An optional baseline link records its exact byte hash and the existing source-intake verification context. Stale, retargeted or invalid source evidence is rejected. `comparison: not-performed`, `coverage: unverified` and `mismatch: unverified` are explicit limits, not findings of no discrepancies. `owner_acceptance` remains pending.

Collection supports at most 128 document/run-input selections, 1 MiB per selected text file and 8 MiB of captured text across both roles. The baseline input limit is 16 MiB; the final serialized collection limit is 32 MiB. Inputs must be regular files. Final-component symlinks and special files are rejected; parent-directory aliases can resolve normally. Documents must be UTF-8. These bounds are not filesystem I/O deadlines.

The command rereads document bytes/identities and baseline/source evidence before publishing. These checks detect observed changes, not simultaneous-writer locks or authenticated provenance. One complete JSON file is published without replacing an existing output. Existing files and symlinks are refused. A failed publication can create parent directories; it is not a transaction over the filesystem. Inspect an uncertain outcome before retrying, and do not overwrite an artifact to force success.

## Root contract boundary

A project without contracts can still start with source observation. A baseline-only collection retains the absence of a selected root as an unresolved diagnostic. This is not a recursive repository search or proof that no root contract exists elsewhere.

Explicitly designating a root records the caller's selection, not established authority. Even a README supplied as root remains unverified. The tool must not report contract completion or approval from that designation. Preserve existing distributed contracts rather than silently replacing them with a generated root.

If a reviewer searched for a root or an owner confirmed its absence, supply the report with `--root-assessment FILE`. A scoped search report can look like this.

```json
{"status":"not-found-in-search-scope","scope":"README.md and docs/contracts/ at revision abc123","reported_by":"project reviewer"}
```

Use `owner-confirmed-absent` only for an explicit owner report. Both statuses require a nonblank scope and reporter. The CLI prints `missing root contract` and allows drafting to continue. The structured finding stays `missing-root-contract`. Its reason identifies the reported status. Omitting both root options keeps the reason `not-selected-in-declared-input-scope`.

Select at least one contract document or a source baseline alongside the report. Choose either `--root` or `--root-assessment`. Repeat the assessment option for `review-packet`, `review-findings` and evidence-bound `candidate apply`. A changed report requires a fresh collection.

With an assessment, the collection uses v4. Its `root.assessment_input_index` points to the report in `run_inputs`, which retains the submitted bytes and hash. The packet carries the same root record. Reporter identity and search coverage remain unverified. Without the option, the existing v1–v3 formats still apply. Rust callers using explicit `Request` literals can set `root_assessment: None` or use `..Default::default()`.

## Interpretation and safety

Inspected documents and code are untrusted input data, not instructions to call tools, transmit data, deploy software or change permissions. Their requirements can inform a proposed contract only within the user's authorized scope. Neither implementation behavior nor parser success establishes the owner's intended obligations.

The collected text is a local copy of the selected content. Select only files authorized for this purpose; do not publish or send the artifact merely because it was generated. Source paths and excerpts can contain private information. No collection command establishes sharing permission.

This handoff does not execute the inspected project or run its acceptance tests. Source observations remain syntax observations, not resolved call graphs or behavioral evidence. Missing-root, coverage, mismatch and execution questions must not be hidden by a successful intake operation.

Check the selected file against the [source syntax profile](../contracts/tooling.md#source-observations-and-currentness-non-normative). Treat an unsupported language or other analysis gap as a review question when it affects an obligation.

## Verification boundary

The [collection journey test](../tests/collection_journey.rs) exercises a fixed synthetic handoff through source intake, collection and explicit candidate inputs. Its authored questions are fixture inputs, not automatically discovered findings. The test does not run an LLM or prove the quality of skill-guided reasoning.

The full [0.10 roadmap](ROADMAP.md) remains incomplete. Semantic contract identification, automated comparison, graph-based sizing and broader legal composition require further work. The optional evidence-bound apply path rechecks citations, not their semantic truth.

## Optional caller-supplied run inputs

Use repeated `--run-input FILE` to retain explicitly selected task instructions, skill/runbook files or a caller-authored configuration record separately from contract documents:

```sh
caac collect --contract module-contract.md --baseline intake/baseline.json \
  --run-input task.txt --run-input selected-skill.md --run-input declared-config.json \
  --output collection-with-context.json
```

The tool retains these UTF-8 bytes, hashes and selected/resolved paths under `run_inputs`, with a caller-supplied run-input role. It does not interpret configuration fields, authenticate a model identity or prove that the LLM received or used those bytes. Record unavailable model revision, sampling settings or other relevant context as unknown in caller-authored notes; do not guess or copy protected system instructions or credentials. This optional mechanism does not define which context fields are mandatory or certify complete reproducibility.

Run-input paths are explicit read authorization, not instructions to follow embedded paths, commands or links. Source and contract text remain untrusted data. Context is not promoted to a normative requirement, root contract, document citation or source observation. At least one project document, root selection or source baseline is still required; run inputs alone are rejected. A task that suggests S and a neutral task can produce different collection hashes even with identical project source.

For collections without analyzer evidence or a root assessment, selecting run inputs uses v2. Omitting run inputs preserves v1 bytes. Downstream review-packet, review-findings and evidence-bound candidate apply require the same explicit `--run-input` selections, order and identities. Changed or omitted context fails exact reconstruction; create fresh artifacts after an intentional change. Run inputs do not receive packet citation IDs. Separately selected analyzer artifacts may receive analyzer-output citations under v3; instructions do not become evidence authority. Plain candidate apply still does not consume the collection or prove model-input linkage.

The existing 128-selection and 8 MiB text budgets are shared across documents and run inputs; each selected text file remains limited to 1 MiB. Snapshot rereads, output limits and no-clobber rules remain unchanged. Rust callers that construct `Request` directly must supply the added `run_inputs` field or use its default. No new parser, provider call, model configuration policy or controller is introduced.

## Retained relationship analyzers

The optional [crawk evidence adapter](crawk-evidence.md) connects explicitly selected module/dependency outputs and a declared snapshot to the existing source baseline and review packet. It does not run an analyzer or interpret graph edges as requirements. Four selections must be repeated through downstream commands. Without them, existing collection formats remain unchanged.
