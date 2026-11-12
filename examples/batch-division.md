# Batch division contract

Revision: draft-1. Status: draft. Owner acceptance: pending.

This example defines a pure operation. It exposes a result and no external effects. It uses no retries, persistent state, recursion, or concurrent work.

## Terms and inputs

The actor is `BatchProcessor`. Its operation is `run(items)`.

An integer is a mathematical whole number. A `Pair` contains exactly two named fields: `numerator` and `denominator`.

Each field contains an integer from -1000 through 1000, inclusive. A Boolean, missing field, extra field, or non-integer value makes a pair invalid.

`items` is a finite ordered list containing zero through 100 pairs, inclusive. An absent input, non-list input, or invalid pair makes the entire input invalid.

The caller supplies an immutable input value. Text decoding, concurrent mutation, process termination, and resource exhaustion are outside this example's execution model.

Equality compares integer values. Pair equality compares both named fields. List equality compares length and each value at the same position.

The output is a tagged value. Its permitted shapes are:

| Shape | Meaning |
| --- | --- |
| `InvalidInput` | The entire input is invalid. This tag has no fields. |
| `Completed(results)` | The batch completed. The tag contains exactly one field, `results`. |
| `Ok(value)` | One item succeeded. The tag contains exactly one integer field, `value`. |
| `Error(ZeroDivisor)` | One item failed. The tag contains exactly one field with the constant `ZeroDivisor`. |

`results` is an ordered list of `Ok` or `Error` values. These shapes describe values, not a wire encoding.

## Abstract base: Batch, revision draft-1

The only extension point is `apply(pair)`. It accepts every valid `Pair`. It returns exactly one `Ok(integer)` or `Error(ZeroDivisor)`.

BATCH-001 | `BatchProcessor` must return `InvalidInput` if the entire input is invalid.

BATCH-002 | `BatchProcessor` must not call `apply` when the entire input is invalid.

BATCH-003 | For valid input, `BatchProcessor` must call `apply` exactly once per item, in ascending list-index order.

BATCH-004 | `BatchProcessor` must complete each `apply` call before starting the next call.

BATCH-005 | `BatchProcessor` must append each `apply` result to `results` in the same order as the input items.

BATCH-006 | `BatchProcessor` must continue with the next item after an `Error` result.

BATCH-007 | After processing every item, `BatchProcessor` must return `Completed(results)`.

BATCH-008 | For empty input, `BatchProcessor` must return `Completed([])` without calling `apply`.

BATCH-009 | `BatchProcessor` must not change the input or produce external effects.

BATCH-010 | For valid input, `BatchProcessor` must initialize `results` to `[]` before processing any item.

BATCH-011 | `BatchProcessor` must terminate each `apply` call for every valid pair.

BATCH-012 | `BatchProcessor` must return exactly one declared result from each `apply` call.

BATCH-013 | The extension author must replace behavior only within `apply`.

BATCH-014 | The extension author must define `apply` for every valid pair.

The extension author writes the concrete profile. A different input domain requires a different base contract.

## Concrete extension: Divide, revision draft-1

`Divide` inherits @BATCH-001 through @BATCH-014 from `Batch, revision draft-1` in this file. It binds `apply` to integer division.

`abs(x)` is the nonnegative magnitude of `x`. `floor(x)` is the greatest integer less than or equal to `x`.

For a nonzero denominator, let `magnitude = floor(abs(numerator) / abs(denominator))`. This expression uses exact mathematical division.

Opposite signs means one value is negative and the other is positive. Zero has neither sign.

DIV-001 | If the denominator is zero, `BatchProcessor` must return `Error(ZeroDivisor)` from `apply`.

DIV-002 | Otherwise, if the numerator and denominator have opposite signs, `BatchProcessor` must return `Ok(-magnitude)` from `apply`.

DIV-003 | Otherwise, `BatchProcessor` must return `Ok(magnitude)` from `apply`.

DIV-004 | `BatchProcessor` must not evaluate `magnitude` when the denominator is zero.

DIV-005 | `BatchProcessor` must evaluate @DIV-001 through @DIV-003 in order and execute exactly one branch.

`Error(ZeroDivisor)` is an item result, not an exception. @BATCH-012 requires a declared result from each call.

## Contract cases

In this table, `(a, b)` means a pair with numerator `a` and denominator `b`. Each case uses the concrete `Divide` profile.

These cases are expected results derived from the contract. They are not records of executed implementation tests.

| Case | Input | Expected result | Clauses |
| --- | --- | --- | --- |
| Empty | `[]` | `Completed([])`, zero `apply` calls | @BATCH-008 |
| Positive fraction | `[(7, 2)]` | `Completed([Ok(3)])` | @BATCH-003, @BATCH-005, @BATCH-007, @BATCH-010, @DIV-003 |
| Negative numerator | `[(-7, 2)]` | `Completed([Ok(-3)])` | @BATCH-007, @DIV-002 |
| Negative denominator | `[(7, -2)]` | `Completed([Ok(-3)])` | @BATCH-007, @DIV-002 |
| Both negative | `[(-7, -2)]` | `Completed([Ok(3)])` | @BATCH-007, @DIV-003 |
| Zero numerator | `[(0, -2)]` | `Completed([Ok(0)])` | @BATCH-007, @DIV-003 |
| Continue after error | `[(1, 0), (9, 2)]` | `Completed([Error(ZeroDivisor), Ok(4)])` | @BATCH-004, @BATCH-005, @BATCH-006, @BATCH-007, @DIV-001, @DIV-003 |
| Duplicate pairs | `[(4, 2), (4, 2)]` | `Completed([Ok(2), Ok(2)])`, two `apply` calls | @BATCH-003, @BATCH-005, @BATCH-007, @BATCH-010, @DIV-003 |
| Invalid later item | `[(4, 2), (1001, 1)]` | `InvalidInput`, zero `apply` calls | @BATCH-001, @BATCH-002 |
| Lower bound | `[(-1000, 1)]` | `Completed([Ok(-1000)])` | @BATCH-007, @DIV-002 |
| Upper bound | `[(1000, 1)]` | `Completed([Ok(1000)])` | @BATCH-007, @DIV-003 |
| Maximum length | 100 copies of `(1, 1)` | `Completed` with 100 `Ok(1)` results | @BATCH-003, @BATCH-005, @BATCH-007, @BATCH-010, @DIV-003 |
| Excess length | 101 copies of `(1, 1)` | `InvalidInput`, zero `apply` calls | @BATCH-001, @BATCH-002 |
| Wrong field type | `[{numerator: true, denominator: 1}]` | `InvalidInput`, zero `apply` calls | @BATCH-001, @BATCH-002 |
| Wrong shape | Missing input, a scalar, a missing field, or an extra field | `InvalidInput`, zero `apply` calls for each case | @BATCH-001, @BATCH-002 |

## Review notes (non-normative)

The cases expose expected behavior for discussion. They do not prove that every interpretation or boundary is correct.

The owner must accept the input limits, division rule, and continuation rule before implementation. These are example design choices, not universal CAAC rules.

A reviewer must check the contract without inferring missing meaning from an implementation. The [Rust implementation](../src/batch.rs) follows the [prototype binding](../contracts/tooling.md).

## See also

- [Method contract](../CONTRACT.md): method rules.
- [Tooling contract](../contracts/tooling.md): CLI and checker rules.
