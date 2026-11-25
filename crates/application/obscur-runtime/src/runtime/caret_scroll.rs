//! Keeping the caret visible in a scrolling field: [`CaretScroll`].

use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{Length, PropertyId};
use obscur_tree::tree::RetainedTree;

use super::{EditableField, TextEditSession};
use crate::orchestration::{
    CARET_WIDTH, ComputedStyles, LayoutBoxes, SceneCoordinator, caret_line_scroll_geometry,
    display_text, line_caret_x, line_of_byte, overflow_scrolls, text_transform_for,
    white_space_for,
};

#[doc = include_str!("../../docs/api/runtime/caret_scroll.doc/CaretScroll.md")]
pub(crate) struct CaretScroll<'a> {
    /// The retained tree the field's text leaf is found in.
    tree: &'a RetainedTree,
    /// This frame's computed styles: white-space, overflow, the field's own
    /// padding/border insets and its definite height.
    computed: &'a ComputedStyles,
    /// This frame's arranged boxes, which supply the scrollport dimensions.
    boxes: &'a LayoutBoxes,
    /// The scene's paragraph orchestrator — the SAME advance and wrapped-line
    /// model the compose pass paints the caret with.
    composer: &'a SceneCoordinator,
}

impl<'a> CaretScroll<'a> {
    /// Bundles what one caret-follow recomputation measures against.
    pub(crate) fn new(
        tree: &'a RetainedTree,
        computed: &'a ComputedStyles,
        boxes: &'a LayoutBoxes,
        composer: &'a SceneCoordinator,
    ) -> Self {
        Self {
            tree,
            computed,
            boxes,
            composer,
        }
    }

    #[doc = include_str!("../../docs/api/runtime/caret_scroll.doc/horizontal.md")]
    pub(crate) fn horizontal(&self, session: &TextEditSession) -> Option<f32> {
        let field = session.node();
        let text_node = EditableField::first_text_descendant(self.tree, field)?;
        // Wrapping (multi-line) fields never scroll horizontally: pin to 0.
        if white_space_for(self.computed.get(&text_node)).wraps() {
            return Some(0.0);
        }
        // No retained box yet ⇒ nothing to scroll into.
        let field_box = self.boxes.get(&field)?;
        let (scrollport_left, scrollport_width) = field_box.used_content_rect().map_or_else(
            || {
                let style = self.computed.get(&field);
                let left = field_box.origin().x
                    + px_length_of(style, PropertyId::PaddingLeft)
                    + px_length_of(style, PropertyId::BorderLeftWidth);
                (left, field_content_width(style, field_box.size().width))
            },
            |content| (content.origin.x, content.size.width),
        );
        let text_box = self.boxes.get(&text_node)?;
        let style = self.computed.get(&text_node);
        let view = session.view();
        let text = display_text(
            view.text,
            Some(view),
            text_transform_for(style),
            white_space_for(style),
        );
        let focus = view
            .preedit
            .map_or(view.selection.focus(), |preedit| preedit.at + preedit.caret);
        let layout = self.composer.text().shape_editable_lines(
            self.tree,
            self.computed,
            self.boxes,
            text_node,
            &text,
            text_box.size().width,
        );
        let Some(line) = layout.lines().get(line_of_byte(&layout, focus)) else {
            return Some(0.0);
        };
        let line_left = text_box.origin().x - scrollport_left + line.x_offset();
        let caret_x = line_left + line_caret_x(line, focus);
        let caret_right = caret_x + CARET_WIDTH.min(scrollport_width);
        let content_width = (line_left + line.width()).max(caret_right);
        let mut scroll_left = session.scroll_left();
        // Scroll-into-view: keep the caret within
        // `[scroll_left, scroll_left + scrollport_width]`.
        if caret_x - scroll_left < 0.0 {
            scroll_left = caret_x;
        }
        if caret_right - scroll_left > scrollport_width {
            scroll_left = caret_right - scrollport_width;
        }
        // Never scroll past the text (and never negative).
        Some(scroll_left.clamp(0.0, (content_width - scrollport_width).max(0.0)))
    }

    #[doc = include_str!("../../docs/api/runtime/caret_scroll.doc/vertical.md")]
    pub(crate) fn vertical(&self, session: &TextEditSession, stored: f32) -> Option<f32> {
        let field = session.node();
        // Only a multiline field with a scrolling, definite-height box scrolls
        // vertically with the caret — the fixed-height `<textarea>` case.
        if !EditableField::is_multiline(self.tree, field)
            || !overflow_scrolls(self.computed, field)
            || !has_definite_px_height(self.computed.get(&field))
        {
            return None;
        }
        let text_node = EditableField::first_text_descendant(self.tree, field)?;
        let (text_box, field_box) = (self.boxes.get(&text_node)?, self.boxes.get(&field)?);
        // A definite height keeps `field_box` stable across typing, so this is exact
        // even before the grown frame lays out.
        let scrollport_height = field_box.used_content_rect().map_or_else(
            || field_content_height(self.computed.get(&field), field_box.size().height),
            |content| content.size.height,
        );
        // The caret's line box top/bottom and the wrapped content height, measured
        // from the text box origin over the live session text — the frame the
        // container's `translate(0, −scroll_top)` shifts the painted line by.
        let focus = session.selection().focus();
        let (line_top, line_height, content_height) = caret_line_scroll_geometry(
            self.composer.text(),
            self.tree,
            self.computed,
            self.boxes,
            text_node,
            session.text(),
            text_box.size().width,
            focus,
        );
        let caret_top = line_top;
        let caret_bottom = line_top + line_height;
        let mut scroll_top = stored;
        // Scroll-into-view: keep the caret's line within
        // `[scroll_top, scroll_top + scrollport_height]`.
        if caret_top - scroll_top < 0.0 {
            scroll_top = caret_top;
        }
        if caret_bottom - scroll_top > scrollport_height {
            scroll_top = caret_bottom - scrollport_height;
        }
        // Never scroll past the content (and never negative); a fitting textarea
        // (`content_height ≤ scrollport_height`) resolves to `0` — byte-identical,
        // unscrolled.
        Some(scroll_top.clamp(0.0, (content_height - scrollport_height).max(0.0)))
    }
}

#[doc = include_str!("../../docs/api/runtime/caret_scroll.doc/field_content_width.md")]
fn field_content_width(style: Option<&ComputedStyle>, border_box_width: f32) -> f32 {
    let pad_left = px_length_of(style, PropertyId::PaddingLeft);
    let pad_right = px_length_of(style, PropertyId::PaddingRight);
    let border_left = px_length_of(style, PropertyId::BorderLeftWidth);
    let border_right = px_length_of(style, PropertyId::BorderRightWidth);
    (border_box_width - pad_left - pad_right - border_left - border_right).max(0.0)
}
/// The field's visible content **height** (its vertical scrollport) — the border
/// box height minus the top/bottom padding + border edges. The VERTICAL analogue
/// of [`field_content_width`]: the height a multiline `<textarea>`'s caret line
/// must stay within as it scrolls vertically (CSS Overflow 3 §3.1). Never
/// negative.
fn field_content_height(style: Option<&ComputedStyle>, border_box_height: f32) -> f32 {
    let pad_top = px_length_of(style, PropertyId::PaddingTop);
    let pad_bottom = px_length_of(style, PropertyId::PaddingBottom);
    let border_top = px_length_of(style, PropertyId::BorderTopWidth);
    let border_bottom = px_length_of(style, PropertyId::BorderBottomWidth);
    (border_box_height - pad_top - pad_bottom - border_top - border_bottom).max(0.0)
}
/// Reads one computed length property in absolute `px`, defaulting to `0.0` when
/// the property is absent or is a non-pixel length (mirroring the scene
/// coordinator's pixel-only inset reads).
fn px_length_of(style: Option<&ComputedStyle>, id: PropertyId) -> f32 {
    style
        .and_then(|s| s.get(id))
        .and_then(ComputedValue::as_length)
        .and_then(Length::as_px)
        .map_or(0.0, |px| px.0)
}
#[doc = include_str!("../../docs/api/runtime/caret_scroll.doc/has_definite_px_height.md")]
fn has_definite_px_height(style: Option<&ComputedStyle>) -> bool {
    style
        .and_then(|s| s.get(PropertyId::Height))
        .and_then(ComputedValue::as_length)
        .and_then(Length::as_px)
        .is_some()
}

#[cfg(test)]
mod tests {
    use obscur_identity::NodeId;

    use super::*;

    /// A field the tree knows nothing about: no text leaf, no box, no styles.
    fn detached() -> (RetainedTree, ComputedStyles, LayoutBoxes, SceneCoordinator) {
        (
            RetainedTree::new(),
            ComputedStyles::default(),
            LayoutBoxes::default(),
            SceneCoordinator::new(),
        )
    }

    #[test]
    fn a_field_with_no_text_leaf_reports_no_offset_on_either_axis() {
        // Both axes answer `None` — "leave it alone" — rather than `Some(0.0)`,
        // which would silently reset a field the frame simply could not measure.
        let (tree, computed, boxes, composer) = detached();
        let caret = CaretScroll::new(&tree, &computed, &boxes, &composer);
        let session = TextEditSession::seeded(NodeId::from_raw(9), "hello");
        assert_eq!(caret.horizontal(&session), None);
        assert_eq!(caret.vertical(&session, 12.0), None);
    }

    #[test]
    fn a_field_that_is_not_multiline_never_scrolls_vertically() {
        // The vertical axis is the fixed-height `<textarea>` case only, so a plain
        // field keeps whatever offset it had rather than being pulled to the caret.
        let (tree, computed, boxes, composer) = detached();
        let caret = CaretScroll::new(&tree, &computed, &boxes, &composer);
        let session = TextEditSession::seeded(NodeId::from_raw(9), "one\ntwo");
        assert!(!EditableField::is_multiline(&tree, session.node()));
        assert_eq!(caret.vertical(&session, 40.0), None);
    }
}
