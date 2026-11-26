---
name: caac-implement
description: Turn a reviewed CAAC clause into working code or compare two independent implementations. Use when the user asks for contract-first implementation with a mock, a behavior check, and separate implementation and review roles.
---

# Implement a CAAC contract

Use the [implementation workflow and role map](../../../docs/skill-workflows.md#contract-first-implementation). The [JSON data](../../../docs/skill-workflows.json) records the same workflow.

1. Pin the selected clauses, source revision, and observable acceptance cases. Sketch the public types or wire payload and the critical behavior in pseudocode. Use a mock to check the boundary before writing the implementation. Ask the reviewer to challenge missing conditions and failure cases. Record changes before dispatching candidates.
2. Give both implementers the same pinned task, inputs, and required outputs, including the fixes from mock review. Assign each an isolated writable worktree or output directory. Keep each implementation private from the other implementer until both submissions settle.
3. Give the reviewer both submissions and the fixed acceptance checks. Replay the normal case and a case that distinguishes the intended behavior from a plausible wrong result. Identify the exact code reviewed. A self-reported pass is a claim until replay.
4. Select a base that passes the required checks. Record why, which parts from the other candidate are worth carrying over, and any unresolved requirement. Rerun the checks on the integrated revision. Return the code, the decision, and the evidence paths.

When a candidate adds a dependency, check the resolved graph for licenses and advisories. Record what upstream says about support and the evidence behind the version choice.

When independent implementations or review are unavailable, report which roles ran and which conclusions the available evidence supports. Keep approval with the owner. Use [caac](../caac/SKILL.md) when a decision requires a new clause and [caac-review](../caac-review/SKILL.md) for review without implementation.
