Emits the `ViewValue` expression for a property's value expression.

A binding emits its **Rust path**, converted through
[`ViewValue::from`](obscur_view::view::ViewValue): `value={label}` becomes
`ViewValue::from(label.clone())`. It used to emit the path as a string
literal, so `value={user.name}` produced the literal text `"user.name"` — the
placeholder that made every dynamic `.okui` document silently wrong.

The `.clone()` is uniform rather than clever: it works for an owned `String`
and for a `Copy` number alike, and a binding read into a `ViewValue` cannot
move out of the caller's data — a `<For>` body would read the same field on
every iteration.

If the path names nothing, or names something no `ViewValue` can be built
from, **rustc says so** in the caller's build. That is the type checking, and
it is not reimplemented here.

A call still emits `Null`: §2.10 allows "explicitly exported pure functions",
but nothing yet decides which functions those are, and emitting an arbitrary
call would be exactly the unconstrained invocation §2.10 forbids.
