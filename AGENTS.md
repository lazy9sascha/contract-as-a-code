# Repository checks

| Rule | Enforcement |
| --- | --- |
| A checkout must preserve the same bytes with Git line-ending conversion on or off. | `.gitattributes` sets the default. Run `sh tests/checkout_bytes.sh` on the staged tree, as CI does. |
| Keep current CLI layout support in [the typed authoring guide](docs/candidate-authoring.md#presentation-and-boundaries). Link there from the method's implementation note and the scan procedure's sizing step. | `cargo test --locked --test parse_regressions` checks both references. CI runs the suite with `cargo test --locked`. |
