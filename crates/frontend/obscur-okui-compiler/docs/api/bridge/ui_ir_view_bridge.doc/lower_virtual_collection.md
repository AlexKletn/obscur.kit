Lowers a `<VirtualFor>` [`Loop`](UiIrNodeKind::Loop) node into a
virtual-collection marker (spec §2.11, §4.9).

The loop's template body becomes the marker's per-row template — lowered
once, not once per item — and the marker carries the default estimated row
extent. The runtime substitutes the marker away before reconcile, replacing
it with as many template clones as the viewport-bounded window holds; the
realized count is therefore a function of the viewport, not of the
collection's length, which is the promise a `<For>` cannot keep.
