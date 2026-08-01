Creates a virtual-list marker: `count` items of `row_extent` each, realized
through the row builder registered under `list_id` (spec §4.9).

The builder is deliberately absent from the view — a `View` is plain data and
a closure is not — so the runtime holds it and this names it. That split is
what lets the model carry every item while only the visible nodes are built.
