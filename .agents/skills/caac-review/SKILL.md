---
name: caac-review
description: Review a CAAC contract draft or candidate package. Use to check project scope, existing promises, clause meaning, acceptance cases, and execution evidence before owner review.
---

# Review a CAAC contract

Review the exact draft the user supplied. Return a disposition, cited findings, and decisions the owner must make. Keep review read-only unless the user asks for edits.

## Establish scope and authority

- Record the requested scope, draft revision, and source revision. Hash the reviewed files when they include local edits.
- Confirm which documents and attachments the user supplied. Declare the scope as partial when known material is missing, or unknown when you cannot establish the boundary. The MCP result echoes this declaration.
- Look for existing contract documents in the project. Read the README and architecture notes for pointers. `caac discover --directory PROJECT --depth 1` can list Markdown paths when the CLI is available.
- Identify the source of each existing promise. Record unresolved document authority as an owner decision. A filename or discovery result does not assign authority.
- Map affected promises to draft clauses, exclusions the user chose, or open decisions. For a project contract, identify the public behavior missing from a component-only draft. For a component contract, identify the parent promises it supports.
- When the owner supplies an approved obligation list, compare it with the reviewed material. Record missing or partial coverage within that scope. Record coverage as unassessed when the list or scope is unknown.
- If the user wants a new draft or approved changes, use the [CAAC authoring skill](../caac/SKILL.md).

## Check artifacts and meaning

1. For handwritten Markdown, send the selected documents and declared scope to [`caac_lint` through MCP](../../../docs/skill-workflows.md#local-mcp-review) when available. Otherwise run [`caac lint --json CONTRACT.md`](../../../docs/contract-lint.md) on the same selection. For a generated package, run `caac candidate verify --candidate candidate.json --review review.md --provenance provenance.json`. Add `--trace ID` to locate a disputed clause. Use the [candidate authoring guide](../../../docs/candidate-authoring.md) for package layouts and options. Report unavailable checks.
2. Read each disputed clause and its definitions before comparing implementation. Identify the actor, operation, conditions, observable result, and exceptions. Use the [case review procedure](../../../docs/scan-to-draft.md#review-case-expectations-against-the-contract) for detailed case mapping.
3. For each material promise, ask whether the case would fail if that promise broke. Check both success and failure for atomic effects. Check returned flags and stored effects when both are promised. Cite the case or test by name.
4. Compare the documented promise, draft, source, and available execution evidence. Report conflicts as findings or owner decisions. Keep implementation observations separate from requirements.

## Report

Start with `ready for owner review`, `changes needed`, or `insufficient evidence` for the reviewed scope. Give each finding a file and clause or case, its consequence, and the smallest correction. List unresolved scope and policy decisions.

State what passed syntax or artifact verification. List the behavior tests actually run and their exact source revision. Distinguish tests read from tests run. A green syntax check supports a syntax claim. A green test supports its observed case. Owner approval requires its own record.
