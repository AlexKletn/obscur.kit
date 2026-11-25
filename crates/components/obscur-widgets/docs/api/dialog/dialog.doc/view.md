Derives the semantic view.

While [closed](Self::is_open) the dialog renders nothing —
[`View::empty`] — so it contributes no top-level nodes. While open it
returns a single [`COMPONENT_ID`] element that carries the role, modal
flag, and title as properties and hosts the content nodes as its children.

# What it deliberately does not do

It does **not** project the surface through a
[`Portal`](obscur_primitives::Portal). It used to, targeting `"dialog-root"`,
and that wrapper was inert: nothing in the workspace maps a portal target
*name* to a node, and `obscur_tree::portal::Portal::apply` — the call that
would move the surface's visual parent — has no production caller. So the
element is emitted where its author wrote it, and an open dialog inside an
`overflow: hidden` ancestor **is clipped by that ancestor**. That is a real
limitation, recorded in `docs/api/12-reality.md` under *Portals*, and it is
stated here rather than hidden behind a wrapper that looks like it lifts the
surface out.

[`COMPONENT_ID`]: Self::COMPONENT_ID
