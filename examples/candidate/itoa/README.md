# Replay the itoa formatting draft

This public example proposes one clause from the [upstream itoa example](https://github.com/dtolnay/itoa/blob/9339ca99a43c550b5ee36e2afe524d1c90413461/src/lib.rs#L20-L24). Formatting the unsigned integer `128u64` returns the text `"128"`. The base leaves coverage of other inputs open. The answers file attributes the proposed clause to the pinned source. CAAC verifies the draft and its review. The source digest check below verifies the cited file separately.

Run this block in a shell from the CAAC repository root. It needs Cargo, curl and shasum. The source download requires network access. Each run uses a fresh output directory.

```sh
(
  set -eu
  out=$(mktemp -d)
  curl -fsSL https://raw.githubusercontent.com/dtolnay/itoa/9339ca99a43c550b5ee36e2afe524d1c90413461/src/lib.rs -o "$out/lib.rs"
  (cd "$out" && echo '4289a1fc5c20e8927f4ce27d1d0f085ae0f9fe7b666462dce92bd36466e89829  lib.rs' | shasum -a 256 -c -)

  cargo run --quiet --locked -- candidate apply \
    --base examples/candidate/itoa/draft.json \
    --answers examples/candidate/itoa/answers.json --output "$out/package"
  cargo run --quiet --locked -- candidate verify \
    --candidate "$out/package/candidate.json" --review "$out/package/review.md" \
    --provenance "$out/package/provenance.json" --trace rule-format-128

  cd "$out/package"
  printf '%s\n' \
    'eb4d363ab0f156eb027bca26ea3d4ba7b596213a0cebe77caa0b13054e4579cc  candidate.json' \
    'f1c008d8e32a95377673c0d564744875ca95575c8ce134f41b2464b6851627db  review.md' \
    'd3629a7b1d9f2b44380b3bae7e90413af662fd841373220b127d104d91391f1e  provenance.json' | shasum -a 256 -c -
)
```

The trace identifies `rule-format-128` in Article 1. Each digest check prints `OK`. The candidate retains pending owner acceptance. This replay exercises CAAC authoring and verification; it does not execute itoa.
