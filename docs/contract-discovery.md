# Experimental contract path discovery

`caac discover` inventories Markdown path candidates in explicitly selected directories. It does not read file bodies, recognize contract meaning, establish a root contract or choose files for collection.

```sh
caac discover --directory . --directory contracts
caac discover --directory path/to/contracts --depth 3
```

Inspect the JSON suggestions locally, then pass selected paths explicitly to [contract collection](contract-collection.md). There is no automatic piping of every result into `collect`. For example, select `contracts/tooling.md` only after checking that it belongs to the intended task. A discovered README or guide is not promoted into contractual authority.

With `--directory contracts`, for a layout such as `contracts/README.md`, `contracts/common/queue.md` and `contracts/local/codex/session.md`, depth zero finds only the first file. Depth one reaches `common`, while depth two reaches `local/codex`. Inspect `frontier` before claiming coverage, and do not classify the README as a project contract. Larger or differently structured repositories can exceed this bounded profile.

## Scope

The default depth is zero: only entries directly inside selected directories are inspected. Optional `--depth 1` through `--depth 4` admit bounded traversal of visible child directories. The operation suggests regular `.md` and `.markdown` files, not their contents. Hidden entries, symbolic links and special files are omitted. Child directories beyond the depth limit are reported as untraversed paths. Explicitly select those paths or choose a larger supported depth only within the authorized scope. The tool does not traverse document links. Names and metadata can themselves be private. No generic private-content detector or sharing permission is provided.

A filename match is only a suggestion. `CONTRACT.md` is not proof of a project-level contract, and a navigation-like filename is only a hint. Root establishment remains unassessed by metadata inventory. No candidates means none matched this bounded selection policy, not that the repository has no contracts. A successful discovery is not completion of the roadmap's project-contract scan or an approval verdict.

The CLI and public Rust core share the same operation. Selected final-component directory symlinks and paths with hidden directory components are rejected. Parent-directory aliases can resolve normally; this is not a no-follow policy for every path component. No Git process, source execution, content parser or new dependency is required. The result is a local snapshot of directory metadata, not an atomic filesystem snapshot. Files can change after discovery. Collection must freshly validate the explicitly selected files and capture exact bytes and hashes.

## Limits and handoff

Use only directories and traversal depths authorized for this task. A deeper scan can expose additional private filenames even though it reads no file bodies. The tool rejects unsupported paths, unreadable scopes and exhausted budgets rather than emitting a successful partial inventory. Omitted entries are reported as omissions; do not describe the result as an exhaustive repository search. JSON escaping protects displayed path strings; consumers must still treat names as data, never shell commands.

The initial selected roots are limited to 16, total inspected entries to 4,096, suggestions to 128, and serialized output to 32 MiB. These are global work/output limits, not an I/O deadline. Canonical directory identities are deduplicated. Explicit omissions and untraversed frontiers remain coverage limits even on success.

The [discovery tests](../tests/contract_discovery.rs) exercise bounded inventory and an explicit selection-to-collection handoff. They do not prove contract classification, semantic comparison or root authority. Continue with the core skill's evidence-guided analysis and [pending candidate workflow](candidate-authoring.md), retaining unknowns and separate human review.
