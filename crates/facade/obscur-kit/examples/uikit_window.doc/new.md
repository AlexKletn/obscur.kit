Builds the application with a fresh [`UiRuntime`] wired to the shared UI-Kit
scene: the reactive view builder (`uikit::view(active_theme)`), a command
handler per switcher tab, and the initial (Material) stylesheet. No window,
surface, or GPU device yet (created on `resumed`).
