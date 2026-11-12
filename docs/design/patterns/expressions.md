# Expressions

Definition revisions follow the index registry. Every row inherits [the library interface](../pattern-library.md), slot rules, annotation, checks, and maturity.

| ID and purpose | Behavior slots | Semantics and failures | Children/context | Filled instance and expected result | Incomplete instance and expected finding | Specific checks |
| --- | --- | --- | --- | --- | --- | --- |
| EXP-EVAL: evaluate a name, operator, or conversion | R `kind:enum(name,operator,convert)`, `resultType:Ref<Type>`, `order:enum(left-to-right,right-to-left)`. C(kind=name) `binding:EntityLocation`. C(kind=operator) `operator:enum(add,subtract,multiply,divide,equal,and,or)`, `arguments:List<Expr>[2..2]`. C(operator=and or or) `shortCircuit:Bool`. C(kind=convert) `operand:Expr`, `conversion:Ref<Operation>`, `loss:enum(reject,declared-rounding)`. C(loss=declared-rounding) `rounding:Predicate`. | Name reads its exact visible binding. Operators use typed domains. Arithmetic on Int is exact. Divide requires nonzero divisor and an explicit division result rule through resultType. Boolean short circuit skips the second operand only when the first fixes the result. Conversion applies the named operation and declared loss policy. Operand failure propagates in evaluation order. Evaluation terminates when every evaluated operand and conversion terminates. | Operand roles: 0 for name, 2 for operator, 1 for conversion. Declared order, visible caller bindings only, no nonlocal exits. Expression. | kind=operator, operator=add, resultType=Int, order=left-to-right, arguments=[1,2]. Result 3. shortCircuit is not applicable. | Change operator=and, arguments=[true,false], resultType=Bool, but omit shortCircuit. Finding `missing-required`, path shortCircuit. | N resolves names. T checks operand/result domains and conditional slots. C checks order, skipped effects, and conversion failure. P checks profile operators. |

Division additionally requires C(operator=divide) `divisionRule:enum(exact-rational,truncate-zero,floor)` and `zero:enum(reject,declared-error)`.

Exact-rational requires a declared rational result type. Declared-error requires a declared error variant in resultType. No language rule follows from notation.

These conditional slots are part of EXP-EVAL's declaration. The positive addition instance does not require them.

Operator meanings use mathematical integers and Boolean values under this proposed common definition. T05 can reject an incompatible language mapping.

| Additional case | Complete assignments | Expected result |
| --- | --- | --- |
| EXPR-NAME | kind=name, resultType=Nat, order=left-to-right, binding={entity:retry-fixture@f1,slotPath:[bindings,retryCount]} resolving declared Known Nat 2 | Value 2. Disabled operator/conversion descendants are NotApplicable. No Unknown requiredness finding. |
| EXPR-WRONG-OPERAND | Copy filled add fixture, replace second Int argument with Bool true | Type finding at arguments[1]. No implicit Boolean-to-integer conversion. |
| EXPR-WRONG-CONTEXT | Place a statement-only child with return exit in an Expr operand role | Child-kind/context conflict at arguments[0], even if its eventual result type is Int. |
| EXPR-INVALID-LOCATION | Copy EXPR-NAME and use an empty path or a path selecting a Type field instead of a Binding | Located name/structure finding. No synthetic Binding entity is created. |

The name slot's EntityLocation resolves through its exact owning entity, package and declared path. It selects a visible Binding or FormalParameter declaration.

Retry-fixture@f1 is the complete FAIL-RETRY filled fixture with retryCount changed to Nat 2. Its other declared slots remain supplied.

A Binding read returns its actual typed Known value. A FormalParameter read derives its static type from the declaration and reads its materialized active-frame value.

No FormalParameter or frame becomes a new T02 entity. Logical paths unwrap declared Known record wrappers and stop on Unknown/Absent.

| Formal-parameter case | Complete static expression assignments | Expected phase result |
| --- | --- | --- |
| EXPR-FORMAL | kind=name, resultType=List<Pair>, order=left-to-right, binding={entity:R-input@r1,slotPath:[bindings,context,parameters,input]} under T04's declared contains/context example | Static type is List<Pair>. At invocation, actual [] or [Pair(-7,2)] is read only after parameter validation/materialization. No future list is stored as Known. |

Runtime activation uses T04's static context selection. No future invocation ID is needed in the canonical location.
