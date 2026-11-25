Returns whether a value of this type may be assigned to a slot of
`target` (spec §2.9 assignability).

The rules, checked in order:

* [`Any`](ValueType::Any) is the escape hatch: it assigns to and from
  every type, in either direction.
* [`Unknown`](ValueType::Unknown) is a not-yet-inferred placeholder, so it
  assigns to anything (the real check is deferred).
* Identical types always assign.
* [`Int`](ValueType::Int) widens to [`Float`](ValueType::Float).
* A [`List`](ValueType::List)`<T>` assigns to `List<U>` exactly when its
  element type `T` assigns to `U`.
* An [`Optional`](ValueType::Optional)`<T>` assigns to `Optional<U>`
  exactly when `T` assigns to `U`.
* A bare value `T` assigns to `Optional<U>` when `T` assigns to `U` (a
  present value fits an optional slot).
* Otherwise the assignment is rejected.
