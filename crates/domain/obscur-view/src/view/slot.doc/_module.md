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

# Named slots only

§2.12 says *named* slots, and says nothing about an unnamed default one.
An outlet therefore always names a slot, and content is matched to it by
name and by nothing else. A child carrying no `slot="…"` reaches no outlet,
and an outlet no child names projects to nothing rather than to a leftover
box. Declaration order is not a destination — inventing one is what
`5e8e7fe` refused for `<Loading>`.

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
