Builds the application with a fresh [`UiRuntime`] wired to the shared
scene: the compiled stylesheet, the reactive view builder, and a handler
per interactive command. No window, surface, or GPU device yet (created on
`resumed`); image resources are also installed there since the sandbox dir
is staged at startup.
