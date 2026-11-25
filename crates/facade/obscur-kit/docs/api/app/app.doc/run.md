Opens a window, wires the full standalone stack, and runs the event loop,
**blocking** until the window is closed.

# Errors

Returns an [`AppError`] if an `.okui` [`screen`](Self::screen) fails to
compile, the event loop cannot be built, the window cannot be created, or no
usable GPU adapter/device/surface is available.
