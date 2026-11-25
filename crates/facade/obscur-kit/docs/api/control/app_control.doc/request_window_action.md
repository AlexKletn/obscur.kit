Queue an action for the ordinary application window.

Request [`WindowAction::BeginMove`] or [`WindowAction::BeginResize`] inside
the primary mouse button's `pointerdown` handler. The runner hands control
to the window system during the same native input event. Support depends on
the window backend. Use the application view to choose the drag and resize
regions.
