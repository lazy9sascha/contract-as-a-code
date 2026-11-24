# Retained-input authoring comparison

## Scope and method

This bounded CAAC-038 review records two independent draft-authoring attempts, not product acceptance or a statistical experiment. CAAC-039 keeps artifact checks, semantic review and implementation execution separate. The method source is draft 0.1.2 at CAAC revision `2fad9d809edca702101654ee792ba15ab3381e9f`.

Both authors received the same neutral saved assignment, shared evidence and 21 enumerated frozen input files. Only their output-directory parameter differed. Neither received the other candidate or historical drafts. Eleven guides/examples matched committed CAAC files. The project inputs were two Rust implementation files, README, manifest and two licenses from public itoa revision `9339ca99a43c550b5ee36e2afe524d1c90413461`. Tests, benches, fuzz, lockfile and images were excluded. Shared intake/collection/packet creation preceded both authoring attempts; it was not independently repeated by each author.

| Retained input | SHA-256 |
| --- | --- |
| Input manifest | `fa429e5299f6c30ecb4563b041c0c86cbce308751a760a4b70e0996547a8108b` |
| Neutral task | `9cb6ffdc0312db3a8c2601cb7b895f66350a255854a605a5101fde0de17ba213` |
| Saved assignment | `e8cc4fc9db5e7d06124341f65ae24c1a945ce01e16bc829721df127bf35c0d4b` |
| Declared configuration | `964a68aa81318281d39755373cfd62204eb86193bfaac811aabec8d9791540a6` |
| Copied trusted CAAC executable | `7aca20cc645ce7e5f82359f93bfff1b6ec4755bef87aedb14acfc372295e602c` |
| Shared collection | `46d94393b27d8b76b43f73aeccb0c56e9d2c0e1c13009668178487dfe6e40306` |
| Shared packet | `af656c7298e6edb0b1fbea63d3b440d4fb36441f6957ef5d3fad5cbdc516ee9e` |

All 21 input hashes matched after both runs. Exact build invocation and reproducible binary-to-commit correspondence were not established. Model identity/revision, sampling settings, hidden context, host determinants and actual model consumption remain unknown or unobserved. This is verified equality of retained files, not complete equivalent conditions. The assignment was separately retained after collection creation; collection alone does not capture the full task.

Both authors initially tried two shortened manifest/configuration paths at the fixture root. They found the files under `inputs/` and received identical path clarifications. No bytes or semantic task changed. Both encountered a shell heredoc temporary-file restriction and truncated guide output, then used scoped authorized commands or bounded reads. These operational events were retained, not erased. Neither had a candidate validation failure or semantic repair before independent review.

## Initial artifacts and review identity

Run-local author identifiers are A and B. Independent reviewer `R1` is a separate cold semantic-review worker, distinct from both authors. R1 derived coverage checks from frozen sources before inspecting candidates, without historical results or a preselected size. These identifiers distinguish roles in this local exercise; they are not authenticated signatures. The review occurred on 2026-11-13.

| Initial artifact | A | B |
| --- | --- | --- |
| Candidate SHA-256 | `a2607ff6c5f16c3f439b8dbd017e20a244d9b55ec3d6484553c573062850bd1a` | `89377bfe9d837119d6a8af92994830d980b2881722a58612c9b2968220924727` |
| Package revision | `sha256:8cf792f1dafa727bf37e031f988d597671ae8910a238004a51c6eece69796663` | `sha256:5630f354ebddefb6e669ff8cb650f37e83adcaea3f05e71bfbe53165b47a0a83` |
| Markdown SHA-256 | `abf32b964955c0891763b2d5a66bc67f5a00b45e851749dde00572f6aed27b9a` | `fd1ff2971adef25541ebbc5a13191652948deb5a3bbb7b590eef79eb111e5834` |
| Proposed clauses / required questions | 16 / 9 | 22 / 8 |
| First apply and verify | Both exit 0 | Both exit 0 |
| Exact citation checks by R1 | 16 of 16 | 13 of 13 |

R1 independently reran candidate verification on both triples. Both passed unsigned artifact, revision and derived-view consistency. The counts describe outputs, not completeness or quality scores. Both chose S, but differ in proposed meaning. No behavior cases or target code ran. All authority and unresolved requirements remain pending.

## Named coverage checks

Lines refer to frozen `src/lib.rs`, except where another file is named. IDs are local to each candidate; equal numbers do not imply equal meaning.

| Check and source | A initial evidence | B initial evidence | R1 finding / limit |
| --- | --- | --- | --- |
| C01 domain/sealing, 116–216 | ITOA-001,006,007; domain/API definitions | ITOA-001,011–013; domain definition | Full ten fixed-width types and observed pointer widths retained. No invented i32 narrowing. Other targets remain open. |
| C02 decimal/sign/extrema, 159–180,259–409 | Decimal definition; ITOA-001,008 | Decimal definition; ITOA-001,002,008,009 | Both retain canonical decimal and signed minima. Proposed cases are unexecuted. |
| C03 MAX_STR_LEN, 119–122,143–160,191–216 | ITOA-006; case reference values | ITOA-012,013; lengths definition | Both preserve the public constant. A keeps numeric reference data outside the canonical definition. |
| C04 Buffer traits, 62–101 | ITOA-004,005; API definition | ITOA-010; Buffer definition | Construction/default/clone/copy usability survives. Neither promises Clone preserves old bytes. |
| C05 borrow/reuse, 103–113 | ITOA-002,003 | ITOA-003–006 | Mutable receiver and borrowed result survive; no false owned-string replacement. |
| C06 unsupported input, 106,118–132,185–216 | ITOA-007; compile-rejection proposal | ITOA-001,011; compile-rejection proposal | Compile-time restriction is not an invented runtime error. Typed compile-fail modeling is absent. |
| C07 resources, Cargo:5; 41,56–74,98–113 | ITOA-015 no std/heap requirement | ITOA-007,014 also no external I/O | B adds a pending no-I/O policy. Silence in A is not automatically a defect. |
| C08 termination/panic/UB, 97–110,168–179,269–323 | Safety/termination and explicit panic questions | ITOA-008,009 plus safety question | B proposes stronger guarantees. Neither proves them; do not resolve A's questions by copying B. |
| C09 helpers, 230–237,334–471; u128_ext:4–22 | ITOA-011–014, internal definition; algorithm question | ITOA-015–020, internal/safety definitions | Arithmetic relations survive without becoming public APIs. Whether helper identity is binding remains owner policy. |
| C10 safety/block writes, 239–252,277–319,412–443 | ITOA-009,010; fixed-block case | ITOA-006,009,020; fixed-block case | Both case plans demanded an untouched adjacent region beyond their clearly stated canonical obligations. Clarification required. |
| C11 toolchain/features/performance, Cargo:8,13–33; README:9–17,38–48 | Platform/performance questions | ITOA-022; platform/performance questions | No fabricated measured guarantee. Compiler/feature matrix and thresholds remain open. |
| C12 legal/contribution scope, README:52–64 and licenses | ITOA-016; license question | ITOA-021; DEC-LICENSE | A did not explicitly retain contribution-policy scope as open/excluded. B did. No legal authority or compliance is inferred. |
| C13 model/authority, task/schema/method | Required model question; pending | Required model question; pending | No fake full-width typed operations or executable cases. Unsupported meaning remains prose and questions. |

R1 also found weak definition navigation: A's canonical-decimal clause referenced only its domain definition, and both candidates omitted relevant Buffer/internal/length definitions from some references. Definitions existed, so this was not wholesale API loss. Structural verification did not detect the semantic reference weakness.

## Repair disposition

Initial artifacts remain unchanged. Repairs are recorded separately after review; no stronger policy is silently adopted. The block-footprint case must be labeled an additional proposed requirement awaiting adoption, or be backed by an explicit canonical requirement. Contribution policy must remain an observation/open scope, not invented legal authority. Definition links must match the concepts actually used.

Both authors created separate repaired inputs and candidate triples. R1 independently rechecked them. Both apply/verify operations passed, and R1 reran verification successfully. The initial artifacts and all 21 shared inputs remained byte-identical. A's 17 repaired citations and B's 13 retained citations passed exact hash/span checks.

| Repaired artifact | A | B |
| --- | --- | --- |
| Candidate SHA-256 | `1964c260943170edf545a0b53c66b3c9c479841068890c39a805ac73f94b54f9` | `db10060000a0ed53f227ef04b9102a90a1b988951085db32a146da54c56acb01` |
| Package revision | `sha256:fccd71cab8272123a3daf2a34c2499abacdc383985a2f9489be19080884e7254` | `sha256:65b3664bd92177538ea395ce40ea227e8db40b871855c1e3b45ee7176917ed75` |
| Markdown SHA-256 | `7c4c4a94f4bd9691129b1fe511d37887f1c2774dac9257990b329d773f037b21` | `a8bcc5e29eb7bca2972db23b026fb4798261a508268ad8147f8e688b9738c374` |

Only definition-reference edges and A's required license-question wording changed in canonical content. Entity text, actors, modalities, IDs, presentation and authority were preserved. B's questions were unchanged. The stronger block-footprint expectations are now explicitly additional proposals awaiting adoption. A's question preserves contribution policy as an observation with unresolved scope. R1 found no unintended policy adoption and closed these three bounded repair findings, not the remaining owner decisions or product acceptance.

Recheck-only deviation: R1 briefly started and interrupted a metadata-only search to locate this report after its path was omitted. No matches or unrelated content were returned. The coordinator then supplied the exact path. This occurred after authoring and is not hidden as an initial-run input.

## Limits and next decision

The exercise demonstrates real assistant-authored drafts and catches traceability/coverage issues after successful CLI checks. It does not close CAAC-038 across all workflows, TOOLPROTO-003 completeness, an accepted controller, product conformance or owner acceptance. Equal S labels do not establish deterministic sizing. Two attempts support no statistical reliability claim.

Required coverage, private-helper stability, panic/resource guarantees, platform matrices and acceptable evidence remain human-owned decisions. A durable local review archive retains initial outputs and sanitized supporting records; it omits source copies, binaries and raw collection/packet content. It is not a complete reproduction bundle. Public documentation identifies artifacts by hashes rather than private machine paths.
