//! The typed classification of a change (Obscur Kit 2 specs §4.3).

/// The typed classification of a change, ordered from shallow to deep.
///
/// A deeper kind implies all the (shallower) work of the kinds above it:
/// changing a node's tree structure ([`Tree`](InvalidationKind::Tree)) implies
/// re-styling, re-laying-out, re-painting and re-composing, whereas a
/// composition-only change ([`CompositionOnly`](InvalidationKind::CompositionOnly))
/// only requires the scene to be re-composed. The predicate methods
/// ([`needs_restyle`](InvalidationKind::needs_restyle),
/// [`needs_relayout`](InvalidationKind::needs_relayout),
/// [`needs_repaint`](InvalidationKind::needs_repaint),
/// [`needs_recompose`](InvalidationKind::needs_recompose)) answer what work a
/// classified change requires.
///
/// The [`Ord`] instance orders variants by depth (shallow < deep), matching the
/// declaration order below.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InvalidationKind {
    /// Only the scene composition changed (e.g. layer ordering / transform);
    /// no repaint, relayout or restyle is required, only a recompose.
    CompositionOnly,
    /// A paint-level change (e.g. colour/opacity of already-laid-out content);
    /// requires a repaint (and recompose) but no relayout or restyle.
    Paint,
    /// Text / intrinsic content changed; requires relayout (content affects
    /// sizing), repaint and recompose, but no restyle.
    Text,
    /// A node's computed style changed; requires restyle. Because layout style
    /// is derived from computed style, in Phase 1 this also requires relayout
    /// (and hence repaint + recompose).
    Style,
    /// Layout inputs changed (e.g. constraints); requires relayout, repaint and
    /// recompose, but no restyle.
    Layout,
    /// The node tree changed (children added/removed); requires restyle,
    /// relayout, repaint and recompose.
    Tree,
    /// The window/environment changed (viewport, DPI); the deepest kind,
    /// requiring restyle, relayout, repaint and recompose across the tree.
    Environment,
}

impl InvalidationKind {
    /// Returns whether a change of this kind requires a restyle pass.
    ///
    /// True for style-affecting kinds: [`Style`](InvalidationKind::Style),
    /// [`Tree`](InvalidationKind::Tree) and
    /// [`Environment`](InvalidationKind::Environment).
    #[must_use]
    pub fn needs_restyle(self) -> bool {
        matches!(
            self,
            InvalidationKind::Style | InvalidationKind::Tree | InvalidationKind::Environment
        )
    }

    /// Returns whether a change of this kind requires a relayout pass.
    ///
    /// True for [`Text`](InvalidationKind::Text),
    /// [`Style`](InvalidationKind::Style) (layout style is derived from computed
    /// style, so a style change feeds layout), [`Layout`](InvalidationKind::Layout),
    /// [`Tree`](InvalidationKind::Tree) and
    /// [`Environment`](InvalidationKind::Environment).
    #[must_use]
    pub fn needs_relayout(self) -> bool {
        matches!(
            self,
            InvalidationKind::Text
                | InvalidationKind::Style
                | InvalidationKind::Layout
                | InvalidationKind::Tree
                | InvalidationKind::Environment
        )
    }

    /// Returns whether a change of this kind requires a repaint pass.
    ///
    /// True for [`Paint`](InvalidationKind::Paint) and every deeper kind. A
    /// [`CompositionOnly`](InvalidationKind::CompositionOnly) change needs only a
    /// recompose and therefore returns `false`.
    #[must_use]
    pub fn needs_repaint(self) -> bool {
        !matches!(self, InvalidationKind::CompositionOnly)
    }

    /// Returns whether a change of this kind requires a scene recomposition.
    ///
    /// Always `true`: every classified change ultimately re-composes the scene.
    #[must_use]
    pub fn needs_recompose(self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_needs_restyle_and_relayout() {
        assert!(InvalidationKind::Style.needs_restyle());
        assert!(InvalidationKind::Style.needs_relayout());
        assert!(InvalidationKind::Style.needs_repaint());
        assert!(InvalidationKind::Style.needs_recompose());
    }

    #[test]
    fn layout_needs_relayout_not_restyle() {
        assert!(!InvalidationKind::Layout.needs_restyle());
        assert!(InvalidationKind::Layout.needs_relayout());
        assert!(InvalidationKind::Layout.needs_repaint());
    }

    #[test]
    fn composition_only_recomposes_only() {
        let k = InvalidationKind::CompositionOnly;
        assert!(!k.needs_restyle());
        assert!(!k.needs_relayout());
        assert!(!k.needs_repaint());
        assert!(k.needs_recompose());
    }

    #[test]
    fn paint_repaints_but_not_relayout() {
        let k = InvalidationKind::Paint;
        assert!(!k.needs_restyle());
        assert!(!k.needs_relayout());
        assert!(k.needs_repaint());
        assert!(k.needs_recompose());
    }

    #[test]
    fn text_relayouts_without_restyle() {
        let k = InvalidationKind::Text;
        assert!(!k.needs_restyle());
        assert!(k.needs_relayout());
        assert!(k.needs_repaint());
    }

    #[test]
    fn tree_and_environment_are_deepest() {
        for k in [InvalidationKind::Tree, InvalidationKind::Environment] {
            assert!(k.needs_restyle());
            assert!(k.needs_relayout());
            assert!(k.needs_repaint());
            assert!(k.needs_recompose());
        }
    }

    #[test]
    fn ordering_is_shallow_to_deep() {
        assert!(InvalidationKind::CompositionOnly < InvalidationKind::Paint);
        assert!(InvalidationKind::Paint < InvalidationKind::Text);
        assert!(InvalidationKind::Text < InvalidationKind::Style);
        assert!(InvalidationKind::Style < InvalidationKind::Layout);
        assert!(InvalidationKind::Layout < InvalidationKind::Tree);
        assert!(InvalidationKind::Tree < InvalidationKind::Environment);
    }
}
