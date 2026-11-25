Creates a **slot outlet**: the position in a component's own output where
the children an author assigned to the slot called `name` are projected
(spec §2.12).

This is the half of a slot that a component owns. The other half is the
author's `<h1 slot="header"/>`; [`project_slots`](super::project_slots)
joins them, replacing this node with that content before the output is
reconciled — so an outlet never becomes a retained node, filled or not.

`name` must be one of the slots the component's
[`ComponentContract`](crate::component::ComponentContract) declares: that
declaration is what both front ends check an author's assignment against,
so an outlet naming an undeclared slot can never receive anything.
