The tallest content extent actually laid out this frame, in **logical**
px — the largest bottom edge across the tree, read from the runtime's live
[`inspect_snapshot`](UiRuntime::inspect_snapshot). Used for the scroll
range, so scrolling reflects the real content rather than a fixed design
height.
