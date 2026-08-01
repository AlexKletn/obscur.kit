An application's request to activate a window and optionally focus an element
within it (spec §6.3).

A `ActivateWindow` names a `target` window and a [`FocusRequest`] describing
whether to also move keyboard focus to a specific element. It is a pure
command value; a backend consumes it and reports an [`ActivationResult`].
