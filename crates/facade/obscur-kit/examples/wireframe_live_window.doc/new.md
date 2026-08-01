Builds the app with a fresh [`UiRuntime`] wired to the shared scene: the
**installed UA default sheet** (the only thing that styles the bare
controls), the layout-chrome author sheet on top, the reactive view builder,
and a handler per interactive command. All handlers mutate the shared
[`WireState`] and refresh its status line so the events are visible.
