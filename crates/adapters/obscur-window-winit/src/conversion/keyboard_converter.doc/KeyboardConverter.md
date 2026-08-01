A stateless converter from winit's native keys to the domain's layout-
independent [`PhysicalKey`] and layout-dependent [`LogicalKey`] (spec §6.7,
§2.7).

These are **pure**, per-key conversions with no event-loop state. The backend
extracts the two halves of a winit `KeyEvent` — its
`physical_key` and `logical_key` — and calls
[`KeyboardConverter::physical_key`] / [`KeyboardConverter::logical_key`] on
them. Only the keys both winit and the domain model are mapped; anything else
(media keys, IME composition, unidentified scancodes) converts to `None`.
