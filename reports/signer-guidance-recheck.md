# Signer authoring guidance recheck

## Evaluation plan recorded before the new candidate

At 2026-11-13 12:26:06 UTC, the coordinator recorded this plan while the new author was working and before inspecting any new candidate. The goal is to test the revised skill on the same bounded Signer API and historical snapshot, not to manufacture a successful result.

The author receives current guides and the source/document selection, but no previous candidate, findings, case answers or validation/audit reports. It may self-review normally. Preserve its first publication and its author-final artifact separately from any later reviewer-guided repair. Assess the author-final result before sending corrections.

Hold the selected source and document identities, API scope and CAAC executable bytes constant where verified. The selected snapshot is the same historical itsdangerous revision `672971d66a2ef9f85151e53283113f33d642dabd`; three source files and three RST documents are specified. Optional README/license context is permitted in both tasks. The executable SHA-256 is `90988c64fe742bfcb9ab25cef2b4274ee041f2c2c684ccac7280e70f61b31de3`. Current skill SHA-256 is `6099584b37d1ff86000fcdee6e23e2a840137aa3b63c6aabfafb1d66ce5c6f5f`; runbook SHA-256 is `6353c88fcc144f7efe5d332de00ff242a4b1423c0387f9a79c6ab891e2fefece`.

Task wording is not byte-identical between runs: the second explicitly excludes prior audit reports and requests a frozen author-final artifact. Skill/runbook bytes intentionally changed. Model sampling, hidden context and actual consumption cannot be equated. This is one blind-to-prior-output recheck, not a controlled performance experiment or statistical causal estimate.

## Prespecified review questions

An independent reviewer reads the new candidate before source, checks these points without seeing old outcomes, and records concrete evidence rather than a score based on clause count:

- Do literal ordering and substring matching preserve the selected source condition?
- Are steps conditioned on successful prerequisites, including combined-invalid constructor inputs?
- Are base methods, replaceable algorithms and overrides distinguished without silently narrowing scope?
- Are digest/none derivation, callback failures and validation results correctly bounded?
- Does the draft preserve full data domains, profile/configuration uncertainty and owner decisions?
- Is there a real case plan with named inputs, clause references, expected observables and unresolved oracles, rather than only a coverage list?
- Are exact quotations current and complete when labeled exact?
- Did source/document intake actually lead to pending JSON/Markdown, with root absence and observation limits retained?

After this blind review, compare the first run's v3 pre-feedback candidate (after its alphabet self-correction; v2 is retained separately) with the new author's frozen result. Separately report self-review, structural failures, reviewer findings and review-guided repairs. No target library code, tests, imports or scripts may run. No cryptographic security acceptance follows.

## Result

The B author-final froze at 2026-11-13 12:30:48 UTC before receiving any independent findings. The coordinator separately rehashed all 47 frozen files against the manifest; they match. The blind reviewer checked five selected artifact identities. The six selected source/document hashes and executable hash match A; guide bytes differ as planned. B first publication and author-final use the same candidate. A minor narrative article-count typo is recorded in self-review rather than silently changing the original.

B candidate SHA-256: `3a1d93d157fa4b7815bb84056d1d8ee9027780c91e52ad90514ca8fd0173b08d`. Review SHA-256: `94dfd49cee8cd64d1771a320293a1309a53894c79c79e79eae12029376f9dafb`. Package revision: `sha256:abd78831446560e7fa44ef76de602d993e4870b062755052e1c84a4ed2c19751`.

A fresh reviewer read B before source and did not see earlier outcomes or communicate with its author. It found no demonstrated incorrect base-path outcome in the inspected construction, derivation, signing, verification or validation descriptions. That is a bounded static review result, not exhaustive semantic acceptance.

| Meaningful criterion | A v3 before review-guided repairs | Frozen B before independent feedback |
| --- | --- | --- |
| Ordered separator literal | Wrong in first publication v2; self-corrected in v3. | Correct source expression and ordered expansion retained initially, with substring boundary cases. |
| Early failures and prerequisites | Unconditional split/key-attempt clauses and a digest-use generalization needed correction. | Definitions and referenced extension rules preserve early failure, none derivation and downstream prerequisites. |
| Overrides and dependency effects | Base guarantees read too broadly; callback assumptions needed explicit limits. | Base/replacement behavior and unknown dependency contracts explicitly distinguished without universal security guarantees. |
| Case-plan presence | Coverage list was not a concrete case plan at review time. | 26 named conditional prose cases exist, but some failure/replacement branches are still missing. |
| Exact selected evidence | Six source/document identities were checked against the retained snapshot. | The same six identities match; inspected exact source expressions are accurate. |
| End-to-end artifact publication | Initial duplicate-answer rejection, then successful publication and checks. | Initial apply and all 11 recorded CAAC calls succeeded. |

Requested scope and selected evidence were shared, but delivered boundaries differed: B directly contracts `derive_key`, `get_signature` and `verify_signature`, which A treated as supporting behavior rather than independent helper contracts. Both cover `secret_key`. Case coverage and authoring effort therefore are not like-for-like completeness measures. This scope difference is retained as an experimental confound, not silently treated as broader completion.

Both runs proposed S, but that is not evidence of calibrated or deterministic sizing. Different entity/case counts are not completeness scores. The reviewed B result avoids several specific errors observed in A; one pair with changed task wording and unobserved model conditions cannot establish that the guidance caused the difference or will reliably prevent recurrence.

## Remaining findings, not hidden successes

The blind reviewer identified incomplete constructor domain/admission specifications for salt, separator, derivation and callable/algorithm parameters; missing individual cases for decode BaseException, base SigningAlgorithm failure, generation callback failure and get_signature replacement; and unresolved exact error-message/chaining compatibility. Existing questions disclose substantial extension/oracle/model debt, but disclosure does not fill every gap.

Definitions contain operative conditions beyond the short clause sentences. Counting twelve short clause texts does not establish whole-contract language compliance. B remains a pending, incomplete draft that is not an implementation target with eight required decisions and no executed case evidence. Its original frozen artifacts are left unchanged; no reviewer-guided B repair is mixed into this comparison.

No additional prompt or core implementation was added from this result. The useful next investigation is explicit constructor-domain and failure/replacement case coverage, not another hierarchy level or a claim of automatic semantic correctness.

## Effort observations

B records eleven successful CAAC calls from the first intake command at 12:26:35 to final trace/freeze at 12:30:48, about 4 minutes 13 seconds. Earlier reading and setup time is unknown. A's comparable recorded command endpoints through its v3 self-correction span about 3 minutes 31 seconds, with thirteen calls including one rejected apply. These exclude different work and produce different case coverage. Neither interval measures developer labor or proves speedup; no timed human baseline or statistically meaningful sample exists.
