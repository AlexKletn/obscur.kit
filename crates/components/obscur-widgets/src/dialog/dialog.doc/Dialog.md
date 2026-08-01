A headless dialog surface hosting arbitrary content (spec §2.6, §6.9).

`Dialog` owns its `title`, an ordered list of arbitrary content
[`ViewNode`]s, whether it is currently [`open`](Self::is_open), and whether
it is [`modal`](Self::is_modal). It bakes in **no** visual theme.

Rendering is gated on the open flag: while closed, [`view`](Self::view)
returns [`View::empty`] — a dialog contributes nothing to the tree until it
is opened. While open, its element is projected through a
[`Portal`](obscur_primitives::Portal) to the `"dialog-root"` target so the
surface escapes its ancestor's clip and z-order; the element carries the
role, modal flag, and title as properties and hosts the content nodes as its
children, rendered by the style layer keyed on
[`COMPONENT_ID`](Self::COMPONENT_ID).

The state machine is a plain open/close toggle ([`open`](Self::open),
[`close`](Self::close), [`toggle_open`](Self::toggle_open)).
