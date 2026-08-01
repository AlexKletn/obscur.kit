Derives the semantic view.

While [closed](Self::is_open) the dialog renders nothing —
[`View::empty`] — so it contributes no top-level nodes. While open it
returns a single [`Portal`](obscur_primitives::Portal) node targeting
`"dialog-root"`, wrapping a [`COMPONENT_ID`] element that carries the
role, modal flag, and title as properties and hosts the content nodes as
its children.

[`COMPONENT_ID`]: Self::COMPONENT_ID
