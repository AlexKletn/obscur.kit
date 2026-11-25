#![doc = include_str!("../../docs/api/orchestration/arrange_pass.doc/_module.md")]

use obscur_geometry::{BoxConstraints, Insets, Point, Rect, Size};
use obscur_identity::NodeId;
use obscur_layout::algorithm::LayoutChild;
use obscur_layout::model::{
    AlignItems, Dimension, Direction, LayoutBox, LayoutDisplay, LayoutStyle, Position, WritingMode,
};
use obscur_text::layout::Hyphenator;
use obscur_text::shaping::ParagraphShaper;
use obscur_tree::tree::RetainedTree;

use super::arrange_cache::ArrangeCache;
use super::arrange_key::ArrangeKey;
use super::formatting_context::{algorithm_for, partition_children, style_of};
use super::layout_coordinator::LayoutBoxes;
use super::layout_coordinator::LayoutStyles;
use super::measure_cache::MeasureCache;
use super::measure_pass::{RecMeasurer, measure};
use super::paint_transform::has_transform;
use super::scroll::overflow_scrolls;
use super::{ComputedStyles, EditingView, GridTemplates};

#[doc = include_str!("../../docs/api/orchestration/arrange_pass.doc/padding_box.md")]
fn padding_box(border_box: Rect, border: Insets) -> Rect {
    let x = border_box.origin.x + border.left;
    let y = border_box.origin.y + border.top;
    let width = (border_box.size.width - border.left - border.right).max(0.0);
    let height = (border_box.size.height - border.top - border.bottom).max(0.0);
    Rect::new(Point::new(x, y), Size::new(width, height))
}

#[doc = include_str!("../../docs/api/orchestration/arrange_pass.doc/arrange.md")]
// The recursive arrange walk threads the tree, both style maps, the target
// origin/constraints, the containing block + viewport for positioning, the
// output map, the shaper, and the depth budget; these are cohesive positional
// arguments of one internal recursion, not a public API.
#[allow(clippy::too_many_arguments)]
pub(super) fn arrange(
    tree: &RetainedTree,
    styles: &LayoutStyles,
    computed: &ComputedStyles,
    grid_templates: &GridTemplates,
    node: NodeId,
    origin: Point,
    constraints: BoxConstraints,
    boxes: &mut LayoutBoxes,
    paragraph: &ParagraphShaper,
    hyphenator: Option<&dyn Hyphenator>,
    containing_block: Rect,
    viewport: Rect,
    scroll_offset: (f32, f32),
    editing: Option<EditingView<'_>>,
    depth: u32,
    cache: &MeasureCache,
    arrange_cache: &ArrangeCache,
    // The style to lay `node` out with instead of its own, for the one node that
    // has no computed style to look up: the synthetic root, which *is* the initial
    // containing block. `None` everywhere else — a recursive call never passes it,
    // because every other node has a style of its own.
    style_override: Option<LayoutStyle>,
    natural_block_size: bool,
) {
    if depth == 0 {
        cache.record_depth_limit(node);
        return;
    }

    // §4.3 "early exit when geometry is stable" / §3.4 step 8. Two conditions, and
    // the [`ArrangeCache`] doc says why neither alone would do: the key proves this
    // caller put the node in the same place at the same size, and a surviving entry
    // proves nothing in its subtree restyled (`layout_invalidated` evicted the
    // upward closure of everything the dirty set can have touched). Both hold ⇒ the
    // subtree's absolute boxes are what the last pass wrote, so serve them and do
    // not descend. `serve` writes nothing and answers `false` if the node has no
    // retained box, which falls through to the real walk below.
    let key = ArrangeKey::new(
        origin,
        constraints,
        containing_block,
        viewport,
        scroll_offset,
    )
    .with_natural_block_size(natural_block_size);
    let key = if styles
        .get(&node)
        .is_some_and(|style| style.padding_lengths.is_some())
    {
        key.with_padding_basis(constraints)
    } else {
        key
    };
    if arrange_cache.get(node) == Some(key) && arrange_cache.serve(tree, node, boxes) {
        return;
    }
    arrange_cache.record(node, key);

    // The node's own style wins wherever it *said* something; the override only
    // fills in what it left `auto`. It has to be that way round twice over: the
    // synthetic root does get a computed style (the style walk starts at it, and
    // nothing matches it, so both axes come back `auto`), and a host that hands the
    // root an explicit size or padding means it — replacing that would be the
    // override deciding layout for a box the caller had already described.
    let mut style = match style_override {
        Some(icb) => {
            let own = style_of(styles, node);
            LayoutStyle {
                width: if own.width.is_auto() {
                    icb.width
                } else {
                    own.width
                },
                height: if own.height.is_auto() {
                    icb.height
                } else {
                    own.height
                },
                ..own
            }
        }
        None => style_of(styles, node),
    };

    style.resolve_padding(if style.writing_mode.is_vertical() {
        constraints.percent_height()
    } else {
        constraints.percent_width()
    });
    if natural_block_size && style.display == LayoutDisplay::Flex {
        // A tight arrangement constraint carries the measured size, but the
        // cyclic percentage in a Flex gap still resolves against zero.
        if let Some(gap) = style.row_gap.as_ref().or(style.gap_dimension.as_ref()) {
            style.row_gap = Some(Dimension::Px(gap.resolve(0.0).unwrap_or(0.0).max(0.0)));
        }
    }

    // A text node is a constraint-aware leaf: shape + wrap to `max.width` and
    // write the wrapped box, bypassing the box algorithms (spec §5.10). It has
    // no laid-out children, so there is nothing to recurse into.
    //
    // The size comes from [`measure`], which is where that shaping lives: this
    // used to inline a byte-identical copy of it, which meant the arrange walk
    // re-shaped every text leaf on every frame no matter what the measure cache
    // held. Routing it through the one implementation is what lets a retained entry
    // actually save the work (spec §4.2 "may cache measure/layout results"), and
    // leaves the size itself unchanged — `measure` returns exactly this branch's
    // answer for a text node.
    if tree.get(node).is_some_and(|n| n.kind().text().is_some()) {
        let size = measure(
            tree,
            styles,
            computed,
            grid_templates,
            node,
            constraints,
            paragraph,
            hyphenator,
            editing,
            depth,
            cache,
        );
        boxes.insert(node, LayoutBox::new(Rect::new(origin, size)));
        return;
    }

    let Some(algorithm) = algorithm_for(style.display, node, grid_templates, style.gap) else {
        return;
    };
    // Out-of-flow children are excluded from the formatting context (they do not
    // shift in-flow siblings or contribute to content size) and positioned
    // separately below. The in-flow list is what the algorithm sees, so its
    // indices map one-to-one to the algorithm's `measure_child(index)`.
    let (children, out_of_flow) = partition_children(tree, styles, node);

    // Scroll container (CSS Overflow Module Level 3 §scroll-container): an
    // `overflow: scroll`/`auto` box with a **definite (finitely-constrained) block
    // size** clips and scrolls its content, so its content lays out at its NATURAL
    // block size and is allowed to overflow the fixed box — the scrollport. The
    // arrange pass constrains every in-flow node to its measured size (a tight
    // block axis), so `constraints.max.height` here IS the scrollport height. Relax
    // the block-axis max to unbounded for CHILD measurement (so the algorithm lays
    // children out at their natural height instead of clamping them to the box),
    // then pin the node's OWN box back to that scrollport height. A scroll
    // container under an unbounded parent (infinite max height) has no definite
    // size — it grows to fit — so it is left unrelaxed. Only `overflow: scroll`/
    // `auto` nodes are touched; every other node is byte-for-byte identical.
    //
    // The natural content height is produced by the **single** layout pass below
    // (`out.size()`), NOT a second measure: the relaxation only changes this one
    // call's block-axis bound, and overflow is later derived from the resulting
    // absolute boxes (`scrollable_content_height`), so the common frame spends zero
    // extra measures here. The relaxation is also skipped when it provably cannot
    // change anything — a childless scroll container has no content to overflow, so
    // its box is identical relaxed or not; keeping it unrelaxed avoids forcing the
    // whole subtree onto the indefinite-height path for nothing.
    // Size containment uses the actual assignment to lay out the children.
    let scroll_relaxed = overflow_scrolls(computed, node)
        && !style.size_containment.contains_height(style.writing_mode)
        && constraints.max.height.is_finite()
        && !children.is_empty();
    let child_constraints = if scroll_relaxed {
        BoxConstraints {
            min: Size::new(constraints.min.width, 0.0),
            max: Size::new(constraints.max.width, f32::INFINITY),
            // The relaxation frees the block axis only; the percentage basis the
            // caller supplied still describes the same containing block.
            percent_basis: constraints.percent_basis,
        }
    } else {
        constraints
    };

    // Measurer is pure: it does not borrow `boxes`, so `boxes` is free after.
    let out = {
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
        algorithm.layout(&style, &children, child_constraints, &mut measurer)
    };

    cache.record_grid_limit(out.grid_limited_item());

    // Now safe to write this node's absolute box. A relaxed scroll container keeps
    // its scrollport height (the tight block constraint) while its content
    // overflows; every other node uses the algorithm's own size verbatim.
    let node_size = if scroll_relaxed {
        Size::new(out.size().width, constraints.max.height)
    } else {
        out.size()
    };
    let border_box = Rect::new(origin, node_size);
    let mut layout_box = LayoutBox::new(border_box);
    if style.padding_lengths.is_some() {
        layout_box = layout_box.with_content_insets(Insets {
            top: style.padding.top + style.border.top,
            right: style.padding.right + style.border.right,
            bottom: style.padding.bottom + style.border.bottom,
            left: style.padding.left + style.border.left,
        });
    }
    boxes.insert(node, layout_box);

    // The containing block this node establishes for its positioned descendants:
    // a positioned node (`relative`/`absolute`/`fixed`) — OR any node carrying a
    // `transform` other than `none` (css-transforms-1 §3) — hands down its own
    // padding box; a plain `static` node is transparent and passes the inherited
    // block through (so an absolute grandchild resolves against the nearest
    // positioned/transformed ancestor, not this static node).
    let establishes_cb_by_transform = computed.get(&node).is_some_and(has_transform);
    let node_padding_box = padding_box(border_box, style.border);
    let descendant_cb = if style.position.is_positioned() || establishes_cb_by_transform {
        node_padding_box
    } else {
        containing_block
    };
    // A `transform` other than `none` ALSO becomes the containing block for
    // `position: fixed` descendants, which otherwise resolve against `viewport`
    // (css-transforms-1 §3). Without a transform the viewport rides through
    // unchanged, so an untransformed subtree is byte-for-byte identical.
    let descendant_fixed_cb = if establishes_cb_by_transform {
        node_padding_box
    } else {
        viewport
    };

    // The containing block for a relatively-positioned child is this node's
    // content box; we approximate its size with this node's border-box size
    // (`out.size()`), which is exact when the node has no border/padding and is
    // the only containing-block size available at this point in the walk. Only a
    // definite percentage inset is affected by the approximation; a `px` inset is
    // exact.
    let containing = out.size();
    // The containing block a child's percentage size resolves against: **this**
    // node's content box, not the border box the child is arranged at. Stating it
    // is what stops the child from re-resolving `width: 100%` against its own
    // arranged size and coming back one padding wider at every level (CSS 2.1
    // §10.2; Servo's `SizeConstraint::Definite`).
    //
    // This is the containing block for a block child and for a flex item, but NOT
    // for every formatting context — a grid item is contained by its grid area
    // (`specs/w3c/css-grid-2.txt:712`), a rectangle only the grid algorithm knows.
    // So this is the **fallback**: where the algorithm stated a basis of its own
    // for a child (`LayoutOutput::child_percent_basis`, per axis), that wins, and
    // this fills in the axes it left alone.
    let percent_basis = Size::new(
        (containing.width
            - style.padding.left
            - style.padding.right
            - style.border.left
            - style.border.right)
            .max(0.0),
        (containing.height
            - style.padding.top
            - style.padding.bottom
            - style.border.top
            - style.border.bottom)
            .max(0.0),
    );

    // The scroll container's visible rect in document coordinates: the viewport
    // translated by the current scroll offset. Its top-left is the doc-coord point
    // now at the top-left of the viewport, so a `position: sticky` child clamps its
    // used position against this rect's edges.
    let scrollport = Rect::new(
        Point::new(
            viewport.origin.x + scroll_offset.0,
            viewport.origin.y + scroll_offset.1,
        ),
        viewport.size,
    );
    let block_size_definite = style.height.resolve(constraints.percent_height()).is_some()
        || (!natural_block_size
            && constraints.max.height.is_finite()
            && constraints.min.height >= constraints.max.height);
    // Zipping bounds both sides, so a mismatched length never indexes out of range.
    for (index, (child, child_rel)) in children.iter().zip(out.child_boxes().iter()).enumerate() {
        let rel = child_rel.origin();
        let child_style = style_of(styles, child.node());
        // What THIS child's percentages resolve against: the algorithm's own
        // statement where it made one, this node's content box everywhere else.
        // For a grid item the statement is its grid area's inline size, and it is
        // the whole of the fix for the audit's 2-4 (a): the item's used width came
        // from its track, so its subtree has to inherit the track — not the grid
        // container's content box, a containing block the item never had.
        let child_basis = out.child_percent_basis(index).resolve(percent_basis);
        let static_origin = Point::new(origin.x + rel.x, origin.y + rel.y);
        // A `position: sticky` child stays IN flow (its flow box, and therefore its
        // siblings, are untouched — siblings use the unchanged flow boxes), but its
        // used position is scroll-clamped to the scrollport edge within its
        // containing block; the delta rides its whole subtree, exactly like the
        // relative offset. A `position: relative` child instead offsets by its
        // resolved insets. Any other (`static`/out-of-flow) child yields `(0, 0)`,
        // keeping the walk byte-identical to before. When `scroll_offset` is
        // `(0, 0)` and a sticky child is still below its inset threshold, the sticky
        // delta is `(0, 0)` too, so the default path is unchanged.
        let (dx, dy) = if child_style.position.is_sticky() {
            child_style.inset.sticky_offset(
                static_origin,
                child_rel.size(),
                containing_block,
                scrollport,
            )
        } else {
            child_style.relative_offset(containing.width, containing.height)
        };
        let stretches_block = child_style.align_self.resolve(style.align_items)
            == AlignItems::Stretch
            && !child_style.margin.top.is_auto()
            && !child_style.margin.bottom.is_auto();
        let child_origin = Point::new(static_origin.x + dx, static_origin.y + dy);
        arrange(
            tree,
            styles,
            computed,
            grid_templates,
            child.node(),
            child_origin,
            BoxConstraints::tight(child_rel.size()).with_percent_basis(child_basis),
            boxes,
            paragraph,
            hyphenator,
            descendant_cb,
            descendant_fixed_cb,
            scroll_offset,
            editing,
            depth - 1,
            cache,
            arrange_cache,
            // Every node below the root has a computed style of its own.
            None,
            child_style.height.is_auto()
                && match style.display {
                    LayoutDisplay::Block => true,
                    LayoutDisplay::Flex if style.flex_direction.is_row() => {
                        !block_size_definite || style.flex_wrap.is_multi_line() || !stretches_block
                    }
                    LayoutDisplay::Flex => {
                        !block_size_definite
                            && child_style.flex_basis.resolve(f32::INFINITY).is_none()
                    }
                    LayoutDisplay::Grid => !stretches_block,
                    _ => false,
                },
        );
    }

    for child in &out_of_flow {
        arrange_out_of_flow(
            tree,
            styles,
            computed,
            grid_templates,
            child.clone(),
            descendant_cb,
            descendant_fixed_cb,
            scroll_offset,
            boxes,
            paragraph,
            hyphenator,
            editing,
            depth - 1,
            cache,
            arrange_cache,
        );
    }
}

#[doc = include_str!("../../docs/api/orchestration/arrange_pass.doc/arrange_out_of_flow.md")]
// The out-of-flow placement threads the same cohesive recursion inputs as
// [`arrange`]; they are one internal positioning step, not a public API.
#[allow(clippy::too_many_arguments)]
fn arrange_out_of_flow(
    tree: &RetainedTree,
    styles: &LayoutStyles,
    computed: &ComputedStyles,
    grid_templates: &GridTemplates,
    child: LayoutChild,
    parent_cb: Rect,
    viewport: Rect,
    scroll_offset: (f32, f32),
    boxes: &mut LayoutBoxes,
    paragraph: &ParagraphShaper,
    hyphenator: Option<&dyn Hyphenator>,
    editing: Option<EditingView<'_>>,
    depth: u32,
    cache: &MeasureCache,
    arrange_cache: &ArrangeCache,
) {
    if depth == 0 {
        cache.record_depth_limit(child.node());
        return;
    }
    let mut style = child.style().clone();
    // Fixed positions against the viewport, ignoring ancestors; absolute against
    // the nearest positioned ancestor's padding box.
    let cb = if matches!(style.position, Position::Fixed) {
        viewport
    } else {
        parent_cb
    };

    let mut ancestor = tree.visual_parent_of(child.node());
    let mut axes = (WritingMode::HorizontalTb, Direction::Ltr);
    for _ in 0..tree.len() {
        let Some(node) = ancestor else {
            break;
        };
        if let Some(ancestor_style) = styles.get(&node)
            && (computed.get(&node).is_some_and(has_transform)
                || (!matches!(style.position, Position::Fixed)
                    && ancestor_style.position.is_positioned()))
        {
            axes = (ancestor_style.writing_mode, ancestor_style.direction);
            break;
        }
        ancestor = tree.visual_parent_of(node);
    }
    style.resolve_padding(if axes.0.is_vertical() {
        cb.size.height
    } else {
        cb.size.width
    });

    // Measure only if at least one axis uses intrinsic size. Width and
    // height both resolve before arrange for fixed-size overlays and clips.
    let uses_natural = |extent, size: Dimension, start: Dimension, end: Dimension| {
        size.resolve(extent).is_none()
            && (start.resolve(extent).is_none() || end.resolve(extent).is_none())
    };
    let natural = if uses_natural(
        cb.size.width,
        style.width.clone(),
        style.inset.left.clone(),
        style.inset.right.clone(),
    ) || uses_natural(
        cb.size.height,
        style.height.clone(),
        style.inset.top.clone(),
        style.inset.bottom.clone(),
    ) {
        measure(
            tree,
            styles,
            computed,
            grid_templates,
            child.node(),
            BoxConstraints::loose(cb.size),
            paragraph,
            hyphenator,
            editing,
            depth,
            cache,
        )
    } else {
        Size::ZERO
    };
    // The per-axis border + padding sum, so an explicit `content-box` width/height
    // grows into the border box `resolve_absolute_box` returns (matching the flow
    // path's `box_sizing.content_extent`). `natural` is already a border box.
    let border_padding = Size::new(
        style.border.left + style.border.right + style.padding.left + style.padding.right,
        style.border.top + style.border.bottom + style.padding.top + style.padding.bottom,
    );
    let rect = style
        .inset
        .resolve_absolute_style(cb, &style, natural, border_padding, axes);

    // Arrange the child's subtree under a tight constraint at its resolved box.
    // The recursive `arrange` re-derives and writes the child's own box (identical
    // to `rect` under the tight constraint) and, because the child is positioned,
    // establishes its padding box for its own positioned descendants. Keep
    // the inherited block only when a direct sticky child uses it. Otherwise
    // an animated clipping ancestor's width must not invalidate this box.
    let has_sticky_child = tree.get(child.node()).is_some_and(|record| {
        record.child_nodes().iter().any(|node| {
            styles
                .get(node)
                .is_some_and(|style| style.position.is_sticky())
        })
    });
    let inherited_block = if has_sticky_child {
        cb
    } else {
        Rect::new(Point::new(0.0, 0.0), Size::ZERO)
    };
    arrange(
        tree,
        styles,
        computed,
        grid_templates,
        child.node(),
        rect.origin,
        BoxConstraints::tight(rect.size).with_percent_basis(cb.size),
        boxes,
        paragraph,
        hyphenator,
        inherited_block,
        viewport,
        scroll_offset,
        editing,
        depth,
        cache,
        arrange_cache,
        // Every node below the root has a computed style of its own.
        None,
        false,
    );
}
