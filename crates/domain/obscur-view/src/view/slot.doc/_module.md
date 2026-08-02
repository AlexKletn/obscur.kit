Slots: the reserved `slot="…"` assignment, the outlet a component's output
declares, and the projection that joins them (spec §2.12, §2.13).

# What §2.12 asks for

> Components may declare: named slots; allowed child contracts; slot
> cardinality; exported style parts; exported visual states.

and §2.13 lists `slots` among the fields of the contract a component
**exports**. So a slot is a *declaration*, and a declaration is only
load-bearing if something acts on it. Two halves have to meet:

* the **assignment** — an authored `<h1 slot="header"/>` under a component
  tag. Both front ends already lower this, to an ordinary property under the
  reserved [`SLOT`] symbol, and both already check it against the
  component's declared slots at compile time (name, cardinality, and the
  slot's own child rule);
* the **outlet** — the position *inside the component's own output* where
  that content belongs. This is the half that did not exist: a component's
  output named no such position, so there was nowhere to route to and the
  authored children were dropped.

[`ViewNode::slot`](super::ViewNode::slot) is that position, and
[`project_slots`] is the join.

# Named slots, and the default one

§2.12 says *named* slots, and content that names a slot reaches that slot and
no other. What it does not say is where content that names **nothing** goes,
and that question has an answer that cannot be avoided: `<button>Save</button>`
is a control's authored content, its only child is a text run, and a text run
carries no properties — so it can never name a slot, however the author writes
it. Either such content has a destination or a mounted control silently eats
the author's markup.

So there is one, and exactly one, unnamed destination: the **default slot**,
declared by an outlet that names no slot, filled by the content that names
none. Both halves are explicit — the component says "the author's content
goes here" by rendering the outlet, and the author says "not the default" by
writing a `slot="…"`. What stays refused is a destination *inferred* from
something that never said it was one: declaration order is still not a
destination, which is what `5e8e7fe` refused for `<Loading>`, and content
naming a slot the output declares no outlet for still reaches nothing rather
than falling back to the default.

An outlet nothing is assigned to renders its **fallback** — its own children —
and one with no fallback renders nothing at all rather than a leftover box.
That is what makes the default slot enough on its own for a control whose
content has a prop spelling too: `<button label="Save"/>` and
`<button>Save</button>` are one component rendering one position, and the
author's content wins where there is any.

# The authored spelling

`<slot name="header"/>` is the outlet, on both front ends. It needs no node of
its own in either parser: it is an ordinary element tag carrying an ordinary
attribute, and the whole of it is two rules in the frontend crate's
`name_binding` — the tag binds to [`SLOT_OUTLET`] and, *on that tag only*,
`name` binds to [`SLOT`]. A component's render is a `View`, and `okui!`
produces a `View`, so `okui! { <slot name="header"/> }` inside
`Component::render` is the same outlet
[`ViewNode::slot`](super::ViewNode::slot) builds by hand.

# Why an outlet is a reserved `ComponentId` and not a `ViewNode` variant

Because an outlet never becomes a retained node. Projection runs over a
component's output *before* it is reconciled, so the outlet is substituted away
exactly like an `<If>` is at lowering — "these constructs do not create real
retained nodes unless their rendered output requires them" (§2.11). A third
[`ViewNode`] variant would put a case in all sixty-odd exhaustive matches over
the enum to describe a node no consumer of the retained tree can ever see.

**A stray outlet is refused, not retained.** An outlet authored *outside* a
component — one in a view that is nobody's render output — meets no projection
pass, and used to land in the tree as an ordinary `Element` carrying the
reserved id, rendering as an empty box. The reconciler now refuses to
materialize any node [`is_outlet`] answers `true` for, so the sentence above is
unconditional on every path that builds a tree; `UiRuntime`, which has a §1.6
diagnostics channel, also *reports* the stray rather than dropping it quietly.
This paragraph said "never" once before, as a claim rather than a mechanism,
and a verifier built a stray and found it in the tree. What changed is the
mechanism, not the prose.
