Replaces every `<VirtualFor>` marker in `view` with the window it realizes
for a viewport of `viewport_height` logical pixels (spec §4.9).

Called only for a view that carries a marker, so the overwhelmingly common
view is never cloned.

Two things happen here in one call because their order is the whole point.
First, any list whose scroll-reset token changed has its container's offset
seated at the top — the app has replaced that list's content, and the stored
offset belonged to the old (on the frame a list first appears there is no
container yet and the offset is already zero, so this is a no-op then).
Only then is each registered list's window asked of `VirtualLayout`
**against that same offset**, its rows built through the registered builder,
and the window and the extent it was chosen at recorded together. Finally
the older template marker — a `<VirtualFor>` with no builder — clones its
template across a viewport-sized window of its own.
