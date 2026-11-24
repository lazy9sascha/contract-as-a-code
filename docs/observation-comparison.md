# Compare behavior observations

Use `compare-observations` to compare two runs over the same inputs and reference implementation. Supply records from the harness that ran the original and reconstructed implementations.

```sh
caac compare-observations --before before.json --after after.json
```

Each input uses this shape.

```json
{
  "format": "caac-observation-run-v1",
  "oracle": {"id": "copy-command", "revision": "pinned-commit"},
  "scope": {"fixture_revision": "example-v1", "runtime": "Python 3.9.6"},
  "cases": [
    {
      "id": "copy-one-file",
      "input": {"argv": ["input", "output"], "files": {"input/a.txt": "a"}},
      "expected": {"exit": 0, "stdout": "", "tree": ["output/a.txt"]},
      "actual": {"exit": 0, "stdout": "", "tree": []}
    }
  ]
}
```

Set `oracle.revision` to the exact revision used to obtain the expected results. Use `scope` to record the runtime, fixture identity and comparison limits. The command requires equal oracle data, scope, case IDs, inputs and expected values across both runs. It rejects duplicate keys and case IDs.

Object member order, case order and string escape spelling can differ. Array order matters. Numbers compare by their exact JSON spelling. For example, `1` and `1.0` differ.

The report includes both input hashes and four counts. A `fixed` case starts different and becomes equal to its expected value. A `regressed` case starts equal and becomes different. The `still-matches` and `still-differs` counts record cases whose match status persists.

Each case reports JSON Pointer paths for differences. A side with `present: false` indicates a missing field. A present side retains its JSON value as text in `raw_json`.

Exit status `0` means the command produced a comparison, including when it reports regressions. Inspect the counts to decide whether a change meets your criteria. Invalid input or a limit error exits `2` with empty stdout. The command compares caller-supplied observations. Evidence of execution, oracle provenance and acceptance remain the responsibility of the harness and reviewer.

Each input is limited to 8 MiB, 100,000 JSON nodes and a nesting depth of 64. The comparison allows at most 100,000 differences. It also checks a 64 MiB construction budget before retaining differences and a 64 MiB limit on the encoded report.
