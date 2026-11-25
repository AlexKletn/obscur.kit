Reports a **stray §2.12 slot outlet** in `desired` on the diagnostics
channel (spec §1.6, §2.12).

An outlet is a position inside a *component's* output: `render_components`
runs `project_slots` over that output and substitutes every one of them
away before it is committed. `desired` is the application's own view — it
is nobody's render output, nothing will ever project it, and an outlet in
it therefore names a slot that will never be filled.

The tree refuses it either way (the reconciler will not materialize an
outlet), so this is the half that makes the refusal *loud* rather than
silent — the same treatment `float`, `@container` and `::before` get, moved
to the only place that can decide it. The front ends cannot: a component's
render returns a `View` and `okui!` produces a `View`, so
`okui! { <slot name="header"/> }` is legal inside `Component::render` and
illegal one function away, and no proc-macro can tell those two call sites
apart. Whether an outlet is stray is a fact about *who is projecting the
view*, which is knowable here and nowhere earlier.

One walk of the desired tree, on structural frames only, and only when the
frame reconciles at all. It is not free — it is a second visit to nodes the
reconcile that follows visits anyway — and there is no cheaper signal: a
document that authors no outlet cannot be recognised without looking.
