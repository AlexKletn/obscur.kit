#![doc = include_str!("arrange_pass.doc/_module.md")]

use obscur_geometry::{BoxConstraints, Insets, Point, Rect, Size};
use obscur_identity::NodeId;
use obscur_layout::algorithm::LayoutChild;
use obscur_layout::model::{LayoutBox, LayoutStyle, Position};
use obscur_style::computed::ComputedValue;
use obscur_style::property::PropertyId;
use obscur_text::layout::Hyphenator;
use obscur_text::shaping::ParagraphShaper;
use obscur_tree::tree::RetainedTree;

use super::arrange_cache::{ArrangeCache, ArrangeKey};
use super::formatting_context::{algorithm_for, partition_children, style_of};
use super::layout_coordinator::LayoutBoxes;
use super::layout_coordinator::LayoutStyles;
use super::measure_cache::MeasureCache;
use super::measure_pass::{RecMeasurer, measure};
use super::scroll::overflow_scrolls;
use super::{ComputedStyles, EditingView, GridTemplates};

#[doc = include_str!("arrange_pass.doc/transform_establishes_containing_block.md")]
fn transform_establishes_containing_block(computed: &ComputedStyles, node: NodeId) -> bool {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::Transform))
        .and_then(ComputedValue::as_transform)
        .is_some_and(|transform| transform.establishes_containing_block())
}

#[doc = include_str!("arrange_pass.doc/padding_box.md")]
fn padding_box(border_box: Rect, border: Insets) -> Rect {
    let x = border_box.origin.x + border.left;
    let y = border_box.origin.y + border.top;
    let width = (border_box.size.width - border.left - border.right).max(0.0);
    let height = (border_box.size.height - border.top - border.bottom).max(0.0);
    Rect::new(Point::new(x, y), Size::new(width, height))
}

#[doc = include_str!("arrange_pass.doc/arrange.md")]
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
) {
    if depth == 0 {
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
    );
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
    let style = match style_override {
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
    let scroll_relaxed = overflow_scrolls(computed, node)
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

    // Now safe to write this node's absolute box. A relaxed scroll container keeps
    // its scrollport height (the tight block constraint) while its content
    // overflows; every other node uses the algorithm's own size verbatim.
    let node_size = if scroll_relaxed {
        Size::new(out.size().width, constraints.max.height)
    } else {
        out.size()
    };
    let border_box = Rect::new(origin, node_size);
    boxes.insert(node, LayoutBox::new(border_box));

    // The containing block this node establishes for its positioned descendants:
    // a positioned node (`relative`/`absolute`/`fixed`) — OR any node carrying a
    // `transform` other than `none` (css-transforms-1 §3) — hands down its own
    // padding box; a plain `static` node is transparent and passes the inherited
    // block through (so an absolute grandchild resolves against the nearest
    // positioned/transformed ancestor, not this static node).
    let establishes_cb_by_transform = transform_establishes_containing_block(computed, node);
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

    // Recurse into in-flow children, translating each parent-relative box to
    // absolute. Zipping bounds both sides, so a mismatched length never indexes
    // out of range.
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
        // The static (normal-flow) absolute origin the algorithm computed for this
        // child, before any positioning offset.
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
        );
    }

    // Position each out-of-flow child within its containing block, then recurse
    // into its subtree as an independent positioned root.
    for child in &out_of_flow {
        arrange_out_of_flow(
            tree,
            styles,
            computed,
            grid_templates,
            *child,
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

#[doc = include_str!("arrange_pass.doc/arrange_out_of_flow.md")]
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
        return;
    }
    let style = child.style();
    // Fixed positions against the viewport, ignoring ancestors; absolute against
    // the nearest positioned ancestor's padding box.
    let cb = if matches!(style.position, Position::Fixed) {
        viewport
    } else {
        parent_cb
    };

    // Natural (content) size: measure loosely up to the containing block. This is
    // the used size on an axis that is not pinned by both insets.
    let natural = measure(
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
    );
    // The per-axis border + padding sum, so an explicit `content-box` width/height
    // grows into the border box `resolve_absolute_box` returns (matching the flow
    // path's `box_sizing.content_extent`). `natural` is already a border box.
    let border_padding = Size::new(
        style.border.left + style.border.right + style.padding.left + style.padding.right,
        style.border.top + style.border.bottom + style.padding.top + style.padding.bottom,
    );
    let rect = style.inset.resolve_absolute_box(
        cb,
        style.width,
        style.height,
        natural,
        style.box_sizing,
        border_padding,
    );

    // Arrange the child's subtree under a tight constraint at its resolved box.
    // The recursive `arrange` re-derives and writes the child's own box (identical
    // to `rect` under the tight constraint) and, because the child is positioned,
    // establishes its padding box as the containing block for its descendants; the
    // `containing_block` argument we pass is therefore only a placeholder it
    // overrides.
    arrange(
        tree,
        styles,
        computed,
        grid_templates,
        child.node(),
        rect.origin,
        BoxConstraints::tight(rect.size),
        boxes,
        paragraph,
        hyphenator,
        cb,
        viewport,
        scroll_offset,
        editing,
        depth,
        cache,
        arrange_cache,
        // Every node below the root has a computed style of its own.
        None,
    );
}
