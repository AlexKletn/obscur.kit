//! Routes classified invalidations to the domain accumulators and tracks which
//! pipeline phases are dirty (Obscur Kit 2 specs §4.3).

use obscur_layout::invalidation::{LayoutInvalidation, LayoutInvalidationReason};
use obscur_style::invalidation::{StyleInvalidation, StyleInvalidationReason};

use crate::invalidation::{Invalidation, InvalidationReason};

/// Maps a runtime [`InvalidationReason`] to the [`StyleInvalidationReason`] the
/// style domain should record for the affected node.
///
/// Only the style-affecting causes map to a concrete style reason:
///
/// | [`InvalidationReason`]              | [`StyleInvalidationReason`]        |
/// |-------------------------------------|------------------------------------|
/// | `StyleChanged`                      | `DeclarationsChanged`              |
/// | `InlineStyleChanged`                | `InlineStyleChanged`              |
/// | `PseudoStateChanged`                | `PseudoStateChanged`              |
/// | `CustomPropertyChanged`             | `CustomPropertyChanged`           |
/// | `TreeChanged` / `ContentChanged` / `EnvironmentChanged` | `None`         |
///
/// A `None` result means the change dirties the style phase (when its kind
/// [`needs_restyle`](crate::invalidation::InvalidationKind::needs_restyle)) but
/// does not correspond to a single-node style edit; the frame pipeline restyles
/// the affected subtree from the domains' dependency records.
fn style_reason(reason: InvalidationReason) -> Option<StyleInvalidationReason> {
    match reason {
        InvalidationReason::StyleChanged => Some(StyleInvalidationReason::DeclarationsChanged),
        InvalidationReason::InlineStyleChanged => Some(StyleInvalidationReason::InlineStyleChanged),
        InvalidationReason::PseudoStateChanged => Some(StyleInvalidationReason::PseudoStateChanged),
        InvalidationReason::CustomPropertyChanged => {
            Some(StyleInvalidationReason::CustomPropertyChanged)
        }
        InvalidationReason::TreeChanged
        | InvalidationReason::ContentChanged
        | InvalidationReason::EnvironmentChanged => None,
    }
}

/// Maps a runtime [`InvalidationReason`] to the [`LayoutInvalidationReason`] the
/// layout domain should record for the affected node.
///
/// Every cause maps to a layout reason because each ultimately feeds layout:
///
/// | [`InvalidationReason`]  | [`LayoutInvalidationReason`] |
/// |-------------------------|------------------------------|
/// | `StyleChanged` / `InlineStyleChanged` / `PseudoStateChanged` / `CustomPropertyChanged` | `StyleChanged` |
/// | `TreeChanged`           | `TreeChanged`               |
/// | `ContentChanged`        | `ContentChanged`            |
/// | `EnvironmentChanged`    | `EnvironmentChanged`        |
fn layout_reason(reason: InvalidationReason) -> LayoutInvalidationReason {
    match reason {
        InvalidationReason::StyleChanged
        | InvalidationReason::InlineStyleChanged
        | InvalidationReason::PseudoStateChanged
        | InvalidationReason::CustomPropertyChanged => LayoutInvalidationReason::StyleChanged,
        InvalidationReason::TreeChanged => LayoutInvalidationReason::TreeChanged,
        InvalidationReason::ContentChanged => LayoutInvalidationReason::ContentChanged,
        InvalidationReason::EnvironmentChanged => LayoutInvalidationReason::EnvironmentChanged,
    }
}

/// Routes classified invalidations to the domain accumulators and tracks which
/// pipeline phases are dirty for the current frame (§4.3).
///
/// The router does **not** walk the tree. It records node-level invalidations in
/// the [`StyleInvalidation`] and [`LayoutInvalidation`] accumulators and flags
/// which phases the frame pipeline must run. The spec's "recalculation starts at
/// the nearest affected context" is realised by the domains' own dependency
/// records, not by this router.
///
/// The dirty-phase flags accumulate across calls to [`route`](Self::route) within
/// a frame and are cleared by [`reset`](Self::reset); the domain accumulators are
/// owned and reset elsewhere.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InvalidationRouter {
    /// The style pass must run.
    style_dirty: bool,
    /// The layout pass must run.
    layout_dirty: bool,
    /// The scene must be re-composed.
    scene_dirty: bool,
    /// The scene must be re-painted.
    paint_dirty: bool,
}

impl InvalidationRouter {
    /// Creates a router with no dirty phases.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Routes a single [`Invalidation`] to the domain accumulators and flags the
    /// dirty pipeline phases.
    ///
    /// From the invalidation's [`InvalidationKind`](crate::invalidation::InvalidationKind):
    /// a restyle-needing change flags the style phase; a relayout-needing change
    /// flags the layout phase; every change flags the scene (recompose) phase; a
    /// repaint-needing change flags the paint phase.
    ///
    /// When the invalidation carries a node (see [`Invalidation::node`]) the
    /// change is forwarded to the domains: if it needs a restyle and maps to a
    /// concrete [`StyleInvalidationReason`] (see [`style_reason`]) the node is
    /// invalidated in `style`; if it needs a relayout the node is invalidated in
    /// `layout` with the mapped [`LayoutInvalidationReason`] (see [`layout_reason`]).
    /// A global change (node `None`) only sets the phase flags — the frame
    /// pipeline invalidates the whole tree.
    pub fn route(
        &mut self,
        inv: Invalidation,
        style: &mut StyleInvalidation,
        layout: &mut LayoutInvalidation,
    ) {
        let kind = inv.kind();

        if kind.needs_restyle() {
            self.style_dirty = true;
        }
        if kind.needs_relayout() {
            self.layout_dirty = true;
        }
        // Every classified change re-composes the scene.
        self.scene_dirty = true;
        if kind.needs_repaint() {
            self.paint_dirty = true;
        }

        if let Some(node) = inv.node() {
            if kind.needs_restyle()
                && let Some(reason) = style_reason(inv.reason()) {
                    style.invalidate(node, reason);
                }
            if kind.needs_relayout() {
                layout.invalidate(node, layout_reason(inv.reason()));
            }
        }
    }

    /// Returns whether the style pass is flagged dirty.
    #[must_use]
    pub fn needs_style(&self) -> bool {
        self.style_dirty
    }

    /// Returns whether the layout pass is flagged dirty.
    #[must_use]
    pub fn needs_layout(&self) -> bool {
        self.layout_dirty
    }

    /// Returns whether the scene pass (recompose and/or paint) is flagged dirty.
    #[must_use]
    pub fn needs_scene(&self) -> bool {
        self.scene_dirty || self.paint_dirty
    }

    /// Returns whether no pipeline phase is flagged dirty.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        !(self.style_dirty || self.layout_dirty || self.scene_dirty || self.paint_dirty)
    }

    /// Clears the dirty-phase flags for the next frame.
    ///
    /// The domain accumulators ([`StyleInvalidation`] / [`LayoutInvalidation`])
    /// are owned and reset elsewhere.
    pub fn reset(&mut self) {
        self.style_dirty = false;
        self.layout_dirty = false;
        self.scene_dirty = false;
        self.paint_dirty = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::NodeId;

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    #[test]
    fn new_router_is_clean() {
        let router = InvalidationRouter::new();
        assert!(router.is_clean());
        assert!(!router.needs_style());
        assert!(!router.needs_layout());
        assert!(!router.needs_scene());
    }

    #[test]
    fn style_change_marks_all_phases_and_forwards_to_both_domains() {
        let mut router = InvalidationRouter::new();
        let mut style = StyleInvalidation::new();
        let mut layout = LayoutInvalidation::new();
        let n = node(3);

        router.route(
            Invalidation::new(Some(n), InvalidationReason::StyleChanged),
            &mut style,
            &mut layout,
        );

        assert!(router.needs_style());
        assert!(router.needs_layout());
        assert!(router.needs_scene());
        assert!(!router.is_clean());

        assert_eq!(
            style.reason(n),
            Some(StyleInvalidationReason::DeclarationsChanged)
        );
        assert_eq!(
            layout.reason(n),
            Some(LayoutInvalidationReason::StyleChanged)
        );
    }

    #[test]
    fn tree_change_routes_to_layout_tree_changed() {
        let mut router = InvalidationRouter::new();
        let mut style = StyleInvalidation::new();
        let mut layout = LayoutInvalidation::new();
        let n = node(9);

        router.route(
            Invalidation::new(Some(n), InvalidationReason::TreeChanged),
            &mut style,
            &mut layout,
        );

        assert!(router.needs_style());
        assert!(router.needs_layout());
        // TreeChanged has no single-node style reason mapping.
        assert!(!style.is_invalidated(n));
        assert_eq!(layout.reason(n), Some(LayoutInvalidationReason::TreeChanged));
    }

    #[test]
    fn content_change_relayouts_without_restyle() {
        let mut router = InvalidationRouter::new();
        let mut style = StyleInvalidation::new();
        let mut layout = LayoutInvalidation::new();
        let n = node(4);

        router.route(
            Invalidation::new(Some(n), InvalidationReason::ContentChanged),
            &mut style,
            &mut layout,
        );

        assert!(!router.needs_style());
        assert!(router.needs_layout());
        assert!(!style.is_invalidated(n));
        assert_eq!(
            layout.reason(n),
            Some(LayoutInvalidationReason::ContentChanged)
        );
    }

    #[test]
    fn global_environment_change_sets_flags_without_node_invalidation() {
        let mut router = InvalidationRouter::new();
        let mut style = StyleInvalidation::new();
        let mut layout = LayoutInvalidation::new();

        router.route(
            Invalidation::global(InvalidationReason::EnvironmentChanged),
            &mut style,
            &mut layout,
        );

        assert!(router.needs_style());
        assert!(router.needs_layout());
        assert!(router.needs_scene());
        assert!(style.is_empty());
        assert!(layout.is_empty());
    }

    #[test]
    fn reset_clears_flags() {
        let mut router = InvalidationRouter::new();
        let mut style = StyleInvalidation::new();
        let mut layout = LayoutInvalidation::new();

        router.route(
            Invalidation::new(Some(node(1)), InvalidationReason::StyleChanged),
            &mut style,
            &mut layout,
        );
        assert!(!router.is_clean());

        router.reset();
        assert!(router.is_clean());
        assert!(!router.needs_style());
        assert!(!router.needs_layout());
        assert!(!router.needs_scene());
    }
}
