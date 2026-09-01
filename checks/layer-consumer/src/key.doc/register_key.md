Register the component with a runtime.

Passed to `AppControl::with_runtime` in `panel.rs`, which is the only route an
application has to `UiRuntime::register_component` — and therefore the route
that has to survive on the layer path.
