---
title: "CAAC quickstart"
doc_type: "guide"
audience: "users"
status: "maintained"
publication: "review-required"
---

# CAAC quickstart

This guide creates a first contract in five minutes.

## Prerequisites

- Rust toolchain with Cargo.
- A terminal.

## Step 1: Build

```sh
cargo build
```

## Step 2: Create a scratch contract

```sh
cargo run -- scratch /tmp/my-contract --title "Recipient normalization"
```

This creates `/tmp/my-contract/CONTRACT.md` with terms, inputs, results, failures,
clauses, cases, and open decisions.

## Step 3: Inspect the draft

```sh
cat /tmp/my-contract/CONTRACT.md
```

The draft contains `OPEN:` lines. These are unresolved decisions. Fill them in.

## Step 4: Check syntax

```sh
cargo run -- check /tmp/my-contract/CONTRACT.md
```

Expected output: `FAIL: declared syntax only. Semantic review pending.`

This is normal. The checker validates syntax, not meaning.

## Step 5: Fix findings

Edit the contract. Add clause IDs, actors, and normative markers. Re-run check
until it reports no findings.

```sh
cargo run -- check /tmp/my-contract/CONTRACT.md
```

Expected output: `PASS: declared syntax only. Semantic review pending.`

## Step 6: Review

Review the contract without inferring meaning from implementation code. Record
findings. Fix them. Repeat.

## Next steps

- [CONTRACT.md](../CONTRACT.md): method rules.
- [contracts/tooling.md](../contracts/tooling.md): CLI and checker rules.
- [ROADMAP.md](ROADMAP.md): version plan.

- Read [CONTRACT.md](../CONTRACT.md) for method rules.
- Read [examples/batch-division.md](../examples/batch-division.md) for a complete
  example.
- Read [contracts/tooling.md](../contracts/tooling.md) for CLI and checker rules.
