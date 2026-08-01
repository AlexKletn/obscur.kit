Builds the app with a fresh [`UiRuntime`] wired to the shared scene: the
compiled stylesheet, the reactive view builder, and a handler per
interactive command. All command handlers mutate the shared
[`GalleryState`] and refresh its status line so the events are visible.
