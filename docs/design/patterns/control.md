# Control flow

Definition revisions follow the index registry. Every row inherits [the library interface](../pattern-library.md), slot rules, annotation, checks, and maturity.

| ID and purpose | Behavior slots | Semantics and failures | Children/context | Filled instance and expected result | Incomplete instance and expected finding | Specific checks |
| --- | --- | --- | --- | --- | --- | --- |
| FLOW-SEQUENCE: execute finite ordered work | R `steps:List<Op>[0..n]`, `failure:enum(stop,continue-with-results)`. | Start each step after the prior step completes. Empty steps returns []. Stop returns the first error with prior effects retained. Continue returns all tagged results. Terminates when every executed child terminates. | steps 0..n, list order, explicit input/result mappings, exits only declared by each enclosing operation. Statement/operation. | steps=[Done,Done], failure=stop. Both execute once and return [unit,unit]. | Remove failure. Finding `missing-required`, path failure. | C checks mappings/order and retained effects. E checks empty and first failure. |
| FLOW-BRANCH: select one Boolean path | R `condition:Pred`, `then:Op`, `otherwise:enum(child,no-op,reject)`. C(otherwise=child) `else:Op`. | Evaluate condition once. True executes then. False uses the explicit otherwise policy. Reject returns RejectedCondition. Child failure propagates. Terminates when condition and selected child terminate. | condition 1, then 1, else 0..1. Condition precedes one selected branch. Branch-local names remain local. Exits require enclosing permission and target. Statement/operation. | condition=Yes, then=Done, otherwise=no-op. Done executes once. | Remove otherwise. Finding `missing-required`, path otherwise. The false case remains unresolved. | T checks false policy. C checks branch result compatibility and exit/effect joins. |
| FLOW-MATCH: choose one tagged alternative | R `value:Expr<Variant>`, `arms:Map<Tag,Op>[1..n]`, `unmatched:enum(reject,child)`. C(unmatched=child) `fallback:Op`. | Evaluate value once. Execute exactly its matching arm. Missing arm invokes fallback or returns UnmatchedTag. Arm failure propagates. Terminates when selected evaluation and arm terminate. | value 1 then one arm or fallback. Arm sees only its typed payload and declared outer inputs. Explicit enclosing exit permissions. Expression/statement. | value=Some(2), arms={Some:Id}, unmatched=reject. Returns 2. | Remove unmatched. Finding `missing-required`, path unmatched. | S/T check tag coverage and payload typing. C checks arm joins and unreachable alternatives. |
| FLOW-LOOP: repeat bounded or ongoing work | R `kind:enum(collection,counter,while)`, `itemType:Ref<Type>`, `body:Op`, `continuation:enum(bounded,until-cancel)`, `failure:enum(stop,continue)`, `order:enum(ascending,descending)`. C(continuation=bounded) `termination:Predicate`. C(kind=collection) `items:List<itemType>[0..n]`. C(kind=counter) `initial:Int`, `bound:Int`, `step:Int`. C(kind=while) `condition:Pred`, `update:Op`. C(kind=while and continuation=bounded) `maximum:Nat`. C(continuation=until-cancel) `stop:Pred`, `cancel:ExitTarget`, `active:Pred`, `observation:Pred`, `onCancel:Observable`. | Evaluate loop inputs before its first iteration. Collection consumes each item once. Counter applies nonzero step toward bound. While tests condition before body. Bounded mode enforces progress and maximum. Until-cancel is valid only for while and promises per-iteration continuation instead of global termination. Stop requests cancellation at its observation boundary. Acknowledged cancellation returns onCancel with known effects retained. Child errors follow failure policy. | body 1, update at most 1. Iteration binding is local. Ordered iteration, explicit break/continue/cancel targets. Statement/operation. | kind=collection, itemType=Unit, continuation=bounded, items=[], body=Done, termination=remaining count reaches 0, failure=stop, order=ascending. Zero calls. Nonempty and ongoing cases follow below. | Remove termination in bounded mode. Finding `missing-required`, path termination. Ongoing missing-stop case follows below. | T checks bounds/required policy. C checks progress, exits, cancellation, active invariants, and retained effects. |
| FLOW-EXIT: transfer to one declared target | R `kind:enum(return,break,continue)`, `target:ExitTarget`. C(kind=return) `value:TypedValue`. | Return completes its target operation with value. Break ends its target loop. Continue advances that loop through its declared update. No child resumes after the transfer. Invalid or inaccessible targets create a conflict before execution. | No children. Statement within the named operation or loop. Traversal crosses no forbidden cleanup boundary. | kind=break, target=outerLoop@r1 frame1, with explicit enclosing exit permission. Only outerLoop exits. | Remove target inside nested loops. Finding `missing-required`, path target. | N resolves exactly one target. C checks ancestry, result type, and cleanup obligations. |
| FLOW-RECURSION: call with bounded progress | R `base:Pred`, `baseResult:Expr`, `body:Op`, `measure:Expr<Nat>`, `maximumDepth:Nat`, `atBound:enum(error,base-only)`. | Test base first. Base returns baseResult. Recursive calls decrease measure strictly and cannot exceed maximumDepth. Bound failure returns RecursionLimit. Child errors propagate with declared effects retained. Termination follows decreasing Nat and bound under the stated measure relation. | base 1, baseResult 1, body 1. Each recursive frame has separate locals. Return targets its current operation. Operation/expression. | base=Yes, baseResult=0, body=Id, measure=0, maximumDepth=0, atBound=base-only. Returns 0 without recursive call. | Remove measure. Finding `missing-required`, path measure. | C checks decreasing measure, frame scope, base reachability, and bound outcome. |

Counter bounds include the starting value and stop before crossing the bound. The body receives the current counter before update.

Counter step direction agrees with order and reaches or crosses its bound. An incompatible step creates a constraint finding before execution.

Recursion starts at depth zero. The base case remains available at maximumDepth. A nonbase call at that depth returns RecursionLimit.

Collection ordering refers to list indices. While ordering applies to each update sequence, not to an unspecified concurrent schedule.

FLOW-LOOP has revision p3. Its additional filled cases exercise behavior beyond the empty boundary.

FLOW-LOOP also requires R `resultPolicy:enum(discard,collect)`. Normal completion returns Completed(results).

Collect appends each completed iteration's tagged Ok/Error result in iteration order. Discard returns an empty results list while preserving effects.

Stop on child error returns Failed(error, prior results). Bound failure returns LoopLimit(prior results). Cancellation returns its explicitly declared onCancel.

The empty positive fixture supplies resultPolicy=discard and returns Completed([]). No result shape is inferred from the body type.

| Case | Complete assignments in addition to shared optional annotation absence | Expected result and negative counterpart |
| --- | --- | --- |
| LOOP-NONEMPTY | kind=collection, itemType=Nat, continuation=bounded, items=[1,2], body=EmitIndex with item→i, termination=remaining count reaches zero, failure=stop, order=ascending, resultPolicy=collect | Events are Tick(1), then Tick(2). Result is Completed([Ok(unit),Ok(unit)]). Remove order: missing-required at order. |
| LOOP-ONGOING | kind=while, itemType=Nat, continuation=until-cancel, condition=true, body=EmitIndex(i), update=i+1, initial caller binding i=0, failure=stop, order=ascending, resultPolicy=discard, stop=cancel token set, cancel=serviceLoop@f1 frame1, active=i>=0, observation=before each condition, onCancel=Stopped | With token set after Tick(1), the next boundary returns Stopped after Tick(0),Tick(1). No global deadline is implied. Remove stop: missing-required at stop. |

The ongoing fixture's explicit package bindings initialize i=0 and map update output to the next iteration's i. Each tick/update terminates.

Its token is initially false. A fixture caller sets it after the second observed tick. Cancellation targets that active loop occurrence.

A sequence containing ongoing work remains at that child until its declared stop/completion. Later steps do not start during that continuation.

The sequence's conditional termination statement does not convert an ongoing child's contract into a global termination promise.

### Declared runtime-input case

LOOP-SOURCE uses FLOW-LOOP p3 with kind=collection, itemType=Pair, body=EchoPair, continuation=bounded, termination=remaining count reaches zero,
failure=stop, order=ascending, and resultPolicy=collect. Its items key is absent from static bindings, rather than Known Expression<List<Pair>>.

One package binds-supplier relation S-items has from=R-input@r1, to=loop@i1, source=declaration-location(R-input context.parameters.input),
target=DataSlot(loop@i1,items), mapping=identity List<Pair>, phase=before-consumption, and failure=reject InvalidInput before body effects.

The ordering constraint places materialization/validation before loop input consumption. FormalParameter entry materialization precedes this supplier phase.

R-input is T04's contains relation from operation@i1 to loop@i1. Its FormalParameter input is typed List<Pair> and sourced from the parent's argument port.

The parent signature's input Type is List<Pair>. It is not a runtime value slot. Activation selection [parent] identifies the current parent argument.

EchoPair is a pure terminating fixture operation with Pair input, identical Pair output, and no admitted failure value.

At invocation, [] produces Completed([]). Input [Pair(-7,2)] produces Completed([Ok(Pair(-7,2))]) after one EchoPair call.

The source and target share the exact List<Pair> Type reference. No List covariance, reboxing or future-call type inference is assumed.

itemType is static and determines the iteration port's type. Body arguments use an explicit typed mapping from that port to the selected signature.

The selected body's declared output O and failure E determine iteration results Ok(O)/Error(E). Collect returns their ordered list inside Completed.

This is a declared signature derivation, not inference from observed results. An unresolved body signature leaves loop output typing unresolved.

No literal list or Expression<List<Pair>> occupies the canonical items Binding. The complete typed supplier declaration satisfies authoring requiredness.

Remove S-items: missing-required at items. Add Known [] beside S-items: conflicting sources. Set S-items target to retryCount: forbidden supplier.

A late or cyclic same-activation producer remains unavailable. This finite typed example supplies no proof of arbitrary child termination or liveness.
