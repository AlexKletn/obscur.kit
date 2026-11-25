A headless dialog surface hosting arbitrary content (spec §2.6, §6.9).

`Dialog` owns its `title`, an ordered list of arbitrary content
[`ViewNode`]s, whether it is currently [`open`](Self::is_open), and whether
it is [`modal`](Self::is_modal). It bakes in **no** visual theme.

Rendering is gated on the open flag: while closed, [`view`](Self::view)
returns [`View::empty`] — a dialog contributes nothing to the tree until it
is opened. While open, its element carries the role, modal flag, and title as
properties and hosts the content nodes as its children, rendered by the style
layer keyed on [`COMPONENT_ID`](Self::COMPONENT_ID).

The state machine is a plain open/close toggle ([`open`](Self::open),
[`close`](Self::close), [`toggle_open`](Self::toggle_open)).

# Limitation: the surface does not escape its ancestors

The element is emitted **at the author's position in the tree**. It is not
lifted into an overlay root, so an `overflow: hidden` or a stacking ancestor
between the dialog and the window clips and stacks it like any other
descendant. See [`view`](Self::view) for why the `Portal` wrapper that used to
claim otherwise was removed, and `docs/api/12-reality.md` under *Portals* for
what is missing and where.
