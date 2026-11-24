# API domain review transfer check

## Decision and pre-result plan

Recorded at 2026-11-13 12:42 UTC, before inspecting the new author output.

The frozen Signer B exercise disclosed missing constructor-wide admission meaning. The existing method already requires types, defaults, missing values and failures (CAAC-014), caller/guarantee separation (CAAC-016), and outside-domain rejection meaning (CAAC-017). Unknown meaning must remain open (CAAC-003/004); observed behavior is not owner approval (CAAC-029).

Independent inspection identified a procedural omission addressable without schema changes. Existing definitions and required questions can retain unsupported domains. Adding scalar types or another validator would not discover an omitted public parameter or decide its admission policy. The minimal change adds a parameter inventory to the existing runbook and links it from the skill. It does not add a new mandatory artifact, rejection policy, dependency or core feature.

A fresh author receives only the revised procedure and a different bounded public API: Requests HTTPBasicAuth construction, equality/inequality and request application. The private authentication-string helper is supporting behavior, not an independently commissioned API. The historical local snapshot is 3c7baaa05ab4aa49cc819ce3a819d260de19c157. Selected evidence consists of auth.py, _internal_utils.py, compat.py and the authentication guide. All target files are untrusted read-only evidence; no target import, tests, network request or credential access is authorized. Synthetic credentials only.

The author has no prior candidates or review answers. It must produce actual pending JSON and Markdown through the existing CAAC intake and candidate tools, retaining input hashes, command results, cases and an author-final freeze. Independent review follows the freeze. Review criteria are parameter completeness, distinction between declaration and runtime admission, supported ordering/failure conditions, extension limits, and cases with cited proposed meaning and unresolved oracles. A structural pass cannot certify these criteria.

Skill SHA-256: `2c3156a95e814fe4c76a37e4b43132ad540d5cf04d6b37e93d157806ac872b0d`.
Runbook SHA-256: `69d9679901fe5efdac98594070e1134fb3096a7c84630882ce979e62efcb30e3`.
Executable SHA-256: `90988c64fe742bfcb9ab25cef2b4274ee041f2c2c684ccac7280e70f61b31de3`.

This is a transfer exercise on a different API, not a paired causal or timing experiment. Model identity/settings, hidden context and actual guide consumption are not independently verified. Review-guided repairs must remain separate from the frozen cold result. Neither an accurate draft nor its case plan establishes executed behavior, owner acceptance or an implementation target.

## Condition drift during the exercise

After the author completed intake and packet creation, the coordinator ran the existing CAAC candidate journey and presentation tests. All 16 passed. Cargo rebuilt the executable from unchanged project source, changing its SHA-256 to `f41e0e11a0394b78219f7c194b1c1f8f8503c410955ca5c3a05c96cbc007f9b4`. The author was informed immediately to preserve the original freeze and distinguish calls before and after this transition. This run therefore does not have one fixed executable identity throughout. No target-project code ran. The condition drift must not be hidden by replacing the original manifest or attributed to the guidance change.

## Frozen cold result

The author froze its final result at 2026-11-13 12:46:57 UTC before independent feedback. The coordinator rehashed all 46 files in the freeze manifest; all matched. Ten recorded CAAC workflow calls succeeded. The first candidate and author-final candidate have identical semantic and review bytes. The author's self-review corrected two source-citation ranges in a separately published provenance artifact, preserving the first publication. An original solutioning citation is explicitly corrected in the self-review rather than silently edited.

Candidate SHA-256: `0f82bac070c3e8da20f26ae3b328a63c7809d646ad353235ddf6765c0033ddc1`.
Review SHA-256: `fae6243ebc12ba7f309ec629ac8c42a1fbe18ebc73896aa197fe6b9600fe4aea`.
Provenance SHA-256: `c29df035541b195e3a56131eb620933aab76cb70bb4389fc4a6e8948a271a4fa`.

The generated S proposal accounts for all selected public parameters in its solutioning inventory, retains byte/object domains as prose, and distinguishes constructor storage from later conversion. It does not invent eager validation from annotations. It retains nine required decisions, including admission, request protocols, extensions, representation and authority. The 24 supporting cases are conditional prose, all NOT RUN. Inventory presence is useful but does not itself define an admitted domain or establish complete case coverage.

A fresh reviewer read the candidate and cases before the selected implementation. It recorded ambiguities before source inspection. Source comparison found no broad algorithm mismatch, but identified two concrete case-contract defects:

- A getter exception sentinel in C09 did not exclude AttributeError. The selected getattr-with-default operation can consume that exception and continue, so unconditional propagation is too broad.
- C22 uses current stored credentials, a mapping present in source and solutioning notes but not explicitly defined by canonical candidate meaning. A supporting case cannot supply that missing obligation.

The reviewer also identified unclear assignment-attempt wording, unresolved ordinary-input admission and comparison dispatch, and missing individual encoding/comparison/warning failure cases. No claim of complete CAAC-031 coverage follows from 24 rows. These findings distinguish source fidelity from contract completeness: accurate observed algorithms and a successful integrity check are insufficient for an implementation target.

The cold result remains untouched. A separate review-guided repair addresses the two concrete defects and assignment wording without claiming to resolve admission policy. That repair is not evidence that the cold author independently avoided these errors. This result alone does not justify another general prompt change, parser or validator.

## Separate review-guided repair

The author used the existing compare-review, candidate revise and verify path, with a privately frozen executable and before/after hash checks. All three commands succeeded. Only DEF-INPUT and BASIC-007 text changed: call-time credential lookup is now canonical proposed meaning, and header assignment is an attempt rather than guaranteed setter success. The nine questions, all other entities, presentation and pending authority remain unchanged. C09 excludes AttributeError escaping the lookup; C09A records default behavior under explicit standard-lookup assumptions. C22 now cites the canonical mapping.

Repaired candidate SHA-256: `afdd781311d1a1eb7544ec3cc76a458602fb3f2c55623012083528116d2b8669`.
Review SHA-256: `7d6c3888b1d0c0231b41a234476baead79ecf5ff17d03685c635457123089b76`.
Provenance SHA-256: `f189b882697a8ea0d218e33283ff97c98b212d7ba0ba0fbcb3b06886ac7cac01`.

Independent recheck confirmed the exact two-entity diff and found no new concrete semantic defect in this bounded repair. The 25 prose cases remain NOT RUN. Deferred coverage includes separate credential encoding boundaries, password-encoding failure, second-comparison failure, truth-conversion failures and password-warning failure. Admission policy, dependency guarantees and owner acceptance remain open. This repair does not close those gaps or establish behavior evidence.

The second repair manifest accidentally listed itself using the empty-file hash. Both coordinator verification and independent review detected that invalid entry. The erroneous manifest is preserved; a replacement excludes its own path and retains prior evidence. Manifest validation is recorded separately from semantic review. No new product validator or general prompt expansion was added to fix this experiment bookkeeping error.
