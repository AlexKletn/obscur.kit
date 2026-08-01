#![doc = include_str!("inspectable.doc/_module.md")]

#[doc = include_str!("inspectable.doc/Inspectable.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Inspectable {
    #[doc = include_str!("inspectable.doc/Tree.md")]
    Tree,
    #[doc = include_str!("inspectable.doc/Styles.md")]
    Styles,
    #[doc = include_str!("inspectable.doc/Layout.md")]
    Layout,
    #[doc = include_str!("inspectable.doc/EventFlow.md")]
    EventFlow,
    #[doc = include_str!("inspectable.doc/Focus.md")]
    Focus,
    #[doc = include_str!("inspectable.doc/Animations.md")]
    Animations,
    #[doc = include_str!("inspectable.doc/Resources.md")]
    Resources,
    #[doc = include_str!("inspectable.doc/Accessibility.md")]
    Accessibility,
    #[doc = include_str!("inspectable.doc/Scene.md")]
    Scene,
    #[doc = include_str!("inspectable.doc/Rendering.md")]
    Rendering,
    #[doc = include_str!("inspectable.doc/Caches.md")]
    Caches,
    #[doc = include_str!("inspectable.doc/PluginState.md")]
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
