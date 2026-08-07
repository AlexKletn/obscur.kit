The node's laid-out box as `(x, y, width, height)`.

Read from `UiRuntime::layout_boxes` — the same map the compose phase hands to
`CanvasPainters::paint` to open a painter's region with, so a box read here is
the box a painter would be given. Flattened to a tuple because these tests
compare whole rectangles for equality and report them in failures, and a tuple
of `f32` prints and compares without dragging the geometry types in.
