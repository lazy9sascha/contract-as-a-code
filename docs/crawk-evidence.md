# Retained crawk relationship evidence

This adapter imports explicitly selected existing analyzer output into the CAAC collection/review workflow. It reuses crawk's dependency analysis rather than adding a parser, resolver, SCC implementation or clustering service. It does not execute crawk, Cargo or target code, install dependencies, infer contract boundaries, or certify the output's producer.

## Select existing outputs

Use a current source baseline and four explicit selections together:

```sh
caac collect --baseline intake/baseline.json \
  --contract selected-contract.md \
  --crawk-modules crawk-modules.txt \
  --crawk-deps crawk-deps.txt \
  --crawk-snapshot snapshot.json \
  --crawk-source-root /authorized/pinned-source \
  --output collection.json
```

The module input admits the retained target-tagged `list --targets` output shape; dependency input admits plain dependency lines with optional opaque API annotations. The parser profile is `crawk-0.7.1-plain-v1`. It describes admitted syntax, not proof that crawk 0.7.1 actually produced the files. Do not relabel DOT, grouped output or unsupported syntax to force admission. Malformed records fail explicitly rather than disappearing. This narrow grammar uses ASCII-centered target/module labels; unsupported Unicode names or target forms fail rather than being normalized into a different identity.

The source root must be an explicitly selected nonsymlink directory. The strict snapshot input has `revision` and `files` records with normalized relative `path` and SHA-256 values. Snapshot paths are matching keys only: the adapter never follows them to read more files. Already validated baseline files must be beneath the selected canonical root, have matching relative paths and match the declared hashes. Other manifest entries remain unverified. The revision string and graph-to-snapshot association remain caller claims.

All four flags and a baseline are required. Repeat the same selections for `review-packet`, `review-findings` and evidence-bound `candidate apply`. Existing run-input selections can retain available invocation/configuration records; absent producer or option evidence remains not observed. Embedded paths and instructions never authorize extra reads or actions.

## What the collection means

Analyzer selections use collection v3. Adding an explicit [root assessment](contract-collection.md#root-contract-boundary) selects v4 and retains the same analyzer evidence. `analyzer_evidence` is separate from contract documents and caller context. It retains the three raw input files and their exact hashes, plus parsed inventory and dependency records. The snapshot artifact is a caller manifest, not analyzer-produced source evidence.

Module records retain target tags and labels. Each dependency endpoint lists matching module-inventory indices:

- `inventory-unambiguous`: one matching retained inventory row;
- `ambiguous`: several matches, such as the same module label in library and binary targets;
- `unresolved`: no matching row.

These statuses are not compiler name resolution. API annotations are opaque analyzer text, not resolved calls or exact source spans. Raw line byte ranges refer to the analyzer output, not source code. The adapter does not repair missing edges, infer shared storage coupling or choose a target for an ambiguous name.

`selected_files_match` records only the validated baseline subset. `unverified_manifest_files` lists the rest. Partial agreement does not make the whole graph current or complete, and does not authenticate the alleged analysis run. A fresh baseline can invalidate stale selected evidence, but cannot establish that all possible dependencies were discovered.

## Evidence-to-draft workflow

1. Collect selected source/contract evidence and the retained graph outputs.
2. Create a review packet using the same explicit selections.
3. Read graph relationships as analyzer claims. Independently cite selected source spans for important seams. Retain ambiguity and known coverage gaps.
4. Compare shared requirements plus subordinate profiles against separate responsibility contracts. Use purpose, ownership, I/O, permissions, failure/retry and change boundaries, not edge counts or SCC membership as automatic policy.
5. Explain size and legal-structure alternatives. Preserve missing scope and unsupported model features as questions.
6. Let the CAAC skill author explicit pending base/answers and execute existing candidate publication/verification. Graph data supplies evidence, not obligations or acceptance.

Packet references `COLLECTION_HASH:analyzer:0`, `:1` and `:2` quote raw modules, dependencies and snapshot content respectively. Their kind is `analyzer-output`: such a quote supports only what the retained artifact says. It cannot satisfy the existing source requirement for an `aligned` or `suspected-mismatch` finding. Use actual source evidence alongside contract claims; analyzer citations may supplement it. Citation correctness still does not establish semantic truth.

## Limits and compatibility

Each analyzer input is bounded to 1 MiB and all retained document/context/analyzer text shares the existing 8 MiB budget. The adapter admits at most 4,096 module records, 8,192 dependency records and 4,096 manifest entries. At most 65,536 endpoint inventory-match slots may be copied across all dependencies; this is checked before allocation, and over-budget ambiguity is rejected rather than truncated. Existing strict UTF-8, duplicate-key, no-overwrite and reread protections apply. These are resource limits, not feature counts or sizing thresholds.

Without analyzer selections, previous v1/v2 collection bytes remain unchanged. The public Rust `Request` adds optional `crawk`; callers using explicit struct literals must provide it or use defaults. No dependency was added. Legacy artifact compatibility and a real graph-to-draft exercise are recorded separately; this adapter is not a general analyzer runner.

## Why import instead of automatically run

The inspected cached crawk 0.7.1 implementation invokes Cargo metadata from `Analyzer::new()` without automatic offline/no-deps constraints. Both CLI and public library share this behavior. The graph API also uses bare edge labels and provides no exact source spans. Adding its library would not by itself solve provenance, target ambiguity or permitted read scope.

Automated execution against arbitrary working repositories is therefore outside this increment. A later runner needs an explicitly reviewed snapshot, Cargo configuration/offline policy and allowed read/effect boundary. The current adapter can use already authorized retained outputs without granting new execution or network permission.

## Executed bounded example

The [Pigeon graph-to-draft exercise](../reports/crawk-to-draft.md) records real imported evidence, an actual shared-rule/API-profile candidate, independent findings and limits. It is not a benchmark, completeness result or accepted contract.
