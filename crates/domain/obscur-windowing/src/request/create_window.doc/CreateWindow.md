An application's intent to create a new window (spec §6.1, §6.2).

A `CreateWindow` bundles the desired [`WindowProperties`], size
[`WindowConstraints`], and [`WindowRelation`] for a window that does not yet
exist. It is a pure command value: a backend consumes it, mints a
[`WindowId`], and the request can then be materialized into a retained
[`Window`] via [`CreateWindow::into_window`].
