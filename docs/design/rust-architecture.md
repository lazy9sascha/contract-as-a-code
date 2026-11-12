# Rust core, host interfaces, and prototype transition

Revision: `rust-architecture-draft-4`. Status: provisional design. Designer: `the reviewer`. Reviewer: `the designer`.

This document answers T13 r2 and proposes D23–D26. Future interfaces below are specified designs, not implemented modules or accepted decisions.

## Exact sources and limits

| Input | Retained identity and use |
| --- | --- |
| T13 r2 | SHA-256 `b0a44ce45cddcbf91a4d63207826f12b89aefe4745b8e2ec5f086710b66c1e45`. Task obligations and acceptance cases. |
| Current prototype inventory | Manifest SHA-256 `89803ed486abab1291e80f78653a90af42a536052bd67a2aae6562dfbc1d22c5`. Exact source/test/contracts, notes, consumer search, and executed current tests. |
| T06 protocol | SHA-256 `6f976c1d0bb63085000d6456fae374d12093bb5252bfcab6441f0ab7c276022c`. Tool requests, receipts, capability and evidence limits. |
| T07 skill | SHA-256 `71600bbb0e3003d89c940db5f646359e3f6a972e28feef073b502e677d26dee7`. Context, typed data suppliers, proposals, decisions, routing, and host requirements. |
| T11 controller | SHA-256 `324aeffa90eb3bf40e6a78fcfdcd122b80e8894138d1cb5deacaf3ce8393f8d5`. Guarded states/control events, static-or-supplier gate, ModeSession, shared ledger, journal, and recovery. |
| Retained T12 input | boundary-draft-3, SHA-256 `683d751b01f9fe5204f529b24fe927e9fdf06b173e6f3e61691e288daef49b78`. T12 r3 typed ChoiceBasis/RecordRef, required ModeSession members, action eligibility, runtime admission, enforcement, and outcomes. |
| Model draft 5 and catalog draft 6 | Model SHA-256 `6bfc4bd3a430d5481e020c812de1e47b451715da0ef1be787791b4b2835d9b4c`; catalog manifest `1fe4321c706bfb5617ebd668940d4c077db858be03bb1c298843bc5291963654`. Explicit phase and typed-reference contracts. |
| Owner sources and continuation | B13–B16 and retained RUN-001/RUN-002 sources authorize Rust/tool/skill design. They do not accept D23–D26. |

T06/T07/T11/T12 remain input and close-gate dependencies. Their positive provisional reviews do not establish accepted implementation targets.

Repository observation: main has no commits. The inventory manifest identifies the uncommitted bytes. No commit SHA is invented.

Historical Python files were removed before immutable snapshots were available, according to the coordinator's observation. Exact earlier source and file inventory remain unavailable.

Current Rust source and tests cannot establish the missing historical Python behavior. That evidence limit remains unresolved under D26.

## D23 proposal: product and Rust boundary

The contract harness remains primary. It runs the LLM skill and bounded tools until declared contract gates pass, then constrains downstream implementation.

Proposal: retain one Rust library package with logical components. A separate crate graph is unnecessary for this conceptual boundary.

The current `author`, `checker`, and `batch` modules remain prototype observations. The following component names denote future responsibilities only.

| Planned component | Rust core responsibility | External responsibility |
| --- | --- | --- |
| Model and revisions | Typed package/reference validation, lineage, impact inputs, exact digest/reference checks. | Host supplies retained source bytes and owner authority evidence. No missing meaning is inferred. |
| Protocol and registry | Versioned host operations, T06 descriptors, exact result validation, bounded error records. | Host discovers actual builds and supplies implementation/test evidence. |
| Contract controller | T11 creation, assignments, proposals, work graph, state transitions, gate, ModeSession and shared budgets. | LLM host interprets intent and returns proposals/questions under T07. |
| Evidence and persistence port | Required immutable objects, journal commands, expected-sequence transactions, checkpoint validation. | Host store supplies durable atomic persistence and integrity capability. |
| Action eligibility | T12 completion/binding/profile/dependency checks and current contract eligibility. | Implementation harness/host enforces pre-effect interception, current runtime permission, actual dispatch and effect observation. |
| CLI adapter | Decode explicit command inputs and display scoped results or invoke library operations. | It is one adapter. Its command shape does not define the product or requirement semantics. |

The core can reject unsupported semantics or host capabilities. It cannot turn an unavailable checker, store, or enforcement boundary into successful evidence.

The core does not execute arbitrary implementation code, access live accounts, choose a user LLM provider, or grant runtime permission.

RUSTCORE-001 | The Rust core must implement contract-controller rules independently of any CLI adapter's command structure.

RUSTCORE-002 | The skill host must submit proposals and evidence without controlling the Rust core's completion gate.

RUSTCORE-003 | The Rust core must preserve unsupported semantics and host capabilities as explicit gaps.

RUSTCORE-004 | The implementation host must enforce runtime authorization and interception independently of contract eligibility.

RUSTCORE-005 | The project must keep new harness implementation behind T14's accepted conceptual baseline.

The model boundary separates static Binding values, formal parameter declarations, and runtime values materialized for one invocation.

Required authoring slots use a valid static binding or exactly one allowed typed supplier under T02/T03/T04. A known mapping is not itself a value of its output type.

The gate checks the source/type/phase/availability contract and required evidence. It does not require future caller values to exist during authoring.

Runtime adapters validate actual arguments and materialize data before consumption. They cannot parameterize an unresolved owner policy without its declared permission.

Canonical Ref identifies T02 entities. RecordRef identifies retained host records such as Run, ModeSession, Completion, Checkpoint, and ActionBinding.

Neither a runtime record nor contained Binding/Choice data becomes a canonical entity merely because a host operation references it.

RUSTCORE-031 | The core must distinguish static bindings, typed runtime suppliers, and materialized values without coercing expressions into their output types.

## D24 proposal: versioned host operations

The host interface is a typed library boundary with a versioned serialization adapter. The proposal uses T06-compatible UTF-8 JSON for adapters.

T02 still owns canonical contract meaning. Host serialization does not replace the semantic package or select precedence between its sentences and records.

Every operation request has these common fields. The dispatcher validates complete shape and references before committing any state change.

| Field | Required type/cardinality and meaning |
| --- | --- |
| `api_version` | One exact supported host protocol version. Unknown versions reject without silent conversion. |
| `request_id` | One unique ID for this operation intent. Same ID and same bytes recover the prior result. Different bytes conflict. |
| `scope` | One host-only marker for describe_host, new-session marker for creation, or exact ModeSession/Run/ExecutionRun reference for existing state. |
| `expected_sequence` | Exact event tail for existing state, not-created for guarded creation, or NotApplicable for host-only discovery. New stale intents reject before commit. |
| `base_package` | Exact package reference or required-member map; initial marker for creation; NoPackage for an existing package-less run; NotApplicable for host-only discovery. |
| `authority` | One retained source/authority reference covering this operation's scope. It is not a credential or implied runtime grant. |
| `payload` | One operation-tagged record with exact schema version and the operation-specific fields below. Unknown fields reject. |

Request byte digests are retained outside their hashed bytes. No request includes a digest that depends on its own value.

NoPackage contains the exact run ID and event sequence proving its current package is absent. It is not a fabricated empty package.

next_assignment, initial submit_proposal, context answers, control events, checkpoint, and resume admit NoPackage while that exact run lacks its first package.

A session member map can include such NoPackage entries. Initial application replaces that member's absent marker through the ordinary atomic commit.

Package-dependent checks, gate evaluation, and implementation actions reject NoPackage. A run with an existing package cannot reuse it as an initial marker.

Dispatch first validates envelope identity and access to retained results. It then looks for the request ID and exact retained bytes.

A committed identical request returns its stored result before current-tail/base mutation guards. A conflicting request returns request-conflict without replay.

An unknown commit outcome requires authoritative lookup. Only a genuinely new unapplied intent proceeds to current-tail/base/authority validation and commit.

Returning an existing result never reapplies the mutation or replenishes a budget. Its original event sequence remains recorded as historical output.

All operation-specific fields below are required except explicitly absent initial/predecessor values. Missing values produce validation errors or named capability gaps.

| Operation | Required payload | Successful observable | Scope and failure behavior |
| --- | --- | --- | --- |
| `describe_host` | Host/build identity, supported API/store/owner-channel/tool/admission capability evidence refs. | Registered capability report with exact supported versions and limits. | Read-only. Missing evidence prevents support claims, without inventing an adapter. |
| `create_session` | T07 intent/routing/context, required package roles, exact policy/shared-ledger definition, initial package refs or absent markers. | T11 ModeSession, member run IDs, guarded initial states, retained creation event. | No direct complete state. Missing recoverable policy/context pauses; invalid identity/authority rejects creation. |
| `next_assignment` | Current work snapshot and referenced context inputs. | One T07 assignment or exact waiting/blocked/paused outcome. | Reserves shared budget atomically. No assignment when entry, capability, or budget guards fail. |
| `submit_proposal` | T07 handoff, exact changed entities/relations, source map, affected criteria, intended new package identity. | Retained proposed revision; atomically applied revision only when current-base/authority checks pass. | Stale base rejects application. No hidden acceptance or partial pointer update. |
| `record_answer` | Decision/question IDs, exact answer source, answering authority, target revisions, applicability inputs. | Immutable answer record and affected work re-evaluation. | Invalid authority/scope rejects answer applicability. Known values and unanswered questions remain retained. |
| `apply_control_event` | Stable event ID/kind, authoritative source, exact affected run/session/scope, kind-specific change refs, reason, and observed generation. | Journaled accepted/rejected event with guarded state/work effects under T11, preserved ledger, and cancellation/denial intents when required. | Caller supplies no target state. Invalid/revoked authority, conflicting event bytes, or a missing state guard rejects without the requested transition. |
| `prepare_check` | Exact T06 request fields, descriptor/build, units, retained input refs, retry root/predecessor, budget reservation. | Journaled invocation identity and dispatch intent. | No checker is fabricated. Unknown build, unavailable method, stale context, or exhausted budget prevents dispatch. |
| `record_check_result` | Exact invocation ref, retained raw response, T06 caller receipt validation inputs, usage evidence. | Accepted/rejected/stale receipt and scoped assessments, budget accounting, affected work. | T06 result states remain unchanged. A malformed response supplies no successful evidence. |
| `evaluate_gate` | Exact package/criteria, current evidence/reviews, owner acceptance, profile/policy refs. | T11 completion object only when all required conditions satisfy. | Unsatisfied/unresolved conditions return findings. Skill assertions never substitute for evidence. |
| `checkpoint` | Exact context/event-prefix refs, requested snapshot scope, and store capability evidence. Core reads its authoritative records to construct state/work/ledger content. | Immutable checkpoint object plus separate digest-bearing event. | Caller-supplied replacement state or ledger is invalid. Missing objects/inconsistent tail reject validity. No self-hash dependency. |
| `resume` | Exact checkpoint/event prefix, host capabilities, current input/authority comparisons, explicit resume evidence. | Guarded resumed state and retained context with original ledgers/identities. | Unsupported schema or corrupt/missing data prevents claimed safe resumption. No reset through new context. |
| `propose_action_binding` | T12 completion object, ActionBinding, required ModeSession/member refs when applicable, enforcement/profile/effect/dependency refs. | Retained binding proposal and contract-eligibility assessment. | Missing completion authority, required member, scope, or coverage rejects eligibility. No runtime tool call. |
| `preflight_action` | Exact T12 ActionRequest/binding/completion, required session members, current target and enforcement refs. | Contract-eligibility result with exact checked generation and remaining host checks. | It returns no runtime grant. Incomplete required members reject delegation. Host checks current authority at actual admission. |
| `record_action_result` | T12 admission/journal/action identity, typed result, known/unknown effects, conformance evidence. | Immutable action record and dependent-edge continue/stop result. | Nonconforming/unknown required results stop dependent actions. No inferred rollback. |

Every operation response, including host-only discovery and read-only preflight, carries the same versioned result envelope below.

| Response field | Required meaning |
| --- | --- |
| `api_version`, `operation`, `request_id`, `request_digest` | Exact admitted API version and operation/request identity, including retained request-byte digest. One each. |
| `input_refs` | Exact scope, target, authority, and context references, including explicit NoPackage/NotApplicable variants when admitted. |
| `outcome` | One of reported, recorded, rejected, unsupported, indeterminate, or execution-error. |
| `event_sequence` | Committed sequence for recorded results; observed current tail for run-scoped reported results; NotApplicable for host-only discovery. |
| `outputs`, `diagnostics`, `limits` | Declared typed data/artifact references, bounded reasons/targets, and applicable method/coverage/execution limits. Explicit empty collections carry no extra evidence. |

Reported returns data without a requested state mutation. Recorded means the declared operation record committed, not that a semantic check passed.

An unsupported or unreadable request version receives a dispatcher rejection receipt naming the requested version and supported rejection-envelope schema.

That receipt does not impersonate a successfully admitted host operation. No silent version downgrade or state mutation occurs.

Semantic assessments retain T06's passed/failed/unsupported/indeterminate/execution-error states. T12 execution and conformance states remain separate again.

| Error code | Trigger | Observable and recovery |
| --- | --- | --- |
| `invalid-request` | Shape, duplicate/unknown field, invalid value or unresolved required reference. | Reject before the requested state change. Retain its rejection receipt; correct the request under the operation contract. |
| `version-unsupported` | Unsupported API, payload, checkpoint, or host capability version. | Return supported exact versions/limits. No silent downgrade or invented migration. |
| `request-conflict` | Reused request ID has different retained bytes. | Reject new intent. Preserve original result and identify conflicting digests. |
| `stale-sequence` / `stale-base` | Event tail or package identity changed. | No requested mutation. Refresh context and propose against current references. |
| `authority-missing` | Required requirement/action authority absent or out of scope. | Deny the dependent operation. Retain missing authority and permitted independent work. |
| `capability-gap` | Required tool, semantic method, durable store, owner channel, or enforcement tuple unavailable. | Return exact unsupported responsibility and affected guarantees. |
| `integrity-error` | Retained bytes/digests/journal relationships fail. | Stop dependent work. Recover trusted bytes or use T11 failed outcome. |
| `budget-unavailable` | Reservation exceeds shared remaining budget or policy is missing. | Retain paused/unfinished outcome. No fresh budget through a child run. |
| `store-unavailable` / `commit-unknown` | Persistence cannot establish a transaction's final outcome. | No success claim. Query the stable operation key before any mutation replay. |

Diagnostics identify code, operation, request, exact affected targets/paths, observed/expected identity, and bounded reason/evidence. Payload text remains data.

Rejected intents can retain separate immutable receipt/audit evidence. They do not apply the requested package, run-state, or ledger mutation.

### Control-event ingress

ControlEvent is a tagged payload. Its kinds are owner-pause, owner-abort, authority-change, policy-change, capability-change, and host-interruption.

T11 owner-answer uses record_answer, requirement-change uses submit_proposal, and resume uses resume. No checkpoint field replaces those guarded operations.

T11 authoring-policy-change maps to policy-change. Profile/method/capability changes map to capability-change; runtime authorization changes map to scoped authority-change.

Every event has exact source authority, affected scope, event identity, and expected generation. Change events identify old/new policy, grant, descriptor, or governing-input records.

Owner-pause preserves work and the saved guarded state. It never clears consumed or reserved budget. Owner-abort ends an unfinished run without erasing its history.

Authority-change distinguishes requirement/acceptance authority from runtime grants. Runtime revocation stops protected dispatch under T12 without erasing historical authoring completion.

Governing requirement/profile/method/policy changes invalidate affected applicability and reopen work through T11's guards. They do not silently replace the accepted package.

Capability-change records exact new evidence and affected coverage. A newly missing capability cannot become a successful check or reset a failed attempt.

Host-interruption records available checkpoint/journal evidence. If the process died before committing that event, resume uses the last retained in-flight intent instead.

Duplicate event ID and identical bytes recover the original event result. Conflicting bytes or revoked event authority reject without a requested state transition.

Checkpoint only snapshots controller-owned state. It accepts no arbitrary caller state as authority for pause, abort, reopening, resumption, or completion.

Control events can identify an authorized budget-policy change, but actual consumed/reserved amounts remain retained. A new limit does not rewrite prior usage.

RUSTCORE-028 | The core must accept control events only through typed identity, authority, scope, and transition guards.

RUSTCORE-029 | The core must preserve budgets and history when control events pause, abort, invalidate, or resume work.

RUSTCORE-030 | The checkpoint interface must not treat caller-supplied state as transition authority.

RUSTCORE-006 | The adapter must reject unsupported versions, malformed payloads, and unknown fields before any state commit.

RUSTCORE-007 | The core must bind every operation to its declared request, scope, sequence, target, and authority references.

RUSTCORE-008 | The core must reject reused request identities with different retained bytes.

RUSTCORE-009 | The core must recover a committed same-identity operation without applying its state change again.

RUSTCORE-010 | Each operation result must separate protocol outcome, semantic assessment, contract completion, and implementation execution.

RUSTCORE-011 | The adapter must expose exact unsupported capabilities without substituting successful records or checker results.

### Planned skill bundle and loader interface

This is a distribution design, not a generated package or installation. The future bundle uses these relative paths.

| Planned path | Content and authority |
| --- | --- |
| `manifest.json` | Bundle identity/version, every member's digest/media type, exact instruction/schema/mode/catalog refs, compatible host APIs, core build refs, and capability requirements. |
| `SKILL.md` | Discoverable name/description and the exact reviewed T07 entry instructions. Content identifies source authority and routes modes without claiming completion. |
| `modes/scratch.md`, `modes/plugin.md`, `modes/migration.md` | Exact instruction snapshots derived from accepted T08–T10 revisions, with canonical source refs. They add no separate gate. |
| `schemas/context.json`, `schemas/host-api.json` | Versioned adapter schemas generated from accepted T07/T13 interfaces and validated against their fixtures. No schema files exist from this design alone. |

The manifest contains `bundle_id`, `bundle_version`, `members`, `instruction_ref`, `mode_refs`, `catalog_refs`, `context_schema`, `host_schema`, and `host_api_versions`.

It also contains `core_builds`, `capability_requirements`, `entry_contract`, `resume_contract`, `source_authority`, and `upgrade_contract`. Every listed field occurs once.

`members` is a nonempty map from safe relative paths to exact byte digests/media types. It excludes the manifest itself.

The bundle digest covers the complete manifest bytes and stays outside that manifest. No self-hash dependency is introduced.

`host_api_versions` is an explicit nonempty admitted-version set, not a promise that every future minor version is compatible.

`core_builds` identifies supported native executable or in-process library artifacts by exact target/build digest and declared host interface. Current CLI alone cannot satisfy it.

`capability_requirements` distinguishes boot-required core operations from action-specific owner/store/tool/enforcement needs. Missing optional checks remain visible gaps during useful authoring.

The entry contract names create_session, exact T07 context schema, required identity/authority fields, and guarded initial state. It never bypasses creation guards.

The resume contract names resume, exact checkpoint/context schemas, event-prefix checks, original ledger, decision applicability, and request/effect identity preservation.

Loader sequence: validate manifest/member integrity and supported versions, resolve the actual core build, compare host capability evidence, then load the entry instructions.

Source and contract artifacts remain data. Loading an instruction bundle does not execute embedded source text or grant filesystem/network authority.

An upgrade selects a new immutable bundle for new sessions. Existing sessions retain their original bundle refs until an explicit compatibility/conversion record passes.

No loader silently rewrites a checkpoint, changes catalog semantics, resets a budget, or substitutes a new LLM host's remembered context.

| Loader failure | Required result |
| --- | --- |
| Member missing, unsafe path, digest mismatch, or duplicate manifest key. | Reject the bundle before loading instructions or starting its core actions. Retain exact integrity reason. |
| Host/core/API/context schema version outside the manifest's admitted set. | Return version-unsupported. Do not fall back to standalone CLI behavior or silently downgrade. |
| Required core entry capability absent. | No claimed supported session entry. Return the missing interface capability. |
| A later checking, owner-channel, durable-resume, or enforcement capability is absent. | Preserve the dependent action's gap/unfinished outcome. Unrelated known authoring stays subject to its own guards. |
| New bundle cannot preserve an existing checkpoint or instruction/catalog meaning. | Refuse that resume/upgrade, retain the prior bundle and checkpoint, and identify the required explicit conversion evidence. |

RUSTCORE-025 | The loader must validate bundle identities, member digests, supported versions, and required capabilities before dependent entry or resumption.

RUSTCORE-026 | The loader must preserve active bundle and checkpoint identities until an explicit compatible upgrade or conversion passes.

RUSTCORE-027 | The loader must not infer installation, tool availability, runtime authority, or completion from a valid instruction bundle.

## D25 proposal: ownership and durable recovery

| State or record | Semantic owner | Storage/execution owner | Required boundary |
| --- | --- | --- | --- |
| Canonical package, identities, lineage | Author/authorized owner under T02; core validates. | Host content store. | Immutable content, exact manifest refs, accepted revision changes only through validated proposals. |
| Requirement answers and acceptance | Authorized requirement owner. | Host retains exact source; core records applicability. | No host summary or LLM memory replaces the answer. |
| Findings and evidence | Named checker/reviewer with method/coverage. | Host retains bytes; core validates receipts/currentness. | Append records without editing the assessed package or rewriting old failures. |
| Run state, work, gate | Rust contract controller under T11. | Host transaction store. | Expected-tail comparison and durable commit prevent competing proposals from both applying. |
| ModeSession/shared budgets | Rust controller. | Same durable ledger and event transaction. | Member creation cannot reset consumed/reserved budget or retry chains. |
| Checkpoint and journal | Core defines required content and validates. | Host store provides atomic persistence/integrity evidence. | Unsupported durability prevents advertised resumable behavior. |
| Runtime grant/interception | Host policy and current execution authority. | Host/implementation harness at the effect boundary. | Core eligibility is insufficient for live dispatch. |
| Action/effect history | Implementation harness supplies observations. | Host journal; core records dependency/conformance decisions. | Unknown effects remain unknown until declared reconciliation. |

Proposed commit sequence for local contract state:

1. Validate the current request, source references, base revision, event tail, and authority.
2. Retain immutable proposed package/evidence objects and their complete digests.
3. Commit request identity/result, event append, package pointer, proposal-application record, and ledger update in one durable compare-and-append transaction.
4. Return the committed receipt. A failed comparison leaves the previous current pointer and ledger unchanged.

Prepared but uncommitted immutable objects do not establish accepted state. An orphan object can remain retained without being current.

Crash before commit leaves no applied state. Crash after commit recovers its exact recorded result through the stable request ID.

If the store reports an unknown commit outcome, recovery queries that operation key and current tail. It does not blindly repeat the mutation.

Tool dispatch uses a committed intent and reservation before the host invokes the tool. A lost result remains interrupted under T06/T11 retry and budget rules.

No future mutating checker is enabled by this architecture. Its contract first needs explicit effects, atomicity, authorization, and recovery under T06.

Checkpoint compatibility compares exact schema, core semantic version, policy, package, catalog/profile/descriptor identities, event prefix, and host store guarantees.

A schema migration requires an explicit versioned conversion with preserved fields, losses, validation cases, and authority. Unsupported conversion returns version-unsupported.

A migrated checkpoint does not silently change requirements or reuse stale evidence. Changed semantics trigger T02/T11 applicability review.

The checkpoint manifest excludes its own digest and future log tail. A separate committed event stores the checkpoint digest and existing event-prefix identity.

RUSTCORE-012 | The store must atomically commit event append, current revision, application identity, and budget changes against the expected tail.

RUSTCORE-013 | The core must refuse resumable capability when the host cannot establish required durability and recovery guarantees.

RUSTCORE-014 | The core must preserve decision, action, effect, retry, and consumed-budget identities across resume and linked runs.

RUSTCORE-015 | The core must reject unsupported checkpoint conversions instead of silently changing schema or semantic meaning.

RUSTCORE-016 | The core must verify checkpoint integrity and current applicability before issuing a resumed assignment.

### Both harness interfaces

The contract harness is the Rust controller plus T07 host interaction, tool adapters, and durable store. The LLM returns proposals through its assignment boundary.

The implementation harness receives an exact completion object and proposes T12 action bindings. It receives no blanket permission from a Boolean complete flag.

Multi-contract modes require the exact current completed member map. Every action names its governing completion, while all required session prerequisites remain complete.

At protected effect admission, the host validates the current completion generation, binding, target prestate, enforcement tuple, and runtime authorization together.

If it cannot connect those checks to actual admission, required enforcement remains unsupported. A cached core preflight cannot close that gap.

Revocation after admission follows T12's in-flight policy. Known/uncertain effects remain recorded. Contract eligibility never promises retroactive prevention.

RUSTCORE-017 | The implementation host must recheck current binding, enforcement, and runtime authority at protected effect admission.

RUSTCORE-018 | The core must stop dependent action eligibility after nonconforming or unknown required predecessor results.

## Current prototype inventory

The inventory's complete hashes are canonical in the retained manifest. The following table identifies behavior and consumers within that exact snapshot.

| Current path | Observed role | Known local consumer and limit |
| --- | --- | --- |
| `src/lib.rs` | Exports author, batch, checker. | Rust CLI and integration tests. No controller/model/host protocol exports. |
| `src/main.rs`, `src/cli.rs` | UTF-8 argument boundary; scratch/plugin/migration/check; JSON/text syntax output; exit codes. | Binary tests and README commands. No T06 or T13 envelope. |
| `src/author.rs` | Creates draft headings/open decisions; selected-source digests; rejects existing output and invalid inputs. | CLI and CLI tests. Writes files; later I/O failure can leave partial output. No atomic-output claim. |
| `src/checker.rs` | Declared Markdown syntax checks and located diagnostics. | CLI/library callers. No broad semantic pattern/composition/owner checker. |
| `src/batch.rs` | Validates batch input, ordered trait calls, Divide binding, tagged results. | Batch tests and declared example binding. Public external usage remains unknown. |
| `tests/cli.rs` | Sixteen executable tests for draft creation, source baselines, malformed syntax and JSON/exit handling. | Test runner. No skill/controller/enforcement tests. |
| `tests/batch.rs` | Seven tests including validation-before-apply, order/duplicates, boundaries, and subtraction oracle. | Test runner. Finite selected behavior, not exhaustive or language-profile proof. |
| `Cargo.toml`, `Cargo.lock` | Package `caac` 0.1.0-dev, Edition 2024; exact dependency lock. | Cargo. Package version does not accept conceptual 0.1.0. |
| `contracts/tooling.md`, `CONTRACT.md` | Draft prototype interface and method rules. | Current checker test set and designers. Neither is owner-accepted full harness meaning. |
| `examples/batch-division.md` | Draft bounded example and binding. | Rust batch implementation/tests and human review. No universal product defaults. |
| README/instructions | Product navigation, bounded CLI instructions, development rules. | Local agents/users. Conceptual skill documents are design assets, not installed skills. |

Scoped discovery found no `caac/` directory, Python source, installed SKILL.md package, or implemented host adapter in the inspected normal source/design paths.

The inventory excludes hidden/ignored build output, external repositories, private host settings, and machine-wide consumers. External library/CLI consumption remains unknown.

Known library consumers are the local CLI and batch tests. Known binary consumers are CLI integration tests and documented commands.

The consumer-search artifact retains exact matches and scope. A search result is not evidence that no outside consumer exists.

### Executed current Rust evidence

Command `cargo test --offline` returned exit 0 on the retained source. Seven batch and sixteen CLI tests passed: 23 integration tests total.

The log records zero unit and doc tests. Observed toolchain is rustc/cargo 1.99.1, with exact versions in the inventory notes.

This is current Rust tested conformance to selected cases. It is not historical Python replay, Rust 1.85 conformance, a complete semantic checker, or conceptual acceptance.

No new runtime feature or test was written for this design task. No live service, implementation harness, permission boundary, or skill installation was exercised.

## D26 proposal: parity and staged Python removal

Three evidence classes remain separate: historical parity, current Rust contract/test conformance, and future product implementation acceptance.

No immutable historical Python source is available. Therefore no row below claims empirical Python-to-Rust parity or a historical removal gate pass.

The current tooling draft and Rust source identify these capability classes. They are a candidate parity inventory when historical evidence becomes available.

| Capability class | Current Rust disposition and evidence | Historical parity/removal status |
| --- | --- | --- |
| Draft creation for scratch | Retained in author/CLI. Tests cover draft/open decisions, no source baseline, existing-path rejection. | Earlier Python behavior unavailable. Retention is observed Rust behavior, not accepted product semantics. |
| Selected-source plugin/migration scaffolds | Retained source digests and mode sections. Tests cover missing/duplicate sources and symlink-relative baselines. | Historical comparison unavailable. No claim of full active-project adoption or migration execution. |
| Declared syntax checker | Retained code/JSON and sixteen CLI tests cover clause IDs, references, exclusions, malformed input, and semantic-pending output. | Historical checker success is a conceptual task fixture unless source/output is recovered. Syntax-only meaning remains explicit. |
| Batch example and trait binding | Retained Rust implementation with seven current behavior tests. | Historical implementation/API behavior remains unknown. No generalized trait/profile parity claim. |
| Earlier Python package/runtime entry points | Currently absent in scoped discovery. Exact old files, dependency metadata, APIs and consumers unknown. | Removal already occurred before this evidence capture. Historical parity and safe-removal proof remain missing. |
| Future patterns, skill, controller, checkpoint, interception | Planned interfaces only. No Rust implementation or runtime tests. | Not prototype parity. Their implementation remains behind T14 acceptance. |

Observed historical Python capability set is unavailable, rather than an invented empty set. T13-010 remains evidence-limited until D26 resolves that gap.

Any recovered Python source gets exact retained hashes and capability/consumer mapping before replay. Unrecovered behavior stays unknown.

| Removal stage | Required evidence | Current disposition |
| --- | --- | --- |
| Inventory and retain | Exact Python/Rust source, tests, package metadata, commands/docs, and known consumers before removal. | Current Rust retained. Historical Python set unavailable; do not reconstruct it from Rust. |
| Classify capabilities | Retain/change/remove disposition for every observed Python capability, with approved target differences separated from observations. | Candidate classes listed. Exact earlier behavior remains unresolved. |
| Validate retained behavior | Paired inputs/expected outputs/errors/effects on exact old/new builds, plus current contract tests and mismatch records. | Current 23-test Rust run passed. Historical paired replay unavailable. |
| Check consumers | Exact known CLI/library/package consumers, migration/change notices where required, current compatibility results, unknown usage scope. | Local consumers inventoried. External consumers unknown. No universal compatibility claim. |
| Remove eligible components | Parity/current-consumer gates pass for those exact components, required authority exists, affected files/docs/tests are listed. | No historical gate is claimed. Any remaining/recovered component stays retained until its own gate passes. |
| Integrate future harness | Accepted T14 baseline, implemented new interfaces, host/store/enforcement coverage, full required tests. | Separate future track. It is not a prerequisite for a supported prototype-only removal decision. |

Affected known current paths are the six Rust sources, two tests, Cargo files, tooling/method contracts, example, and README/instructions in the manifest.

Affected historical Python paths and package consumers are unknown. No speculative path list, restore, deletion, or acceptance is manufactured.

For already-absent components, record the missing historical evidence as a deviation. Recovery of evidence or an explicit D26 policy decision is needed.

An owner decision can choose the acceptable future evidence scope. It cannot retroactively make unobserved Python executions into observed parity results.

RUSTCORE-019 | The inventory author must bind every prototype claim to exact retained source, test, contract, or consumer evidence.

RUSTCORE-020 | The inventory author must distinguish observed behavior, approved requirements, planned behavior, and unavailable historical evidence.

RUSTCORE-021 | The parity reviewer must preserve mismatches and unknown historical capabilities instead of inferring parity from current Rust tests.

RUSTCORE-022 | The removal reviewer must require applicable parity and current-consumer evidence before approving remaining Python component removal.

RUSTCORE-023 | The removal plan must list affected source, tests, documentation, and known consumers without inventing unavailable historical paths.

RUSTCORE-024 | The project must keep prototype removal gates separate from unimplemented future harness integration gates.

## Conceptual acceptance cases

These cases evaluate the proposed interfaces and inventory boundaries. Only the separately identified current Rust test log is execution evidence.

| Case | Input | Expected result |
| --- | --- | --- |
| RC01 | Historical checker reports syntax success and semantic review pending. | Inventory classifies syntax-only capability. Missing historical bytes remain missing; no harness completeness follows. |
| RC02 | Recovered Python feature lacks accepted product meaning. | Retain its exact observation/parity disposition separately from product requirements. No automatic adoption or deletion. |
| RC03 | Core lacks a required pattern method or host capability. | capability-gap/unsupported with affected targets. Gate and required delegation remain unfinished. |
| RC04 | Architecture offers only a standalone CLI entry point. | Reject architecture against D23. Library/controller/skill/implementation boundaries remain required. |
| RC05 | Prototype tests pass but skill-to-core resume data is omitted. | T13 acceptance fails. Test pass cannot substitute for D24/D25 interfaces. |
| RC06 | Current 23 Rust tests pass. | Record their exact source/toolchain/log as current tested conformance only. No historical parity or T14 acceptance. |
| RC07 | Removal request lacks old-source parity or current consumer evidence. | Gate remains open with exact missing evidence. Already-absent files do not imply an earlier valid gate. |
| RC08 | Exact prototype parity and current-consumer gates hypothetically pass, while future harness remains unbuilt. | Eligible prototype removal can proceed under its own authority. T13/product integration remains incomplete. |
| RC09 | Two proposals target one tail/package. First commits, second arrives with stale sequence. | First result retained. Second rejects before pointer/ledger mutation. |
| RC10 | Crash occurs after atomic application but before host receives result. | Same request ID/bytes returns retained result. No second application or reset budget. |
| RC11 | Checkpoint version unsupported or declared object digest differs. | version-unsupported or integrity-error. No guessed migration or safe-resume claim. |
| RC12 | Core preflight passes, but runtime grant is revoked before effect admission. | Host denies the protected action. Core eligibility supplies no live permission. |
| RC13 | Host cannot prove atomic storage or pre-effect interception. | Relevant resumable/enforcement capability remains unavailable. No implementation guarantee follows from interface design. |
| RC14 | Migration target member completes, transition member is unresolved. | ModeSession aggregate remains incomplete. Both retain one shared budget ledger. No cutover authorization. |
| RC15 | Action outcome is unknown after a lost result. | Preserve action/effect identity and stop required dependents. Reconcile before any permitted replay. |
| RC16 | Host-only describe_host has no run, package, or event tail. | Explicit NotApplicable markers admit discovery without inventing state. Other operations retain their declared scope guards. |
| RC17 | Scratch run has intent and questions but no initial package. | NoPackage admits assignment, initial proposal, context answers, checkpoint, and resume. Package checks/gates/actions reject until a package exists. |
| RC18 | Same committed request returns after its own commit changed the event tail. | Lookup returns the retained identical result before stale-tail/base mutation validation. Different bytes conflict; no second application occurs. |
| RC19 | Bundle member digest is wrong or a path escapes the declared bundle. | Loader rejects before entry. No source instruction or tool invocation is authorized by the damaged package. |
| RC20 | Bundle requires unsupported core API, or only the current CLI is available. | version-unsupported/capability-gap. No fabricated host integration or standalone-product substitution. |
| RC21 | Upgrade omits compatible checkpoint/schema/catalog conversion evidence. | Existing run keeps its old bundle/context/ledger. New instructions do not reset decisions, budget, or effect identity. |
| RC22 | Authorized owner pauses an unfinished run after consuming budget, then resumes under unchanged policy. | apply_control_event journals paused state and saved work. Resume rechecks guards and retains consumed/reserved budget. |
| RC23 | Control event lacks authority, conflicts with an earlier event ID, or supplies a desired complete state. | Reject event/request without that transition. A checkpoint cannot manufacture completion or reset the ledger. |
| RC24 | Runtime grant revokes while a historical contract completion exists. | Protected dispatch stops under T12; historical completion remains immutable. Governing requirement changes instead reopen affected T11 work. |
| RC25 | Host-only discovery or run-scoped preflight returns data without state mutation. | Version/request/digest/input/result fields remain present. Outcome reported uses NotApplicable host sequence or observed run tail, without implying a committed record or semantic pass. |

## Requirement coverage and handoff

| T13 obligation | Design/evidence | Cases |
| --- | --- | --- |
| T13-001 | Exact inventory bundle, scoped discovery, historical Python gap. | RC01, RC02, RC06, RC07 |
| T13-002 | Authority distinctions and parity-class table. | RC01, RC02, RC06 |
| T13-003 | D23 components and both-harness interface. | RC03, RC04, RC12, RC13 |
| T13-004 | D24 all-operation responses/errors/control events; skill manifest/entry/resume; D25 resume data. | RC05, RC09–RC11, RC16–RC25 |
| T13-005 | Proposal operation, controller-owned gate, RUSTCORE-002. | RC05, RC14 |
| T13-006 | Ownership table, guarded control events, and durable transaction sequence. | RC09–RC11, RC14, RC17, RC18, RC22–RC24 |
| T13-007 | Capability-gap results and maturity limits. | RC03, RC13 |
| T13-008 | Typed library operations, bundle/core compatibility, and controller/host responsibilities. | RC03–RC05, RC19–RC21 |
| T13-009 | CLI adapter role and explicit rejection case. | RC04 |
| T13-010 | Candidate historical parity matrix; observed Python set unavailable. | RC01, RC02, RC07. Evidence-limited; no invented closure. |
| T13-011 | Exact current tests and paired-replay requirements. | RC06–RC08 |
| T13-012 | Staged removal/consumer gate. | RC07, RC08 |
| T13-013 | Three distinct evidence classes and current-test limits. | RC01, RC05, RC06 |
| T13-014 | T14 implementation gate and planned-only components. | RC03–RC06 |
| T13-015 | Manifest/source/test log/consumer artifact identities. | RC06, RC07 |
| T13-016 | Prototype removal versus future integration stages. | RC08 |
| T13-017 | Independent prototype-only removal gate. | RC08 |
| T13-018 | Explicit current affected paths and unknown historical set. | RC02, RC07, RC08 |

D23–D26, accepted exact upstream revisions, independent review, and historical inventory/parity/removal gaps remain open.

T14 consumes the interface cases and explicit evidence limits. This design does not call those unresolved gates complete.

## See also

- [contract-model.md](contract-model.md): contract representation.
- [tool-protocol.md](tool-protocol.md): tool interfaces.
- [CONTRACT.md](../../CONTRACT.md): method rules.
