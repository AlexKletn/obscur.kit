Everything the runners need, in one bundle.

Both backends build the same runtime from the same pieces — root, commands,
stylesheets, virtual lists, the control handle — and only then diverge over how
a surface reaches the screen. Bundling them is what lets that shared half be
written once, in
[`AppParts::boot`](super::app_parts::AppParts::boot), instead of once per
backend where the two copies drift.
