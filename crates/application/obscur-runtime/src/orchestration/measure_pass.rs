#![doc = include_str!("measure_pass.doc/_module.md")]

use obscur_geometry::{BoxConstraints, Size};
use obscur_identity::NodeId;
use obscur_layout::algorithm::{ChildMeasurer, LayoutChild};
use obscur_text::layout::{Hyphenator, TextOverflow};
use obscur_text::shaping::ParagraphShaper;
use obscur_tree::tree::RetainedTree;

use super::editing_override;
use super::formatting_context::{algorithm_for, in_flow_children, style_of};
use super::layout_coordinator::LayoutStyles;
use super::length_resolution::root_font_size;
use super::measure_cache::{MeasureCache, measure_key};
use super::text_leaf_measure::measure_text_node;
use super::{
    ComputedStyles, EditingView, GridTemplates, alignment_for, attributes_for, direction_for,
    display_text, families_for, letter_spacing_for, line_height_for, overflow_for,
    overflow_wrap_for, tab_size_for, text_transform_for, white_space_for, word_break_for,
    word_spacing_for,
};

#[doc = include_str!("measure_pass.doc/RecMeasurer.md")]
pub(super) struct RecMeasurer<'a> {
    #[doc = include_str!("measure_pass.doc/tree.md")]
    pub(super) tree: &'a RetainedTree,
    #[doc = include_str!("measure_pass.doc/styles.md")]
    pub(super) styles: &'a LayoutStyles,
    #[doc = include_str!("measure_pass.doc/computed.md")]
    pub(super) computed: &'a ComputedStyles,
    #[doc = include_str!("measure_pass.doc/grid_templates.md")]
    pub(super) grid_templates: &'a GridTemplates,
    #[doc = include_str!("measure_pass.doc/children.md")]
    pub(super) children: &'a [LayoutChild],
    #[doc = include_str!("measure_pass.doc/paragraph.md")]
    pub(super) paragraph: &'a ParagraphShaper,
    #[doc = include_str!("measure_pass.doc/hyphenator.md")]
    pub(super) hyphenator: Option<&'a dyn Hyphenator>,
    #[doc = include_str!("measure_pass.doc/editing.md")]
    pub(super) editing: Option<EditingView<'a>>,
    #[doc = include_str!("measure_pass.doc/depth.md")]
    pub(super) depth: u32,
    #[doc = include_str!("measure_pass.doc/cache.md")]
    pub(super) cache: &'a MeasureCache,
}

impl ChildMeasurer for RecMeasurer<'_> {
    fn measure_child(&mut self, index: usize, constraints: BoxConstraints) -> Size {
        match self.children.get(index) {
            Some(child) => measure(
                self.tree,
                self.styles,
                self.computed,
                self.grid_templates,
                child.node(),
                constraints,
                self.paragraph,
                self.hyphenator,
                self.editing,
                self.depth,
                self.cache,
            ),
            // Out-of-range index: contribute nothing rather than panic.
            None => Size::ZERO,
        }
    }
}

#[doc = include_str!("measure_pass.doc/overflow_for_text.md")]
fn overflow_for_text(tree: &RetainedTree, computed: &ComputedStyles, node: NodeId) -> TextOverflow {
    let container = tree
        .get(node)
        .and_then(|n| n.logical_parent().parent())
        .unwrap_or(node);
    overflow_for(computed.get(&container))
}

#[doc = include_str!("measure_pass.doc/measure.md")]
// The tree, both style maps, the constraints, the shaper, the hyphenator, and
// the depth budget are cohesive positional arguments of one internal recursion.
#[allow(clippy::too_many_arguments)]
pub(super) fn measure(
    tree: &RetainedTree,
    styles: &LayoutStyles,
    computed: &ComputedStyles,
    grid_templates: &GridTemplates,
    node: NodeId,
    constraints: BoxConstraints,
    paragraph: &ParagraphShaper,
    hyphenator: Option<&dyn Hyphenator>,
    editing: Option<EditingView<'_>>,
    depth: u32,
    cache: &MeasureCache,
) -> Size {
    if depth == 0 {
        return Size::ZERO;
    }
    // Memoize on (node, constraints): the arrange walk measures the same node
    // under the same constraints many times per frame; return the first result.
    let key = measure_key(node, constraints);
    if let Some(hit) = cache.get(&key) {
        return hit;
    }
    let style = style_of(styles, node);
    // A text node is a constraint-aware leaf: shape + wrap to `max.width`,
    // bypassing the box algorithms entirely (spec §5.10).
    if let Some(content) = tree.get(node).and_then(|n| n.kind().text()) {
        let fallback = families_for(computed.get(&node));
        let attributes = attributes_for(computed.get(&node));
        let direction = direction_for(computed.get(&node));
        let alignment = alignment_for(computed.get(&node));
        let overflow = overflow_for_text(tree, computed, node);
        // The document root's `font-size` resolves any `rem` in the text metrics
        // below (CSS Values 4 §5.1.1); each metric's `em` / `%` resolves against
        // this node's own `style.font_size`.
        let root_font = root_font_size(computed, tree.root());
        let line_height = line_height_for(computed.get(&node), style.font_size, root_font);
        let white_space = white_space_for(computed.get(&node));
        // The string to shape, via the SAME `display_text` seam the scene phase
        // composes with (CSS Text 3 §4.1.1): while this node is the focused
        // editable's text, the live session buffer is shaped RAW (the field's real
        // `white-space`, `text-transform`/collapsing skipped for exact caret byte
        // offsets); otherwise `text-transform` (§2.1) then `white-space` collapsing
        // (§4.1.1) rewrite the retained string BEFORE shaping. Either way the box is
        // sized from the identical characters the compose pass paints, so its height
        // (line count) matches the painted lines. Letter-/word-spacing and tab stops
        // are applied post-shaping inside `measure_text_node`.
        let active = editing_override(tree, node, editing);
        let content = display_text(
            content,
            active,
            text_transform_for(computed.get(&node)),
            white_space,
        );
        let size = measure_text_node(
            content.as_ref(),
            &style,
            &fallback,
            &attributes,
            direction,
            alignment,
            overflow,
            white_space,
            overflow_wrap_for(computed.get(&node)),
            word_break_for(computed.get(&node)),
            constraints,
            paragraph,
            hyphenator,
            line_height,
            letter_spacing_for(computed.get(&node), style.font_size, root_font),
            word_spacing_for(computed.get(&node), style.font_size, root_font),
            tab_size_for(computed.get(&node), style.font_size, root_font),
        );
        cache.record(key, node, size);
        return size;
    }
    let Some(algorithm) = algorithm_for(style.display, node, grid_templates, style.gap) else {
        return Size::ZERO;
    };
    // Out-of-flow (absolute/fixed) children are excluded from the formatting
    // context: they contribute nothing to this node's content size. Only the
    // in-flow children reach the algorithm, and the measurer's indices are over
    // that in-flow list.
    let children = in_flow_children(tree, styles, node);
    let mut measurer = RecMeasurer {
        tree,
        styles,
        computed,
        grid_templates,
        children: &children,
        paragraph,
        hyphenator,
        editing,
        depth: depth - 1,
        cache,
    };
    let out = algorithm.layout(&style, &children, constraints, &mut measurer);
    let size = out.size();
    cache.record(key, node, size);
    size
}
