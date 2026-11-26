# Repository checks

Follow the [workspace instructions](../AGENTS.md) when they are available.

| Rule | Enforcement |
| --- | --- |
| A checkout must preserve the same bytes with Git line-ending conversion on or off. | `.gitattributes` sets the default. Run `sh tests/checkout_bytes.sh` on the staged tree, as CI does. |
| Keep current CLI layout support in [the typed authoring guide](docs/candidate-authoring.md#presentation-and-boundaries). Link there from the method's implementation note and the scan procedure's sizing step. | `cargo test --locked --test parse_regressions` checks both references. CI runs the suite with `cargo test --locked`. |
| Use clear names, types, and checks to express code intent. Reserve comments for licenses, public API contracts, and external constraints that code cannot express. | Review new or changed comments with the pstack `no-comments` principle. |
| Keep a human-readable Markdown document and a corresponding JSON representation for each document. Keep a feature map and a Mermaid flowchart for the workflows the document describes. | For [skill workflows](docs/skill-workflows.md), edit the [JSON](docs/skill-workflows.json) and run `python3 scripts/render-skill-workflows.py`. CI checks the generated guide. |
