Maps a winit [`Key`](winit::keyboard::Key) to the domain's layout-
dependent [`LogicalKey`], or `None` for a key the domain does not model
(spec §6.7).

A `Key::Character` maps to the first `char` of the produced text; a
`Key::Named` maps to the matching domain [`NamedKey`] when both crates
model it. `Key::Dead` and `Key::Unidentified`, and named keys outside the
modeled subset, convert to `None`.
