#![doc = include_str!("invalidation_kind.doc/_module.md")]

#[doc = include_str!("invalidation_kind.doc/InvalidationKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InvalidationKind {
    #[doc = include_str!("invalidation_kind.doc/CompositionOnly.md")]
    CompositionOnly,
    #[doc = include_str!("invalidation_kind.doc/Paint.md")]
    Paint,
    #[doc = include_str!("invalidation_kind.doc/Text.md")]
    Text,
    #[doc = include_str!("invalidation_kind.doc/Style.md")]
    Style,
    #[doc = include_str!("invalidation_kind.doc/Layout.md")]
    Layout,
    #[doc = include_str!("invalidation_kind.doc/Tree.md")]
    Tree,
    #[doc = include_str!("invalidation_kind.doc/Environment.md")]
    Environment,
}

impl InvalidationKind {
    #[doc = include_str!("invalidation_kind.doc/needs_restyle.md")]
    #[must_use]
    pub fn needs_restyle(self) -> bool {
        matches!(
            self,
            InvalidationKind::Style | InvalidationKind::Tree | InvalidationKind::Environment
        )
    }

    #[doc = include_str!("invalidation_kind.doc/needs_relayout.md")]
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

    #[doc = include_str!("invalidation_kind.doc/needs_repaint.md")]
    #[must_use]
    pub fn needs_repaint(self) -> bool {
        !matches!(self, InvalidationKind::CompositionOnly)
    }

    #[doc = include_str!("invalidation_kind.doc/needs_recompose.md")]
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
