# Concurrent behavior

Definition revisions follow the index registry. Every row inherits [the library interface](../pattern-library.md), slot rules, annotation, checks, and maturity.

| ID and purpose | Behavior slots | Semantics and failures | Children/context | Filled instance and expected result | Incomplete instance and expected finding | Specific checks |
| --- | --- | --- | --- | --- | --- | --- |
| CON-TASK: start asynchronous bounded or ongoing work | R `body:Op`, `start:enum(immediate,deferred)`, `join:enum(required,detached)`, `failure:enum(propagate,record)`, `cancel:Ref<FAIL-CANCEL>`, `lifetime:Predicate`, `continuation:enum(bounded,until-cancel)`. C(continuation=bounded) `deadline:PositiveDuration`. C(continuation=until-cancel) `active:Pred`, `observation:Pred`, `onCancel:Observable`. C(join=detached) `responsible:Ref<Actor>`. | Immediate starts after registration. Deferred requires an explicit invoke relation. Required join observes one terminal success/error/cancellation before parent completion. Ongoing work has no implicit deadline or global termination promise. Its active invariant holds at observation boundaries until cancellation acknowledgment. Detached work retains a named owner. Known effects survive cancellation. | body 1 concurrent invocation. Only declared inputs/shared resources cross. Return targets task body occurrence. Operation. | body=Done, start=immediate, join=required, failure=propagate, cancel=C, lifetime=through join, continuation=bounded, deadline=1000ms. Parent joins one unit result. | Remove join. Finding `missing-required`, path join. | C checks continuation, lifetime, terminal exclusivity and cancellation. P checks async support. |
| CON-COORDINATE: join tasks and protect shared access | R `tasks:List<Ref<CON-TASK>>[0..n]`, `policy:enum(all,first-success,ordered)`, `shared:Set<Ref<Resource>>[0..n]`, `access:enum(immutable,exclusive,atomic)`, `cancellation:enum(cancel-unneeded,wait-all)`, `continuation:enum(bounded,until-cancel)`, `failure:enum(collect,first-error)`. C(continuation=bounded) `deadline:PositiveDuration`. C(continuation=until-cancel) `stop:Pred`, `observation:Pred`, `onStop:Observable`. C(access=exclusive) `lockOrder:List<Ref<Resource>>[0..n]`. C(access=atomic) `atomicity:Predicate`. | All observes every task. First-success selects earliest success with input-index tie breaking. Ordered exposes results by input index. Shared writes obey declared access. First-error stops dependent work with known effects retained. Empty tasks returns []. Ongoing coordination requests each task's explicit cancellation when stop holds, then applies its declared acknowledgment policy. | tasks 0..n concurrent. Join policy supplies observation order. No implicit shared-name visibility. Cancellation targets exact task occurrences. Operation. | tasks=[], policy=all, shared=[], access=immutable, cancellation=wait-all, continuation=bounded, deadline=1000ms, failure=collect. Returns [] immediately. | Change shared={R}, access=atomic, but omit atomicity. Finding `missing-required`, path atomicity. | C checks races, ordering, synchronization, cancellation, failure fanout and unfinished tasks. P checks runtime capabilities. |

Immutable access prohibits shared writes. Exclusive access requires an acquisition/release path for every shared resource in lockOrder.

Atomic access defines one indivisible state transition at the named boundary. It does not assert a language memory model automatically.

The common model does not select fairness or scheduler guarantees. A required liveness guarantee needs an explicit assumption or a gap.

If first-success receives no success and all tasks terminate, it returns the ordered tagged failures. Its empty case returns [].

CON-TASK and CON-COORDINATE have revision p2. They add the following complete cases.

| Case | Filled assignments | Expected result and incomplete counterpart |
| --- | --- | --- |
| CON-NONEMPTY | tasks=[taskA,taskB], policy=ordered, shared=[], access=immutable, cancellation=wait-all, continuation=bounded, deadline=1000ms, failure=collect | taskA uses Const(1), taskB uses Const(2), both otherwise copy the filled bounded task. If B completes first, exposed results remain [Ok(1),Ok(2)]. No execution order is invented. Remove policy: missing-required at policy. |
| TASK-ONGOING | body=LOOP-ONGOING, start=immediate, join=required, failure=propagate, cancel=C targeting service task occurrence, lifetime=through cancellation acknowledgment, continuation=until-cancel, active=i>=0, observation=each tick boundary, onCancel=TaskStopped | Explicit loop token acknowledgment ends task with TaskStopped after two ticks. Remove observation: missing-required at observation. |
| COORD-ONGOING | tasks=[ongoingTask], policy=all, shared=[], access=immutable, cancellation=wait-all, continuation=until-cancel, failure=collect, stop=service stop token, observation=each coordination boundary, onStop=ServiceStopped | At stop, invoke ongoingTask's C, wait for TaskStopped acknowledgment, then expose ServiceStopped. Remove stop: missing-required at stop. |

C is a complete FAIL-CANCEL fixture: active task ExitTarget, trigger=stop token, mode=cooperative, acknowledgment=target reports TaskStopped,
pending=stop-dependent, effects=retain-and-report, maximumWait=10ms, onFailure=CancellationUnconfirmed. Unconfirmed acknowledgment retains unfinished responsibility.

The bounded task fixtures use C with an inactive trigger during normal completion. Ongoing examples declare the caller's explicit stop-token change.
