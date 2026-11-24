# Specification, contract, and implementation trials

These trials test whether a bounded behavior survives a change of starting artifact. Each forward implementer received a handoff without the source or hidden cases. A code reader received the first implementation without the prior documents. Robot Framework ran the cases and evaluated the results. The agents shared a workspace and followed file-access instructions.

## Specification first and implementation readback

The [Project.txt README at commit `25c213c`](https://github.com/derkork/project.txt-spec/blob/25c213cb3aa5015330ef2b46591fb7f638c0c315/README.md#L215-L223) defines a remaining-effort command. The trial used its prefix, three units, and rules for part order and omission. It froze 18 source-supported cases before drafting the contract. The draft retained six required decisions, with 12 example inputs whose outcomes remained open. It used a local Python adapter and a reversible numeric and lexical profile.

| Trial | Results |
| --- | --- |
| Source specification through CAAC candidate to the first implementation | 18/18 source-supported cases and 5/5 profile cases. The 12 open inputs remained unscored. |
| Code readback through a recovered contract and specification to a new implementation | 12/12 observed-behavior cases. A separate replay also passed the original 18 source-supported cases and five profile cases. |

The code readback captured dictionary order and error results. The initial source did not make these policies part of the upstream format. The recovered contract kept that owner intent open. Candidate verification checked the source-derived package. A separate syntax check passed for the recovered contract.

A second trial started from the [JSON Lines documentation](https://jsonlines.org/). It used a SHA-256 snapshot of the source page and checked 30/30 non-empty byte sequences. Cases covered UTF-8, byte order marks, line endings, blank records, and JSON values. Empty-file validity remained open and had no score. The test adapter exposed a Boolean result.

## Contract first

The [batch division contract](../examples/batch-division.md) led to a separate Python implementation. It passed 24/24 local cases. The cases included rejection of a late invalid item before any callback, callback order, repeated items, and continuation after an item error. A test binding fixed the Python input and result representations.

A separate conversion trial transcribed the five division clauses into the legacy typed schema. Both generated Markdown and generated Rust recovered the same schema fields. The generated Rust trait compiled. Direct parsing of the existing handwritten contract reported unsupported term syntax. Direct extraction from the handwritten Rust implementation reported missing generated metadata. The roundtrip applies to the generated formats in this trial.

## Implementation first

| Pinned source | Scope | First result | After clarification |
| --- | --- | ---: | ---: |
| `cursor/plugins` at `aaeb0ea` | Decision-log shell command in a synthetic filesystem | 16/16 | 16/16 |
| `jingyaogong/minimind` at `cc9629e` | Chat preparation against a local tokenizer stub | 26/33 | 33/33 |

The MiniMind implementer identified an open choice before seeing combined-invalid-input probes. The initial 24 cases all matched. The experiment returned an open-decision marker for conflicting decoder errors. Three targeted probes then exposed the missing precedence. The revised contract specified message order and ordered checks within a message. It fixed all three probes and preserved the initial 24 results. Six additional probes passed after refinement, up from two in the first reconstruction.

## Evidence limits

These are draft and test results for selected boundaries. The complete project grammar, actual model tokenizer, system-wide behavior, and owner approval remain outside their evidence. Executed cases establish the recorded behavior for their inputs. The snapshots, checks, and Robot Framework reports are retained as local experiment artifacts.
