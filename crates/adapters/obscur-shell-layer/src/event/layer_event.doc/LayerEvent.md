One thing that happened to a layer surface, in the host's own vocabulary.

Pointer and touch arrive already converted into the kit's
[`PointerEvent`], so a host feeds them to `UiRuntime::handle_pointer`
unchanged and every device-agnostic behaviour — hover, focus, gestures,
capture — works as it does in a window.
