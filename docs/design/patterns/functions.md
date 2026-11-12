# Functions

Definition revisions follow the index registry. Every row inherits [the library interface](../pattern-library.md), slot rules, annotation, checks, and maturity.

| ID and purpose | Behavior slots | Semantics and failures | Children/context | Filled instance and expected result | Incomplete instance and expected finding | Specific checks |
| --- | --- | --- | --- | --- | --- | --- |
| FN-OPERATION: expose one callable responsibility | R `actor:Ref<Actor>`, `inputs:Ref<Type>`, `result:Ref<Type>`, `requires:Pred`, `ensures:Pred`, `effects:Set<Effect>[0..n]`, `failures:Ref<Type>`, `body:Op`, `continuation:enum(terminating,ongoing)`, `rejection:Observable`. | Validate inputs and precondition before body effects. Rejection returns the declared observable. Successful and error outcomes satisfy their declared result predicates. Undeclared failures conflict. Selected continuation applies to every admitted call. | body exactly 1, declared argument bindings only, return targets this operation. Operation/module. | actor=A, inputs=Int, result=Int, requires=true, ensures=result=input, effects=[], failures=uninhabited error domain, body=Id, continuation=terminating, termination=single finite return, rejection=InvalidInput. Input 2 returns 2. | Remove ensures. Finding `missing-required`, path ensures. | T checks signatures and rejection domain. C checks pre/postconditions, effects, totality and failure coverage. |
| FN-CALLBACK: call an operation supplied as a value | R `signature:OperationSignature`, `binding:Ref<Operation>`, `arguments:TypedValue`, `invocations:Nat`, `order:enum(sequential,concurrent)`, `failure:enum(stop,collect)`, `lifetime:Predicate`. C(order=concurrent) `coordination:Ref<CON-COORDINATE>`. | Binding accepts the signature's entire admitted domain and preserves its guarantees. Invoke at most invocations times while lifetime holds. Complete all invocations unless stop selects an earlier failure. Zero produces []. Stop retains previous effects and returns first failure. Collect returns ordered tagged results. Termination requires every invocation and coordination to terminate. | bound callback one definition, dynamically invoked 0..invocations. Only explicit arguments cross the boundary. No captured local escapes without lifetime coverage. Operation/expression. | signature=Unit→Unit pure terminating, binding=Done, arguments=unit, invocations=1, order=sequential, failure=stop, lifetime=valid for call. Returns [unit]. | Remove binding. Finding `missing-required`, path binding. Callback name similarity establishes no declaration or guarantee. | N resolves actual binding. C checks substitution, invocation count, captured state, lifetime, and concurrency. |

`OperationSignature` is the FN-OPERATION interface without body. Its owner still supplies complete inputs, guarantees, effects, failures, and termination.

The uninhabited error domain contains no admitted error values. It is not Unit or Null and grants no failure outcome.

FN-OPERATION has revision p2. C(continuation=terminating) `termination:Predicate` establishes a total-return guarantee.

Ongoing instead requires C `stop:ExitTarget`, `active:Pred`, `observation:Pred`, and `onStop:Observable`. It promises observation/stop behavior without global termination.

The existing identity fixture supplies continuation=terminating. Its other fields remain unchanged.

An ongoing fixture copies that operation's authority/signature structure, sets inputs=Unit, result=Stopped, effects={Tick}, body=LOOP-ONGOING,
continuation=ongoing, requires=true, ensures=acknowledged stop returns Stopped, failures={CancellationUnconfirmed}, rejection=InvalidInput,
stop=serviceLoop@f1 frame1, active=i>=0, observation=each tick, and onStop=Stopped. Its finite stop trace returns Stopped after Tick(0),Tick(1).

Remove observation from that fixture: missing-required at observation. A global termination predicate is not required in ongoing mode.

OperationSignature includes the same continuation discriminant and enabled fields. Callback compatibility preserves that selected continuation contract.
