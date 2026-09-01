The fixtures, and what each is the sole cover for.

`facade-consumer` — the authoring surface reached through one dependency. The
defect class it exists for is an emitted `::obscur_view::…` path that resolves
inside the workspace and nowhere else.

`layer-consumer` — the same, on a desktop surface, with `window-winit` absent.
Two defect classes: the layer branch of `App::run` failing to compose from
outside, and the layer runner leaving something from `AppParts::boot` unread —
which under `full` looks used, because the windowed runner reads it.
