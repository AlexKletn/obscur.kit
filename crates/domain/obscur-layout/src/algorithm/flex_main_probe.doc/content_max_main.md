The item's **content-based minimum main size** — its max-content size along the
main axis, measured by laying the item out with an *unbounded* main axis and
the container's cross extent (CSS Flexbox Level 1 §4.5 / §9.2,
<https://drafts.csswg.org/css-flexbox-1/#min-size-auto>).

For content whose main-axis extent does not reflow (a column's stacked block
height, an atomic / single-word inline) this equals the min-content size, so it
is the exact automatic minimum. For reflowable content (a multi-word inline in
a row) it is an upper bound of the true min-content — a deliberately safe
over-approximation, since the engine cannot measure a text leaf's min-content
through [`ChildMeasurer`](super::ChildMeasurer): that callback answers a size
under given constraints and has no min-content question to ask.

The result may be [`f32::INFINITY`] for a fill-available container whose
max-content is unbounded; the caller resolves that to a finite floor.
