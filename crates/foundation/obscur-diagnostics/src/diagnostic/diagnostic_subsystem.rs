//! Originating subsystem of a diagnostic.

/// The subsystem a diagnostic originated from, for attribution (spec §7.9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSubsystem {
    /// Component/view layer.
    View,
    /// Retained tree and reconciliation.
    Tree,
    /// Style resolution.
    Style,
    /// Layout.
    Layout,
    /// Text and shaping.
    Text,
    /// Motion engine.
    Motion,
    /// Scene/paint and rendering.
    Render,
    /// Resource loading.
    Resources,
    /// Windowing and platform.
    Platform,
    /// Plugin host.
    Plugin,
    /// Runtime orchestration.
    Runtime,
}
