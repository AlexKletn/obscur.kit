//! Whether a text edit can skip style and layout: [`ComposeOnlyEdit`].

use obscur_identity::NodeId;
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{Length, PropertyId};
use obscur_text::layout::TextAlignment;
use obscur_text::shaping::TextDirection;
use obscur_tree::tree::RetainedTree;

use super::EditableField;
use crate::orchestration::{ComputedStyles, alignment_for, direction_for, white_space_for};

#[doc = include_str!("../../docs/api/runtime/compose_only_edit.doc/ComposeOnlyEdit.md")]
pub(crate) struct ComposeOnlyEdit<'a> {
    /// The retained tree: whether the field is a `<textarea>`, and where its text
    /// leaf is.
    tree: &'a RetainedTree,
    /// This frame's computed styles: the white-space, alignment, direction and
    /// width the three conditions are read from.
    computed: &'a ComputedStyles,
}

impl<'a> ComposeOnlyEdit<'a> {
    /// Bundles the retained frame for one box-stability question.
    pub(crate) const fn new(tree: &'a RetainedTree, computed: &'a ComputedStyles) -> Self {
        Self { tree, computed }
    }

    #[doc = include_str!("../../docs/api/runtime/compose_only_edit.doc/is_box_stable.md")]
    pub(crate) fn is_box_stable(&self, field: NodeId) -> bool {
        // A multiline field grows a line box when a line is added: not box-stable.
        if EditableField::is_multiline(self.tree, field) {
            return false;
        }
        let Some(text_node) = EditableField::first_text_descendant(self.tree, field) else {
            return false;
        };
        let text_style = self.computed.get(&text_node);
        // A wrapping field reflows (its height tracks the wrapped line count).
        if white_space_for(text_style).wraps() {
            return false;
        }
        // A width-dependent alignment would shift the run when the stale text-node
        // box width differs from the (grown) content width.
        if !alignment_ignores_width(alignment_for(text_style), direction_for(text_style)) {
            return false;
        }
        // The field (and thus its ancestors) must be definite-sized so the typed
        // content cannot propagate a size change out to any reused box.
        has_definite_px_width(self.computed.get(&field))
    }
}

#[doc = include_str!("../../docs/api/runtime/compose_only_edit.doc/alignment_ignores_width.md")]
fn alignment_ignores_width(alignment: TextAlignment, direction: TextDirection) -> bool {
    match alignment {
        TextAlignment::Left | TextAlignment::Justify => true,
        TextAlignment::Start => !direction.is_rtl(),
        TextAlignment::End => direction.is_rtl(),
        TextAlignment::Right | TextAlignment::Center => false,
    }
}

#[doc = include_str!("../../docs/api/runtime/compose_only_edit.doc/has_definite_px_width.md")]
fn has_definite_px_width(style: Option<&ComputedStyle>) -> bool {
    style
        .and_then(|s| s.get(PropertyId::Width))
        .and_then(ComputedValue::as_length)
        .and_then(Length::as_px)
        .is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_field_the_frame_does_not_carry_is_never_box_stable() {
        // No text leaf and no style: the unknown answers conservatively, so a
        // session on a node this frame dropped takes the full frame.
        let tree = RetainedTree::new();
        let computed = ComputedStyles::default();
        assert!(!ComposeOnlyEdit::new(&tree, &computed).is_box_stable(NodeId::from_raw(3)));
    }

    #[test]
    fn only_a_leading_edge_alignment_is_width_independent() {
        assert!(alignment_ignores_width(
            TextAlignment::Left,
            TextDirection::LeftToRight
        ));
        assert!(alignment_ignores_width(
            TextAlignment::Justify,
            TextDirection::RightToLeft
        ));
        assert!(alignment_ignores_width(
            TextAlignment::Start,
            TextDirection::LeftToRight
        ));
        assert!(!alignment_ignores_width(
            TextAlignment::Start,
            TextDirection::RightToLeft
        ));
        assert!(alignment_ignores_width(
            TextAlignment::End,
            TextDirection::RightToLeft
        ));
        assert!(!alignment_ignores_width(
            TextAlignment::Center,
            TextDirection::LeftToRight
        ));
        assert!(!alignment_ignores_width(
            TextAlignment::Right,
            TextDirection::LeftToRight
        ));
    }

    #[test]
    fn a_missing_style_has_no_definite_width() {
        assert!(!has_definite_px_width(None));
    }
}
