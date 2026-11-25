# Repository checks

| Rule | Enforcement |
| --- | --- |
| A checkout must preserve the same bytes with Git line-ending conversion on or off. | `.gitattributes` sets the default. Run `sh tests/checkout_bytes.sh` on the staged tree, as CI does. |
