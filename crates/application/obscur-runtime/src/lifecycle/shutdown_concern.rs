//! ShutdownConcern (skeleton stub for the spec-defined entity; not yet implemented).

/// Identifies which subsystem caused a critical stop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownConcern {
    /// The event loop could not be created or run.
    EventLoop,
    /// A window required by the host could not be created.
    Window,
    /// The rendering backend could not be initialized.
    Renderer,
    /// The host reported a runtime failure.
    Runtime,
}
