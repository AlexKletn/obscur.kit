//! The typed classification of a change (Obscur Kit 2 specs §4.3).

#[doc = include_str!("../../docs/api/invalidation/invalidation_kind.doc/InvalidationKind.md")]
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
    #[doc = include_str!("../../docs/api/invalidation/invalidation_kind.doc/needs_restyle.md")]
    #[must_use]
    pub fn needs_restyle(self) -> bool {
        matches!(
            self,
            InvalidationKind::Style | InvalidationKind::Tree | InvalidationKind::Environment
        )
    }

    #[doc = include_str!("../../docs/api/invalidation/invalidation_kind.doc/needs_relayout.md")]
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

    #[doc = include_str!("../../docs/api/invalidation/invalidation_kind.doc/needs_repaint.md")]
    #[must_use]
    pub fn needs_repaint(self) -> bool {
        !matches!(self, InvalidationKind::CompositionOnly)
    }

    #[doc = include_str!("../../docs/api/invalidation/invalidation_kind.doc/needs_recompose.md")]
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
