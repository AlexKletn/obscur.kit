A single keyboard key transition delivered to the focused node (spec §6.7).

A `KeyEvent` is the domain counterpart of a platform key event: it pairs the
layout-independent [`physical`](Self::physical) key position with the
layout-dependent [`logical`](Self::logical) key the active layout produced
(absent for a key the layout does not resolve to a modeled logical key, such
as a bare modifier or an unmapped named key) and a [`pressed`](Self::pressed)
flag distinguishing a press from a release.

It is the keyboard analogue of
[`PointerEvent`](crate::pointer::PointerEvent): a pure payload that the
application layer routes to the focused node and resolves against a
[`CommandRouter`](crate::command::CommandRouter). The two halves come from a
backend converter (e.g. the winit `KeyboardConverter`), which splits a native
key event into its physical and logical parts.
