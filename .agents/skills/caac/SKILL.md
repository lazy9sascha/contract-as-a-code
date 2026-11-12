---
name: "caac"
description: "Use when writing or reviewing a contract in the Contract as a Code style: clauses with IDs, defined terms, short normative sentences, acceptance cases, and evidence-based completion. Triggers: contract, clause, CAAC, contract-as-code, contract completion, contract review, write contract, review contract."
---

# Contract as a Code (CAAC)

CAAC treats a contract as a program. The contract sits above the implementation
harness. A controller repeats contract work until the declared completion gate
passes.

## When to use

- You need to define permitted behavior before implementation starts.
- You need machine-checkable clauses with IDs and acceptance cases.
- You need evidence-based completion, not an LLM assertion.

## Writing procedure

Follow these steps in order. Do not skip steps.

### Step 0: Size the contract

Before writing, estimate the contract scope using T-shirt sizing:

| Size | Scope | Example |
| --- | --- | --- |
| XS | Single function or expression | One validation rule |
| S | One module or class | One data structure with operations |
| M | One service or component | A queue with send/receive/ack |
| L | One application or subsystem | A full agent adapter |
| XL | One system or platform | Multi-agent coordination |

Size determines the minimum contract unit. A size-XS contract needs 3-5 clauses.
A size-XL contract needs 30+ clauses and multiple files. Do not write a
size-XL contract in one file. Split by module.

### Step 1: Define scope and actors

Write a "Scope and exclusions" section. List what is in scope and what is out.
Name the actor (e.g., `Queue`, `BatchProcessor`). State what it does and does not do.

### Step 2: Define terms and inputs

Define every domain term before its first normative use. Each term gets one
meaning. List inputs, outputs, and error types.

Use short sentences (≤20 words for normative clauses). Define "nonblank" and
other predicates explicitly.

### Step 3: Define operation signatures

List each operation with its inputs and outputs:

```
send(envelope) -> SendResult
receive(recipient, lease) -> Delivery or null
```

### Step 4: Write clauses

Each clause occupies one physical line: `ID | sentence`.

Format: `ID | Actor must/must not/may ...`

Rules:
- ID: uppercase letter groups + 3 digits (e.g., `COO-001`, `BATCH-003`).
- One actor, one obligation per clause.
- ≤20 words per sentence.
- No semicolons.
- Use `@ID` to reference another clause.

Example:
```
COO-001 | Queue must commit an ordinary message before returning success.
COO-002 | Queue must assign id and created_at_ms after accepting an envelope.
```

### Step 5: Define state transitions (if applicable)

List states and transitions. Each transition gets a clause ID.

### Step 6: Write acceptance cases

Create a table: Case | Input | Expected result | Clauses.

Each case must reference at least one clause. Cover branches, boundaries, and
failures.

### Step 7: Run syntax check

```sh
caac check <contract-file>
```

Fix all findings. Repeat until `PASS` (syntax only, semantic review pending).

### Step 8: Resolve open decisions

Find `OPEN:` lines. For each, either:
- Replace with a `DECISION:` line recording the answer.
- Or use `caac interview <file>` to prompt for answers.

```sh
caac interview <contract-file>
```

### Step 9: Review

Review the contract without inferring meaning from implementation code. Record
findings. Fix them. Repeat.

Use `caac watch <file>` to re-check on every save.

### Step 10: Completion gate

A contract is complete only when:
- All clauses have acceptance cases.
- All cases have recorded results.
- No open decisions block a required clause.
- An independent reviewer accepted the exact revision.
- Owner acceptance is recorded (when required).

An LLM statement alone does not establish completion.

## Tools

- `caac check <files>`: declared syntax checks. Reports duplicate IDs, unknown
  references, malformed clauses, word-count violations, open decisions, and files
  without clauses. Exit 0 on success, 1 on findings, 2 on input errors.
- `caac scratch <dir> --title "..."`: create a draft contract scaffold.
- `caac plugin <dir> --title "..." --source <file>`: create a plugin contract
  from source observations.
- `caac migration <dir> --title "..." --source <file>`: create a migration
  contract from source.
- `caac interview <file>`: walk through open decisions, prompt for answers.
- `caac resume <file>`: continue a previous interview.
- `caac watch <files>`: re-run checks on file changes.

## References

- [CONTRACT.md](../../../CONTRACT.md): method rules (CAAC-001 through CAAC-039).
- [contracts/tooling.md](../../../contracts/tooling.md): CLI and checker rules.
- [examples/batch-division.md](../../../examples/batch-division.md): complete example.
- [docs/quickstart.md](../../../docs/quickstart.md): five-minute guide.
