//! The set of inspectable live-state targets (spec §7.6).

/// A category of live runtime state that can be inspected via an
/// [`Inspect`](super::CommandKind::Inspect) command.
///
/// The variants enumerate the inspectable surfaces the spec calls out in §7.6.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Inspectable {
    /// The retained element/component tree.
    Tree,
    /// Resolved styles.
    Styles,
    /// Computed layout geometry.
    Layout,
    /// Event routing / dispatch flow.
    EventFlow,
    /// Current focus state.
    Focus,
    /// Active animations and their timelines.
    Animations,
    /// Loaded resources (fonts, images, …).
    Resources,
    /// Accessibility tree and node properties.
    Accessibility,
    /// The composited scene graph.
    Scene,
    /// Rendering pipeline state.
    Rendering,
    /// Cache occupancy and statistics.
    Caches,
    /// Plugin instances and their reported state.
    PluginState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targets_are_distinct() {
        assert_ne!(Inspectable::Tree, Inspectable::Scene);
        assert_eq!(Inspectable::PluginState, Inspectable::PluginState);
    }
}
