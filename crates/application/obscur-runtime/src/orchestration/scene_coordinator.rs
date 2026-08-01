#![doc = include_str!("scene_coordinator.doc/_module.md")]

use std::collections::{HashMap, HashSet};
use std::fmt;

use obscur_identity::NodeId;
use obscur_scene::clip::Clip;
use obscur_scene::command::{DisplayList, DisplayListBuilder, PaintCommand};
use obscur_scene::paint::{Brush, SceneImage, SceneVector, Stroke};
use obscur_scene::path::Path;
use obscur_style::property::FilterOp;
use obscur_text::font::FontResolver;
use obscur_text::layout::Hyphenator;
use obscur_text::shaping::{ParagraphShaper, TextItemizer, TextShaper};
use obscur_tree::tree::RetainedTree;

use super::background_paint::{
    background_color_of, background_gradient_brush, background_image_geometry,
    background_image_url_of, background_layers_of, paint_background_layer,
};
use super::border_paint::{
    border_fills, corner_radii_of, dashed_border_path, dashed_border_pattern, outline_paint,
    uniform_border, uniform_border_ring, uniform_dashed_border,
};
use super::box_shadow_paint::{box_shadows_of, emit_box_shadow};
use super::composite_effects::{
    backdrop_blur_of, blend_mode_of, filter_blur_of, filter_color_matrix_of, filter_ops_of,
    opacity_of,
};
use super::paint_color::{current_color_of, scene_color_from};
use super::paint_transform::resolved_transform;
use super::paragraph_shaping::ParagraphShaping;
use super::replaced_content::{element_image_of, emit_vector, object_fit_tile};
use super::scroll_container_paint::overflow_clips;
use super::stacking::{PaintLayer, PaintUnit, effective_paint_layer, effective_stacking_z};
use super::subtree_brackets::SubtreeBrackets;
use super::text_painter::{emit_generated_run, paint_text_node};
use super::{ComputedStyles, EditingView, GeneratedContent, LayoutBoxes, ScrollPaint};

#[doc = include_str!("scene_coordinator.doc/SceneCoordinator.md")]
pub struct SceneCoordinator {
    #[doc = include_str!("scene_coordinator.doc/text.md")]
    text: ParagraphShaping,
}

impl SceneCoordinator {
    #[doc = include_str!("scene_coordinator.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            text: ParagraphShaping::new(),
        }
    }

    #[doc = include_str!("scene_coordinator.doc/set_hyphenator.md")]
    pub fn set_hyphenator(&mut self, hyphenator: Box<dyn Hyphenator>) {
        self.text.set_hyphenator(hyphenator);
    }

    #[doc = include_str!("scene_coordinator.doc/hyphenator.md")]
    #[must_use]
    pub fn hyphenator(&self) -> Option<&dyn Hyphenator> {
        self.text.hyphenator()
    }

    #[doc = include_str!("scene_coordinator.doc/set_text_shaper.md")]
    pub fn set_text_shaper(&mut self, shaper: Box<dyn TextShaper>) {
        self.text.set_shaper(shaper);
    }

    #[doc = include_str!("scene_coordinator.doc/set_itemizer.md")]
    pub fn set_itemizer(&mut self, itemizer: Box<dyn TextItemizer>) {
        self.text.set_itemizer(itemizer);
    }

    #[doc = include_str!("scene_coordinator.doc/set_font_resolver.md")]
    pub fn set_font_resolver(&mut self, resolver: Box<dyn FontResolver>) {
        self.text.set_resolver(resolver);
    }

    #[doc = include_str!("scene_coordinator.doc/set_language.md")]
    pub fn set_language(&mut self, language: Option<Box<str>>) {
        self.text.set_language(language);
    }

    #[doc = include_str!("scene_coordinator.doc/paragraph.md")]
    #[must_use]
    pub fn paragraph(&self) -> &ParagraphShaper {
        self.text.paragraph()
    }

    #[doc = include_str!("scene_coordinator.doc/text_2.md")]
    pub(crate) fn text(&self) -> &ParagraphShaping {
        &self.text
    }

    #[doc = include_str!("scene_coordinator.doc/compose.md")]
    #[must_use]
    pub fn compose(
        &self,
        tree: &RetainedTree,
        computed: &ComputedStyles,
        boxes: &LayoutBoxes,
        editing: Option<EditingView<'_>>,
    ) -> DisplayList {
        // The image-free path: no `background-image: url(...)` resolves to a
        // painted image, so this is byte-identical to before image backgrounds.
        self.compose_with_images(tree, computed, boxes, editing, &HashMap::new())
    }

    #[doc = include_str!("scene_coordinator.doc/compose_with_images.md")]
    #[must_use]
    pub fn compose_with_images(
        &self,
        tree: &RetainedTree,
        computed: &ComputedStyles,
        boxes: &LayoutBoxes,
        editing: Option<EditingView<'_>>,
        images: &HashMap<Box<str>, SceneImage>,
    ) -> DisplayList {
        // No scroll containers: byte-identical to before per-element scroll existed.
        self.compose_with_images_scrolled(
            tree,
            computed,
            boxes,
            editing,
            images,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        )
    }

    #[doc = include_str!("scene_coordinator.doc/compose_with_images_scrolled.md")]
    #[must_use]
    pub fn compose_with_images_scrolled(
        &self,
        tree: &RetainedTree,
        computed: &ComputedStyles,
        boxes: &LayoutBoxes,
        editing: Option<EditingView<'_>>,
        images: &HashMap<Box<str>, SceneImage>,
        vectors: &HashMap<Box<str>, SceneVector>,
        scroll: &HashMap<NodeId, ScrollPaint>,
        generated: &HashMap<NodeId, GeneratedContent>,
    ) -> DisplayList {
        let Some(root) = tree.root() else {
            return DisplayListBuilder::new().build();
        };

        // Each node's effective stacking z (a positioned element's numeric
        // `z-index`, else its nearest z-index ancestor's, root = 0). Computed once
        // up front so the emission walk can key each paint unit by it.
        let zmap = effective_stacking_z(tree, computed);
        // Each node's CSS 2.1 §9.9 paint-layer rank, keyed like `zmap`. Sorted
        // between `z` and `tree_index`, it lifts a positioned `z:auto`/`0` box (and
        // its inherited subtree) above non-positioned siblings within the same z
        // bucket (catalog bug B2). Uniform (`NonPositioned`) when nothing is
        // positioned, so the sort then reduces to the pre-fix `(z, tree_index)`.
        let layermap = effective_paint_layer(tree, computed);

        // Pre-order walk (parent before children). Children are pushed reversed
        // so the leftmost child is popped first, preserving left-to-right paint
        // order. The visited set makes the walk cycle-safe and panic-free.
        //
        // Instead of emitting directly, each node's own paint (background / text —
        // not its descendants') is captured as a `PaintUnit` tagged with its
        // stacking key `(z, tree_index)`, where `tree_index` is the node's
        // pre-order position. The units are then STABLE-SORTED by `(z, tree_index)`
        // so higher-z positioned elements paint last (on top) while equal-z units
        // keep tree order. When no `z-index` is authored every z is 0 and the sort
        // reproduces the exact pre-order sequence (the regression guard).
        let mut visited: HashSet<NodeId> = HashSet::new();
        // Each stack entry carries the node's parent and its tree depth, so a
        // `border-radius` element can bracket its subtree with a clip whose
        // `PopClip` sorts after the last descendant (see `SubtreeBrackets`).
        let mut stack: Vec<(NodeId, Option<NodeId>, u32)> = vec![(root, None, 0)];
        let mut units: Vec<PaintUnit> = Vec::new();
        let mut next_index: usize = 0;
        // Every subtree wrapper — the rounded / overflow clip, the CSS transform,
        // the opacity / blend / filter group layers, the scroll translation and the
        // outline ring — is recorded here as a PAIR, and emitted below once the walk
        // knows where each subtree ends (see `SubtreeBrackets`).
        let mut brackets = SubtreeBrackets::default();

        while let Some((node, parent, depth)) = stack.pop() {
            if !visited.insert(node) {
                // Already painted (cycle or shared child): do not revisit.
                continue;
            }
            // Assign the node's pre-order document position (the stacking
            // tie-breaker) at pop time, matching the historical emission order.
            let tree_index = next_index;
            next_index += 1;
            brackets.enter(node, parent, depth, tree_index);

            // A node's per-corner `border-radius` (all four corners), read off its
            // computed style and resolved against its arranged border box. The box is
            // needed twice over, which is why this is the one place the radii are
            // derived: a `%` corner resolves against it per axis (CSS Backgrounds 3
            // §4.1 — width for the horizontal radius, height for the vertical, so
            // `50%` is a circle only on a square box), and the §4.5
            // overlapping-curves correction then measures the radii against it. Both
            // run ONCE here, so the SAME resolved, normalized radii feed every
            // consumer — background fill, subtree clip, border ring, box-shadow and
            // outline. Radii that already fit are returned unchanged
            // (byte-identical); a node without an arranged box has no basis for a `%`
            // and no box to correct against, so it keeps its absolute radii.
            let radii = boxes.get(&node).map_or_else(
                || corner_radii_of(computed, node, None),
                |layout_box| {
                    corner_radii_of(computed, node, Some(layout_box.bounds))
                        .normalized(layout_box.bounds)
                },
            );
            let has_radius = radii.any_positive();
            // A node with `overflow` other than `visible` clips its subtree to its
            // box (`hidden` / `clip` / `scroll` / `auto`; scroll/auto clip like
            // hidden — no scrollbars yet). `visible` / absent adds no clip.
            let clips_overflow = overflow_clips(computed, node);
            // The clip shape (border box) bracketing this node's subtree, recorded
            // for a second pass once every descendant's exit index is known. A node
            // that rounds its corners OR clips overflow emits exactly ONE clip: the
            // ROUNDED border box when it has a radius (which also clips overflow),
            // else a plain rect clip to the border box.
            if (has_radius || clips_overflow)
                && let Some(layout_box) = boxes.get(&node)
            {
                let clip = if has_radius {
                    Clip::rounded_rect(layout_box.bounds, radii)
                } else {
                    Clip::rect(layout_box.bounds)
                };
                brackets.clip(node, clip);
            }

            // A scroll container (present in `scroll`): record its descendant
            // scroll transform (elided at `scroll_top == 0` — the identity) and its
            // scrollbar. The `overflow` clip above already brackets the same
            // subtree, so the scroll transform nests inside it (CSS Overflow 3
            // §scrollport). Absent from the map ⇒ nothing recorded ⇒ byte-identical.
            if let (Some(paint), Some(layout_box)) = (scroll.get(&node), boxes.get(&node)) {
                if paint.scroll_top != 0.0 {
                    brackets.scroll(node, paint.scroll_top);
                }
                if paint.shows_scrollbar {
                    brackets.scrollbar(
                        node,
                        layout_box.bounds,
                        paint.scroll_top,
                        paint.content_height,
                    );
                }
            }

            // A non-identity CSS 2D `transform` brackets this node's subtree. The
            // affine is resolved about the node's transform origin (border-box
            // position + `transform-origin`, default center). No transform (or an
            // identity one) records nothing — byte-for-byte identical to before.
            if let Some(layout_box) = boxes.get(&node)
                && let Some(affine) = resolved_transform(computed, node, layout_box.bounds)
            {
                brackets.transform(node, affine);
            }

            // A CSS `opacity` below `1.0` brackets this node's subtree with an
            // opacity group layer. It needs no geometry (the layer applies to the
            // group as a whole), so — unlike the transform — it is recorded even
            // for a node with no arranged box. `opacity: 1` or an absent opacity
            // records nothing (byte-for-byte identical to before).
            if let Some(alpha) = opacity_of(computed, node) {
                brackets.opacity(node, alpha);
            }

            // A CSS `mix-blend-mode` other than `normal` brackets this node's
            // subtree with a blend group layer. Like opacity it needs no box.
            if let Some(mode) = blend_mode_of(computed, node) {
                brackets.blend(node, mode);
            }

            // A CSS `filter` (blur and/or color ops) brackets this node's subtree
            // with a filter group layer clipped to its border box. It needs the
            // arranged box (the effect region), so — unlike opacity — it is only
            // recorded for a node with a box. The color-op chain folds into one
            // 4×5 matrix; blur keeps its radius. No blur AND no color op (or no
            // box) records nothing (byte-for-byte identical).
            if let Some(layout_box) = boxes.get(&node) {
                let radius = filter_blur_of(computed, node);
                let color_matrix = filter_color_matrix_of(computed, node);
                if radius.is_some() || color_matrix.is_some() {
                    brackets.filter(node, layout_box.bounds, radius.unwrap_or(0.0), color_matrix);
                }
            }

            // A CSS `outline` (focus ring): when the node has a non-`none`
            // `outline-style` and a positive `outline-width`, record the stroked
            // ring it paints OUTSIDE its border box (inflated by
            // `outline-offset + width/2`). It needs the arranged box; no outline
            // (or no box) records nothing (byte-for-byte identical to before).
            if let Some(layout_box) = boxes.get(&node)
                && let Some(command) =
                    outline_paint(computed, node, layout_box.bounds, radii, has_radius)
            {
                brackets.outline(node, command);
            }

            // This node's own paint accumulates into a private builder so it can be
            // reordered as one unit; only leaf commands are emitted, so it stays
            // balanced.
            let mut node_paint = DisplayListBuilder::new();

            // CSS `backdrop-filter: blur(r)`: the FIRST command of the node's paint
            // unit, emitted BEFORE its own shadow / background / border / content. It
            // frosts (gaussian-blurs) the backdrop — everything painted earlier
            // (prior siblings, ancestor content) — within the node's border box,
            // clipped to its rounded corners, so the node's (usually translucent)
            // background then draws OVER the frosted backdrop → acrylic. It needs the
            // arranged box (the frost region); no `backdrop-filter` (or no box) emits
            // nothing (byte-for-byte identical). Only the GPU (Vello) backend frosts;
            // the CPU rasterizer honestly skips it (documented).
            if let Some(radius) = backdrop_blur_of(computed, node)
                && let Some(layout_box) = boxes.get(&node)
            {
                node_paint.backdrop_filter(layout_box.bounds, radii, radius);
            }

            // CSS `box-shadow`: a **drop** shadow paints BEHIND the node's
            // background (emitted first), an **inset** shadow ON TOP of it
            // (emitted after the background fill, below). CSS paints the
            // first-listed shadow on top, so each group is emitted in reverse
            // order. The shadow color falls back to the computed `color`
            // (`currentColor`) when the author omitted it. No `box-shadow` emits
            // nothing — byte-for-byte identical to before.
            let shadows = box_shadows_of(computed, node);
            if let (Some(shadows), Some(layout_box)) = (shadows, boxes.get(&node)) {
                let current = current_color_of(computed, node);
                for shadow in shadows.iter().rev().filter(|s| !s.is_inset()) {
                    emit_box_shadow(&mut node_paint, layout_box.bounds, radii, shadow, current);
                }
            }

            // CSS `filter: drop-shadow(...)`: each drop-shadow paints BEHIND the
            // node like a `box-shadow` DROP shadow (spread 0, never inset), reusing
            // the same `DrawShadow` path — the GPU backend blurs it with a real
            // gaussian, the CPU backend approximates a flat offset rect. In real
            // CSS `filter` applies to the already-rendered element (including its
            // `box-shadow`); we approximate the shadow as the element's border box,
            // and emit the filter drop-shadows AFTER the `box-shadow` drop shadows
            // (so they sit just in front of them) but still behind the element's own
            // background / border / text. Only the drop-shadow filter functions
            // reach here; `blur()` / color-matrix / `backdrop-filter` are dropped at
            // CSS lower time (out of scope on vello 0.9). No filter emits nothing —
            // byte-for-byte identical to before.
            let filter_ops = filter_ops_of(computed, node);
            if let (Some(filter_ops), Some(layout_box)) = (filter_ops, boxes.get(&node)) {
                let current = current_color_of(computed, node);
                for shadow in filter_ops.iter().rev().filter_map(FilterOp::as_drop_shadow) {
                    emit_box_shadow(&mut node_paint, layout_box.bounds, radii, shadow, current);
                }
            }

            // The background fill brush: an authored `background-image` gradient
            // (linear / radial / conic) wins over `background-color` (the gradient
            // paints over the color), otherwise a non-transparent
            // `background-color` fills solid. With neither authored the block emits
            // nothing — byte-for-byte identical to before backgrounds existed.
            if let Some(layout_box) = boxes.get(&node) {
                let brush = if let Some(brush) =
                    background_gradient_brush(computed, node, layout_box.bounds)
                {
                    Some(brush)
                } else {
                    background_color_of(computed, node)
                        // Skip fully transparent backgrounds: they paint nothing.
                        .filter(|color| color.a > 0.0)
                        .map(|color| Brush::solid(scene_color_from(color)))
                };
                if let Some(brush) = brush {
                    if has_radius {
                        // A rounded element fills a rounded-rect outline of its
                        // border box instead of a plain rectangle.
                        node_paint.fill(Path::rounded_rect(layout_box.bounds, radii), brush);
                    } else {
                        node_paint.fill_rect(layout_box.bounds, brush);
                    }
                }

                // An authored `background-image: url(...)` whose image has been
                // resolved paints OVER the `background-color` fill and UNDER the
                // border/content — matching CSS background paint order. The image's
                // tile is placed (`background-position`), sized (`background-size`),
                // and tiled (`background-repeat`) within the node's border box (the
                // paint region), resolved by `background_image_geometry`. A URL not
                // present in `images` (unloaded or failed) emits nothing, so the
                // color background shows through — the regression-safe path
                // (byte-identical when `images` is empty). This is the SINGLE-layer
                // path; the multi-layer list is handled just below.
                if let Some(image) =
                    background_image_url_of(computed, node).and_then(|url| images.get(url))
                {
                    let (tile, repeat_x, repeat_y) =
                        background_image_geometry(computed, node, image, layout_box.bounds);
                    node_paint.draw_image(
                        image.clone(),
                        tile,
                        layout_box.bounds,
                        repeat_x,
                        repeat_y,
                    );
                }

                // Multiple `background-image` layers (a comma list). CSS lists them
                // top→bottom (first = top) and paints back-to-front, so we iterate
                // in REVERSE: the last-listed (bottom) layer paints first, the
                // first-listed (top) last. Every layer paints OVER the
                // `background-color` fill emitted above and UNDER the border/content.
                // The single-layer case never reaches here (it stays a flat value
                // above), so one-layer backgrounds are byte-identical. Per-layer
                // `background-size`/`-position`/`-repeat` currently reuse the node's
                // single resolved value for every layer (those longhands are not yet
                // comma lists).
                if let Some(layers) = background_layers_of(computed, node) {
                    for layer in layers.iter().rev() {
                        paint_background_layer(
                            &mut node_paint,
                            layer,
                            computed,
                            node,
                            layout_box.bounds,
                            radii,
                            has_radius,
                            images,
                        );
                    }
                }
            }

            // Inset shadows paint on top of the background (clipped to the box,
            // approximated by the renderer), under the border and text.
            if let (Some(shadows), Some(layout_box)) = (shadows, boxes.get(&node)) {
                let current = current_color_of(computed, node);
                for shadow in shadows.iter().rev().filter(|s| s.is_inset()) {
                    emit_box_shadow(&mut node_paint, layout_box.bounds, radii, shadow, current);
                }
            }

            // Border edges paint over the background, under the text. There are two
            // shapes:
            //
            // - A **uniform** border (all four widths equal & positive, all four
            //   colors equal, every side `solid`) on a node with a positive corner
            //   radius paints as ONE even-odd rounded-rect RING: the border-box
            //   rounded rect with the padding-box rounded rect (radii reduced by the
            //   border width) punched out. This rounds the border's INNER corners as
            //   well as its outer ones, matching CSS — the mitered-quad path below
            //   keeps its corners square-cornered (but 45°-mitered). The ring is
            //   self-contained, so it does not rely on the surrounding rounded
            //   `PushClip` for its shape.
            // - Otherwise (no radius, or a non-uniform border) each visible side
            //   paints one **mitered trapezoid quad**: its outer edge spans the full
            //   border-box side and its inner edge is inset by the two adjacent
            //   border widths, so the corners split at 45° between their two sides
            //   (the correct CSS border-corner model, and what makes the CSS
            //   border-triangle trick render — a collapsed side becomes a triangle).
            //   A rounded non-uniform border is clipped to the rounded border box
            //   (the surrounding `PushClip`) and its inner corners stay approximate
            //   (documented). A transparent or zero-width side, or an unauthored
            //   border, emits nothing.
            if let Some(layout_box) = boxes.get(&node) {
                if let Some((width, color)) = uniform_border(computed, node).filter(|_| has_radius)
                {
                    let ring = uniform_border_ring(layout_box.bounds, radii, width);
                    node_paint.fill(ring, Brush::solid(scene_color_from(color)));
                } else if let Some((width, color)) = uniform_dashed_border(computed, node) {
                    // A uniform DASHED border is a single dashed stroke run along
                    // the border box inset by width/2, so the centered stroke band
                    // exactly fills the CSS border area (rounded when a radius is
                    // present). Solid borders never reach here.
                    let path = dashed_border_path(layout_box.bounds, radii, width, has_radius);
                    let stroke = Stroke::new(Brush::solid(scene_color_from(color)), width)
                        .with_dash(dashed_border_pattern(width), 0.0);
                    node_paint.stroke(path, stroke);
                } else {
                    for (path, color) in border_fills(computed, node, layout_box.bounds) {
                        node_paint.fill(path, Brush::solid(scene_color_from(color)));
                    }
                }
            }

            // An element's own image content (an `<img>` / `Image` primitive's
            // `src`) whose image has been resolved paints as the element's CONTENT:
            // OVER its background / border (emitted after them), UNDER its children
            // and text (they are separate paint units at a later tree index). The
            // image is sized/placed by `object-fit` (default `fill` = stretch to the
            // box) about the box center, and its `region` is the box so `cover`
            // overflow is clipped. The paint uses the element's BORDER box (the
            // content-box insets are not resolved in this pure coordinator — a
            // documented approximation). A `src` absent from `images` (unloaded or
            // failed) or no `src` at all emits nothing (byte-identical when `images`
            // is empty). A `background-image` and an element `src` both paint: the
            // background behind, the element image above.
            if let Some(layout_box) = boxes.get(&node)
                && let Some(image) = element_image_of(tree, node).and_then(|src| images.get(src))
            {
                let tile = object_fit_tile(computed, node, image, layout_box.bounds);
                node_paint.draw_image(image.clone(), tile, layout_box.bounds, false, false);
            }

            // An element's own VECTOR content (an `<img src="...svg">`) paints at
            // exactly the same point in the node's paint unit as a bitmap, placed
            // by the same `object-fit` rule — it is a replaced element either way.
            //
            // It stays vector (spec §5.17): rather than rasterizing, the document's
            // own coordinate space is mapped onto the placed tile by a transform and
            // its outlines are emitted as `DrawPath`, so it rasterizes at the
            // frame's resolution like every other path. `currentColor` is resolved
            // HERE, against the node's computed `color` — the parsed document is
            // colour-independent, so a theme change re-resolves without reparsing.
            if let Some(layout_box) = boxes.get(&node)
                && let Some(vector) = element_image_of(tree, node).and_then(|src| vectors.get(src))
                && !vector.is_empty()
            {
                emit_vector(&mut node_paint, computed, node, vector, layout_box.bounds);
            }

            // §4.16 generated content (`::before` / `::after`) paints inside THIS
            // node's own paint unit, so a `::before` run sorts BEFORE the element's
            // real text child (a separate, later paint unit). It never becomes a
            // tree node, an accessibility node, or a hit-test target — it exists
            // only here in the display list. `::after` is a documented
            // approximation: it also paints in the node's own unit rather than
            // after the subtree.
            if let (Some(content), Some(layout_box)) = (generated.get(&node), boxes.get(&node)) {
                // The placeholder synthetic part (§4.16) paints only while the field
                // is *effectively* empty (`:placeholder-shown`): the resolver already
                // gated it on the retained empty value, but a live editing session on
                // THIS field with typed text or an active IME preedit makes the
                // effective text non-empty, so the hint is suppressed for that frame.
                let placeholder = content.placeholder.as_ref().filter(|_| {
                    editing.is_none_or(|view| {
                        view.node != node
                            || (view.text.is_empty()
                                && view.preedit.is_none_or(|preedit| preedit.text.is_empty()))
                    })
                });
                for run in [content.before.as_ref(), content.after.as_ref(), placeholder]
                    .into_iter()
                    .flatten()
                {
                    emit_generated_run(
                        &self.text,
                        &mut node_paint,
                        tree,
                        computed,
                        boxes,
                        node,
                        layout_box.origin(),
                        layout_box.size().width,
                        run,
                    );
                }
            }

            if let Some(retained) = tree.get(node) {
                // A text node paints its own runs — and, while it is the focused
                // editable's text, its selection highlight, IME preedit underline and
                // caret — into this node's paint unit (see `text_painter`).
                if let Some(retained_content) = retained.kind().text() {
                    paint_text_node(
                        &self.text,
                        &mut node_paint,
                        tree,
                        computed,
                        boxes,
                        node,
                        retained_content,
                        editing,
                    );
                }

                for &child in retained.child_nodes().iter().rev() {
                    if !visited.contains(&child) {
                        stack.push((child, Some(node), depth.saturating_add(1)));
                    }
                }
            }

            // A node with no own paint (no background, no text) contributes no
            // unit — exactly as the historical walk emitted nothing for it.
            let commands = node_paint.build().commands().to_vec();
            if !commands.is_empty() {
                let z = zmap.get(&node).copied().unwrap_or(0);
                let layer = layermap
                    .get(&node)
                    .copied()
                    .unwrap_or(PaintLayer::NonPositioned);
                units.push(PaintUnit {
                    z,
                    layer,
                    tree_index,
                    order: PaintUnit::PAINT_ORDER,
                    commands,
                });
            }
        }

        // Every recorded bracket becomes its `Push*` at the node's enter index and
        // its `Pop*` at the subtree's exit index, in the one place that decides how
        // the kinds nest.
        brackets.emit(&zmap, &layermap, &mut units);

        // Stable sort by `(z, layer, tree_index, order)`, all ascending (CSS 2.1
        // §9.9 / Appendix E painting order): higher z paints last (on top); within
        // one z bucket the §9.9 `layer` rank lifts a positioned `z:auto`/`0` box
        // (`PositionedZeroOrAuto`) above every non-positioned unit (`NonPositioned`)
        // regardless of tree order — the fix for bug B2; equal-(z, layer) units keep
        // pre-order (document) order; and `order` phases a subtree's `PushClip`
        // before the node's paint and its `PopClip` after the whole subtree. When
        // nothing is positioned every `layer` is `NonPositioned` and, with no radius,
        // every `order` is `PAINT_ORDER`, so the key reduces to `(z, tree_index)`,
        // reproducing the pre-fix pre-order sequence byte-for-byte.
        units.sort_by_key(|unit| (unit.z, unit.layer, unit.tree_index, unit.order));

        let mut ordered: Vec<PaintCommand> = Vec::new();
        for unit in units {
            ordered.extend(unit.commands);
        }
        DisplayList::from_commands(ordered)
    }
}

impl Default for SceneCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for SceneCoordinator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SceneCoordinator")
            .field("text", &self.text)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::super::caret_geometry::{caret_offset_at_point, line_caret_x};
    use super::super::paragraph_shaping::DEFAULT_FONT_SIZE;
    use super::super::text_painter::{CARET_WIDTH, PREEDIT_UNDERLINE_THICKNESS, SELECTION_COLOR};
    use super::*;
    use obscur_geometry::{CornerRadii, CornerRadius, Point, Rect, Size};
    use obscur_identity::ComponentId;
    use obscur_layout::model::LayoutBox;
    use obscur_scene::color::Color;
    use obscur_scene::command::PaintCommand;
    use obscur_scene::layer::{LayerEffect, SceneLayer};
    use obscur_scene::paint::ExtendMode;
    use obscur_scene::paint::Paint;
    use obscur_scene::text::{GlyphRun, PositionedGlyph};
    use obscur_style::computed::ComputedStyle;
    use obscur_style::computed::ComputedValue;
    use obscur_style::property::{
        BackgroundLayer, BackgroundPosition, BackgroundRepeat, BackgroundSize, BoxShadow, FilterOp,
        Length, LinearGradient, ObjectFit, PropertyId, PropertyValue, StyleColor, Transform,
        layout_keyword,
    };
    use obscur_text::font::{FontFace, MockFontResolver};
    use obscur_text::selection::{CaretAffinity, SelectionGeometry, TextSelection};
    use obscur_text::shaping::{MockTextShaper, TextDirection};
    use obscur_tree::node::{LogicalParent, NodeKind};

    /// Allocates an element node under `parent`, wiring the child link and the
    /// logical parent, and returns its id.
    fn add_child(tree: &mut RetainedTree, parent: NodeId, raw_component: u64) -> NodeId {
        let id = tree.alloc(NodeKind::Element {
            component: ComponentId::from_raw(raw_component),
        });
        if let Some(node) = tree.get_mut(parent) {
            node.push_child(id);
        }
        if let Some(node) = tree.get_mut(id) {
            node.set_logical_parent(LogicalParent::of(parent));
        }
        id
    }

    /// Allocates a text node carrying `content` under `parent`, wiring the child
    /// link and the logical parent, and returns its id.
    fn add_text_child(tree: &mut RetainedTree, parent: NodeId, content: &str) -> NodeId {
        let id = tree.alloc(NodeKind::Text {
            content: content.into(),
        });
        if let Some(node) = tree.get_mut(parent) {
            node.push_child(id);
        }
        if let Some(node) = tree.get_mut(id) {
            node.set_logical_parent(LogicalParent::of(parent));
        }
        id
    }

    /// Builds a `ComputedStyle` carrying a single text `color`.
    fn with_color(color: StyleColor) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::Color, ComputedValue::color(color));
        style
    }

    /// Collects the `DrawText` runs (origin, glyph run, brush) in a display list.
    fn text_runs(list: &DisplayList) -> Vec<(Point, GlyphRun, Brush)> {
        list.commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawText { origin, run, brush } => {
                    Some((*origin, run.clone(), brush.clone()))
                }
                _ => None,
            })
            .collect()
    }

    /// Builds a `ComputedStyle` carrying a single `background-color`.
    fn with_background(color: StyleColor) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::BackgroundColor, ComputedValue::color(color));
        style
    }

    /// A layout box at the origin with the given size.
    fn box_at(x: f32, y: f32, w: f32, h: f32) -> LayoutBox {
        LayoutBox::new(Rect::new(Point::new(x, y), Size::new(w, h)))
    }

    /// Counts the `DrawPath` (fill) commands in a display list.
    fn fill_count(list: &DisplayList) -> usize {
        list.commands()
            .iter()
            .filter(|c| matches!(c, PaintCommand::DrawPath { .. }))
            .count()
    }

    /// Collects the `DrawPath` (fill) brushes of a display list, in paint order.
    fn fill_brushes(list: &DisplayList) -> Vec<Brush> {
        list.commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawPath { paint, .. } => Some(paint.brush().clone()),
                _ => None,
            })
            .collect()
    }

    /// A `ComputedStyle` carrying a single `background-image` linear gradient.
    fn with_background_gradient(gradient: LinearGradient) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::BackgroundImage,
            ComputedValue::from_concrete(PropertyValue::LinearGradient(gradient))
                .expect("a gradient is a concrete computed value"),
        );
        style
    }

    #[test]
    fn linear_gradient_background_emits_gradient_brush() {
        use obscur_style::property::GradientStop as StyleStop;

        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        // `linear-gradient(90deg, red, blue)` = to right.
        let gradient = LinearGradient::new(
            90.0,
            vec![
                StyleStop::new(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), 0.0),
                StyleStop::new(StyleColor::rgba(0.0, 0.0, 1.0, 1.0), 1.0),
            ],
        );
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, with_background_gradient(gradient));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 100.0, 50.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let brushes = fill_brushes(&list);
        assert_eq!(brushes.len(), 1, "one gradient fill");
        let Brush::Linear(g) = &brushes[0] else {
            panic!("expected a linear gradient brush, got {:?}", brushes[0]);
        };
        // 90deg (to right): the line runs left→right through the box center.
        let near = |a: Point, b: Point| (a.x - b.x).abs() < 1e-3 && (a.y - b.y).abs() < 1e-3;
        assert!(
            near(g.start(), Point::new(0.0, 25.0)),
            "start {:?}",
            g.start()
        );
        assert!(near(g.end(), Point::new(100.0, 25.0)), "end {:?}", g.end());
        // The authored stops, in order, with their colors and offsets.
        assert_eq!(g.stops().len(), 2);
        assert_eq!(g.stops()[0].offset(), 0.0);
        assert_eq!(g.stops()[0].color(), Color::rgba(1.0, 0.0, 0.0, 1.0));
        assert_eq!(g.stops()[1].offset(), 1.0);
        assert_eq!(g.stops()[1].color(), Color::rgba(0.0, 0.0, 1.0, 1.0));
        // A plain linear gradient keeps the default Pad extend.
        assert_eq!(g.extend(), ExtendMode::Pad);
        assert!(list.is_balanced());
    }

    #[test]
    fn repeating_linear_gradient_background_tiles_one_period() {
        use obscur_style::property::GradientStop as StyleStop;

        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        // `repeating-linear-gradient(90deg, #000 0 4px, #fff 4px 8px)`: an 8px
        // period of two hard bands, to the right, over a 100×50 box.
        let gradient = LinearGradient::new_repeating(
            90.0,
            vec![
                StyleStop::new(StyleColor::BLACK, 0.0),
                StyleStop::new(StyleColor::BLACK, 0.5),
                StyleStop::new(StyleColor::rgba(1.0, 1.0, 1.0, 1.0), 0.5),
                StyleStop::new(StyleColor::rgba(1.0, 1.0, 1.0, 1.0), 1.0),
            ],
            Length::px(8.0),
        );
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, with_background_gradient(gradient));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 100.0, 50.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let brushes = fill_brushes(&list);
        assert_eq!(brushes.len(), 1, "one gradient fill");
        let Brush::Linear(g) = &brushes[0] else {
            panic!("expected a linear gradient brush, got {:?}", brushes[0]);
        };
        // The axis spans exactly one 8px period from the box's 0% anchor, and the
        // pattern tiles via Repeat.
        assert_eq!(g.extend(), ExtendMode::Repeat);
        let near = |a: Point, b: Point| (a.x - b.x).abs() < 1e-3 && (a.y - b.y).abs() < 1e-3;
        assert!(
            near(g.start(), Point::new(0.0, 25.0)),
            "start {:?}",
            g.start()
        );
        assert!(near(g.end(), Point::new(8.0, 25.0)), "end {:?}", g.end());
        assert_eq!(g.stops().len(), 4);
        assert_eq!(g.stops()[1].offset(), 0.5);
        assert!(list.is_balanced());
    }

    /// A solid 2×2 opaque RGBA image, for the `background-image: url(...)` tests.
    fn canned_image() -> SceneImage {
        use obscur_scene::color::AlphaMode;
        let pixels: std::sync::Arc<[u8]> = std::sync::Arc::from(vec![0u8; 2 * 2 * 4]);
        SceneImage::new(2, 2, pixels, AlphaMode::Straight).expect("2x2 rgba is valid")
    }

    /// Collects the `DrawImage` `(tile, region, repeat_x, repeat_y)` tuples of a
    /// display list, in paint order.
    fn image_draws(list: &DisplayList) -> Vec<(Rect, Rect, bool, bool)> {
        list.commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawImage {
                    tile,
                    region,
                    repeat_x,
                    repeat_y,
                    ..
                } => Some((*tile, *region, *repeat_x, *repeat_y)),
                _ => None,
            })
            .collect()
    }

    /// A `ComputedStyle` carrying a `background-image: url(...)` plus explicit
    /// `background-size` / `-position` / `-repeat` computed values.
    fn with_background_layer(
        url: &str,
        size: BackgroundSize,
        position: BackgroundPosition,
        repeat: BackgroundRepeat,
    ) -> ComputedStyle {
        let mut style = with_background_image(PropertyValue::BackgroundImageUrl(url.into()));
        style.set(
            PropertyId::BackgroundSize,
            ComputedValue::from_concrete(PropertyValue::BackgroundSize(size)).expect("concrete"),
        );
        style.set(
            PropertyId::BackgroundPosition,
            ComputedValue::from_concrete(PropertyValue::BackgroundPosition(position))
                .expect("concrete"),
        );
        style.set(
            PropertyId::BackgroundRepeat,
            ComputedValue::from_concrete(PropertyValue::BackgroundRepeat(repeat))
                .expect("concrete"),
        );
        style
    }

    #[test]
    fn url_background_image_emits_draw_image_over_the_box() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_image(PropertyValue::BackgroundImageUrl("x.png".into())),
        );

        let mut boxes = LayoutBoxes::new();
        let node_box = box_at(5.0, 7.0, 100.0, 50.0);
        boxes.insert(a, node_box);

        // Without the URL in the map: no DrawImage (regression-safe).
        let empty = HashMap::new();
        let bare =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &empty);
        assert!(
            image_draws(&bare).is_empty(),
            "no image resolved → no DrawImage"
        );
        assert!(bare.is_balanced());

        // With the URL resolved: one DrawImage. With NO authored size/position/
        // repeat, the CSS initials apply (auto size = the intrinsic 2×2 tile at the
        // box's top-left, repeat both axes) — the CSS-correct default, NOT a
        // full-box stretch. The region is the node's border box.
        let mut images = HashMap::new();
        images.insert(Box::<str>::from("x.png"), canned_image());
        let list =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &images);
        let draws = image_draws(&list);
        assert_eq!(draws.len(), 1, "one resolved image paints one DrawImage");
        let (tile, region, rx, ry) = draws[0];
        assert_eq!(region, node_box.bounds, "region is the node's border box");
        // Intrinsic 2×2 tile at the box origin (top-left).
        assert_eq!(tile.origin, node_box.bounds.origin);
        assert!((tile.size.width - 2.0).abs() < 1e-3);
        assert!((tile.size.height - 2.0).abs() < 1e-3);
        assert!(
            rx && ry,
            "CSS initial background-repeat is repeat on both axes"
        );
        assert!(list.is_balanced());
    }

    #[test]
    fn background_no_repeat_contain_fits_one_tile() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_layer(
                "x.png",
                BackgroundSize::Contain,
                BackgroundPosition::default(),
                BackgroundRepeat::NoRepeat,
            ),
        );
        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 10.0, 10.0));

        let mut images = HashMap::new();
        images.insert(Box::<str>::from("x.png"), canned_image());
        let list =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &images);
        let draws = image_draws(&list);
        assert_eq!(draws.len(), 1);
        let (tile, _region, rx, ry) = draws[0];
        // `contain` of a 2×2 image into a 10×10 box scales to 10×10 (aspect kept).
        assert!(
            (tile.size.width - 10.0).abs() < 1e-3,
            "contain width, got {tile:?}"
        );
        assert!((tile.size.height - 10.0).abs() < 1e-3, "contain height");
        assert_eq!(tile.origin, Point::new(0.0, 0.0), "top-left position");
        assert!(!rx && !ry, "no-repeat clears both axis flags");
    }

    #[test]
    fn background_cover_scales_to_cover_the_box() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_layer(
                "x.png",
                BackgroundSize::Cover,
                BackgroundPosition::default(),
                BackgroundRepeat::NoRepeat,
            ),
        );
        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 10.0, 10.0));

        let mut images = HashMap::new();
        images.insert(Box::<str>::from("x.png"), canned_image());
        let list =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &images);
        let (tile, _region, _rx, _ry) = image_draws(&list)[0];
        // `cover` scales to at least the box on both axes (≥ 10×10). The region
        // clips any overflow at paint time.
        assert!(
            tile.size.width >= 10.0 - 1e-3,
            "cover width ≥ box, got {tile:?}"
        );
        assert!(tile.size.height >= 10.0 - 1e-3, "cover height ≥ box");
    }

    /// Sets an element node's own image `src` on its retained attributes, the way
    /// the reconciler would from an `Image` primitive's `SRC` prop.
    fn set_image_src(tree: &mut RetainedTree, node: NodeId, src: &str) {
        use obscur_view::view::{SRC, StyleAttributes, ViewProperty, ViewValue};
        let attrs =
            StyleAttributes::extract(&[ViewProperty::new(SRC, ViewValue::Text(src.into()))]);
        if let Some(n) = tree.get_mut(node) {
            n.set_attributes(attrs);
        }
    }

    /// A `ComputedStyle` carrying a single `object-fit`.
    fn with_object_fit(fit: ObjectFit) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::ObjectFit,
            ComputedValue::from_concrete(PropertyValue::ObjectFit(fit)).expect("concrete"),
        );
        style
    }

    #[test]
    fn element_image_src_emits_draw_image_as_content() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let img = add_child(&mut tree, root, 10);
        set_image_src(&mut tree, img, "logo");

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        // Default object-fit (fill): no ObjectFit set → stretch to the box.
        computed.insert(img, ComputedStyle::new());

        let mut boxes = LayoutBoxes::new();
        let node_box = box_at(0.0, 0.0, 10.0, 10.0);
        boxes.insert(img, node_box);

        // Absent from the map: no DrawImage, no panic (honest skip).
        let empty = HashMap::new();
        let bare =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &empty);
        assert!(
            image_draws(&bare).is_empty(),
            "no resolved image → no DrawImage"
        );
        assert!(bare.is_balanced());

        // Resolved: one DrawImage over the box; default `fill` stretches to the box.
        let mut images = HashMap::new();
        images.insert(Box::<str>::from("logo"), canned_image());
        let list =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &images);
        let draws = image_draws(&list);
        assert_eq!(draws.len(), 1, "the element image paints one DrawImage");
        let (tile, region, rx, ry) = draws[0];
        assert_eq!(region, node_box.bounds, "region is the element box");
        assert_eq!(
            tile, node_box.bounds,
            "object-fit fill stretches to the box"
        );
        assert!(!rx && !ry, "an element image never tiles");
        assert!(list.is_balanced());
    }

    #[test]
    fn object_fit_none_centers_the_intrinsic_tile() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let img = add_child(&mut tree, root, 10);
        set_image_src(&mut tree, img, "logo");

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(img, with_object_fit(ObjectFit::None));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(img, box_at(0.0, 0.0, 10.0, 10.0));

        let mut images = HashMap::new();
        images.insert(Box::<str>::from("logo"), canned_image());
        let list =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &images);
        let (tile, region, _rx, _ry) = image_draws(&list)[0];
        // `none`: the intrinsic 2×2 tile, centered in the 10×10 box at (4, 4).
        assert!((tile.size.width - 2.0).abs() < 1e-3 && (tile.size.height - 2.0).abs() < 1e-3);
        assert!((tile.origin.x - 4.0).abs() < 1e-3 && (tile.origin.y - 4.0).abs() < 1e-3);
        assert_eq!(
            region.size,
            Size::new(10.0, 10.0),
            "region clips overflow to the box"
        );
    }

    #[test]
    fn object_fit_contain_and_cover_fit_the_box() {
        // A non-square box makes `contain` (letterbox) and `cover` (clip) differ.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let img = add_child(&mut tree, root, 10);
        set_image_src(&mut tree, img, "logo");

        let node_box = box_at(0.0, 0.0, 20.0, 10.0);

        // contain: min-scale fit of a 2×2 image into 20×10 → scale 5 → 10×10,
        // centered horizontally at x=(20−10)/2=5, y=0.
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(img, with_object_fit(ObjectFit::Contain));
        let mut boxes = LayoutBoxes::new();
        boxes.insert(img, node_box);
        let mut images = HashMap::new();
        images.insert(Box::<str>::from("logo"), canned_image());
        let list =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &images);
        let (tile, _region, _rx, _ry) = image_draws(&list)[0];
        assert!((tile.size.width - 10.0).abs() < 1e-3 && (tile.size.height - 10.0).abs() < 1e-3);
        assert!((tile.origin.x - 5.0).abs() < 1e-3 && (tile.origin.y - 0.0).abs() < 1e-3);

        // cover: max-scale of 2×2 into 20×10 → scale 10 → 20×20, centered vertically
        // at y=(10−20)/2=−5 so it fully covers; the region clips the overflow.
        computed.insert(img, with_object_fit(ObjectFit::Cover));
        let list =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &images);
        let (tile, region, _rx, _ry) = image_draws(&list)[0];
        assert!(tile.size.width >= 20.0 - 1e-3 && tile.size.height >= 10.0 - 1e-3);
        assert_eq!(
            region.size,
            Size::new(20.0, 10.0),
            "cover overflow clips to the box"
        );
    }

    #[test]
    fn background_position_right_bottom_places_tile_at_far_corner() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_layer(
                "x.png",
                BackgroundSize::Auto,
                // `right bottom` == 100% / 100%.
                BackgroundPosition::new(Length::Percent(100.0), Length::Percent(100.0)),
                BackgroundRepeat::NoRepeat,
            ),
        );
        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 10.0, 10.0));

        let mut images = HashMap::new();
        images.insert(Box::<str>::from("x.png"), canned_image());
        let list =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &images);
        let (tile, _region, _rx, _ry) = image_draws(&list)[0];
        // Intrinsic 2×2 tile positioned bottom-right: origin = box − tile = (8, 8).
        assert!(
            (tile.origin.x - 8.0).abs() < 1e-3,
            "x at box right − tile, got {tile:?}"
        );
        assert!((tile.origin.y - 8.0).abs() < 1e-3, "y at box bottom − tile");
        assert!((tile.size.width - 2.0).abs() < 1e-3);
        assert!((tile.size.height - 2.0).abs() < 1e-3);
    }

    #[test]
    fn background_default_repeat_sets_both_axis_flags() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_layer(
                "x.png",
                BackgroundSize::Auto,
                BackgroundPosition::default(),
                BackgroundRepeat::Repeat,
            ),
        );
        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 10.0, 10.0));

        let mut images = HashMap::new();
        images.insert(Box::<str>::from("x.png"), canned_image());
        let list =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &images);
        let (_tile, _region, rx, ry) = image_draws(&list)[0];
        assert!(rx && ry, "repeat sets both axis flags");
    }

    #[test]
    fn url_background_image_paints_over_background_color() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        // A node with BOTH a background-color and a background-image url.
        let mut style = with_background_image(PropertyValue::BackgroundImageUrl("x.png".into()));
        style.set(
            PropertyId::BackgroundColor,
            ComputedValue::color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)),
        );
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, style);

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 20.0, 20.0));

        let mut images = HashMap::new();
        images.insert(Box::<str>::from("x.png"), canned_image());
        let list =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &images);

        // Paint order: the background-color fill (DrawPath) comes BEFORE the image
        // (DrawImage) so the image sits on top of the color.
        let kinds: Vec<&str> = list
            .commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawPath { .. } => Some("fill"),
                PaintCommand::DrawImage { .. } => Some("image"),
                _ => None,
            })
            .collect();
        assert_eq!(
            kinds,
            vec!["fill", "image"],
            "color fills, image paints over it"
        );
    }

    #[test]
    fn multiple_background_layers_paint_back_to_front_over_color() {
        use obscur_style::property::GradientStop as StyleStop;

        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        // Two layers, in CSS author order: a linear gradient on TOP, a url image on
        // the BOTTOM — over a background-color.
        let gradient = LinearGradient::new(
            90.0,
            vec![
                StyleStop::new(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), 0.0),
                StyleStop::new(StyleColor::rgba(0.0, 0.0, 1.0, 1.0), 1.0),
            ],
        );
        let layers: Box<[BackgroundLayer]> = Box::new([
            BackgroundLayer::Linear(gradient),
            BackgroundLayer::Url("x.png".into()),
        ]);
        let mut style = with_background_image(PropertyValue::BackgroundLayers(layers));
        style.set(
            PropertyId::BackgroundColor,
            ComputedValue::color(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)),
        );
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, style);

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 100.0, 50.0));

        let mut images = HashMap::new();
        images.insert(Box::<str>::from("x.png"), canned_image());
        let list =
            SceneCoordinator::new().compose_with_images(&tree, &computed, &boxes, None, &images);
        assert!(list.is_balanced());

        // Paint order, bottom-to-top: the background-COLOR fill, then the BOTTOM
        // layer (the url image, painted first), then the TOP layer (the gradient
        // fill, painted last so it sits on top). BOTH a gradient fill and a
        // DrawImage are emitted.
        let kinds: Vec<&str> = list
            .commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawPath { .. } => Some("fill"),
                PaintCommand::DrawImage { .. } => Some("image"),
                _ => None,
            })
            .collect();
        assert_eq!(
            kinds,
            vec!["fill", "image", "fill"],
            "color, then bottom url layer, then top gradient layer"
        );
        // The two fills are the solid background-color (bottom) and the gradient
        // (top); the image draw is the middle (bottom) layer.
        let brushes = fill_brushes(&list);
        assert!(
            matches!(brushes[0], Brush::Solid(_)),
            "bg-color fills first"
        );
        assert!(
            matches!(brushes[1], Brush::Linear(_)),
            "the top gradient layer fills last"
        );
        assert_eq!(
            image_draws(&list).len(),
            1,
            "the bottom url layer draws one image"
        );
    }

    /// A `ComputedStyle` carrying a single `background-image` computed value.
    fn with_background_image(value: PropertyValue) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::BackgroundImage,
            ComputedValue::from_concrete(value).expect("a gradient is a concrete computed value"),
        );
        style
    }

    #[test]
    fn radial_gradient_background_emits_radial_brush() {
        use obscur_style::property::{
            GradientStop as StyleStop, RadialGradient, RadialShape, RadialSize,
        };

        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        // `radial-gradient(circle farthest-corner at center, red, blue)`.
        let gradient = RadialGradient::new(
            RadialShape::Circle,
            RadialSize::FarthestCorner,
            Length::Percent(50.0),
            Length::Percent(50.0),
            vec![
                StyleStop::new(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), 0.0),
                StyleStop::new(StyleColor::rgba(0.0, 0.0, 1.0, 1.0), 1.0),
            ],
        );
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_image(PropertyValue::RadialGradient(gradient)),
        );

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 100.0, 50.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let brushes = fill_brushes(&list);
        assert_eq!(brushes.len(), 1, "one radial gradient fill");
        let Brush::Radial(g) = &brushes[0] else {
            panic!("expected a radial gradient brush, got {:?}", brushes[0]);
        };
        // Center at the box center; farthest-corner radius = distance to a corner.
        let near = |a: f32, b: f32| (a - b).abs() < 1e-3;
        assert!(near(g.center().x, 50.0), "center x {:?}", g.center());
        assert!(near(g.center().y, 25.0), "center y {:?}", g.center());
        assert!(
            near(g.radius(), 50.0_f32.hypot(25.0)),
            "radius {}",
            g.radius()
        );
        assert_eq!(g.stops().len(), 2);
        assert_eq!(g.stops()[0].color(), Color::rgba(1.0, 0.0, 0.0, 1.0));
        assert_eq!(g.stops()[1].offset(), 1.0);
        assert!(list.is_balanced());
    }

    #[test]
    fn conic_gradient_background_emits_conic_brush() {
        use obscur_style::property::{ConicGradient, GradientStop as StyleStop};

        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        // `conic-gradient(from 90deg at center, red, blue)`.
        let gradient = ConicGradient::new(
            90.0,
            Length::Percent(50.0),
            Length::Percent(50.0),
            vec![
                StyleStop::new(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), 0.0),
                StyleStop::new(StyleColor::rgba(0.0, 0.0, 1.0, 1.0), 1.0),
            ],
        );
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_image(PropertyValue::ConicGradient(gradient)),
        );

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 100.0, 50.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let brushes = fill_brushes(&list);
        assert_eq!(brushes.len(), 1, "one conic gradient fill");
        let Brush::Conic(g) = &brushes[0] else {
            panic!("expected a conic gradient brush, got {:?}", brushes[0]);
        };
        let near = |a: f32, b: f32| (a - b).abs() < 1e-3;
        assert!(near(g.center().x, 50.0), "center x {:?}", g.center());
        assert!(near(g.center().y, 25.0), "center y {:?}", g.center());
        // from 90deg → start_angle = (90 − 90)° = 0 rad (scene +x reference).
        assert!(
            near(g.start_angle(), 0.0),
            "start_angle {}",
            g.start_angle()
        );
        assert_eq!(g.stops().len(), 2);
        assert_eq!(g.stops()[0].color(), Color::rgba(1.0, 0.0, 0.0, 1.0));
        assert!(list.is_balanced());
    }

    /// A `ComputedStyle` carrying a `background-color` and a resolved `transform`.
    fn with_background_and_transform(bg: StyleColor, matrix: [f32; 6]) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::BackgroundColor, ComputedValue::color(bg));
        style.set(
            PropertyId::Transform,
            ComputedValue::from_concrete(PropertyValue::Transform(Transform::new(matrix)))
                .expect("a transform is a concrete computed value"),
        );
        style
    }

    #[test]
    fn transformed_node_brackets_subtree_with_push_pop_transform() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        // `transform: translate(10px, 0)` over a background box.
        computed.insert(
            a,
            with_background_and_transform(
                StyleColor::rgba(0.0, 1.0, 0.0, 1.0),
                [1.0, 0.0, 0.0, 1.0, 10.0, 0.0],
            ),
        );

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 40.0, 20.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert!(list.is_balanced(), "the transform bracket is well-formed");
        let commands = list.commands();

        // The transform opens first (outermost) and closes last, bracketing the
        // node's whole subtree paint.
        let Some(PaintCommand::PushTransform(affine)) = commands.first() else {
            panic!(
                "expected a leading PushTransform, got {:?}",
                commands.first()
            );
        };
        // A pure translation commutes with the origin wrapping, so the affine is
        // exactly translate(10, 0) regardless of the (default center) origin.
        assert_eq!(affine.coeffs, [1.0, 0.0, 0.0, 1.0, 10.0, 0.0]);
        assert!(
            matches!(commands.last(), Some(PaintCommand::PopTransform)),
            "the subtree closes with a PopTransform"
        );
        // The background fill still paints inside the bracket.
        assert_eq!(fill_count(&list), 1);
    }

    #[test]
    fn untransformed_node_emits_no_transform_bracket() {
        // Regression: a plain background (no transform) emits no PushTransform.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, with_background(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 10.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert!(
            !list.commands().iter().any(|c| matches!(
                c,
                PaintCommand::PushTransform(_) | PaintCommand::PopTransform
            )),
            "no transform authored ⇒ no transform bracket"
        );
    }

    #[test]
    fn identity_transform_emits_no_bracket() {
        // An explicit identity transform (e.g. `transform: none`) brackets nothing.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_and_transform(
                StyleColor::rgba(0.0, 1.0, 0.0, 1.0),
                [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            ),
        );

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 10.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert!(
            !list
                .commands()
                .iter()
                .any(|c| matches!(c, PaintCommand::PushTransform(_))),
            "an identity transform is a no-op bracket"
        );
    }

    #[test]
    fn solid_background_still_emits_solid_brush() {
        // Regression: a plain `background-color` (no image) still emits a solid
        // brush, byte-identical to before background images existed.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, with_background(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 10.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let brushes = fill_brushes(&list);
        assert_eq!(brushes, vec![Brush::solid(Color::rgba(0.0, 1.0, 0.0, 1.0))]);
    }

    /// Collects the `DrawPath` (fill) rectangles of a display list, in paint order.
    fn fill_rects(list: &DisplayList) -> Vec<Rect> {
        list.commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawPath { path, .. } => path.bounds(),
                _ => None,
            })
            .collect()
    }

    /// Collects the `DrawShadow` commands in a display list, in paint order.
    fn shadow_commands(list: &DisplayList) -> Vec<&PaintCommand> {
        list.commands()
            .iter()
            .filter(|c| matches!(c, PaintCommand::DrawShadow { .. }))
            .collect()
    }

    /// A `ComputedStyle` carrying a `background-color` and a `box-shadow` list.
    fn with_background_and_shadow(bg: StyleColor, shadows: Vec<BoxShadow>) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::BackgroundColor, ComputedValue::color(bg));
        style.set(
            PropertyId::BoxShadow,
            ComputedValue::from_concrete(PropertyValue::BoxShadow(shadows.into_boxed_slice()))
                .expect("a box-shadow list is a concrete computed value"),
        );
        style
    }

    #[test]
    fn box_shadow_emits_draw_shadow_behind_background() {
        // A node with `box-shadow: 4px 4px 8px black` over its background.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let shadow = BoxShadow::new(
            Length::px(4.0),
            Length::px(4.0),
            Length::px(8.0),
            Length::px(0.0),
            Some(StyleColor::BLACK),
            false,
        );
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_and_shadow(StyleColor::rgba(0.0, 1.0, 0.0, 1.0), vec![shadow]),
        );

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 20.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);

        // Exactly one DrawShadow, carrying the authored params.
        let shadows = shadow_commands(&list);
        assert_eq!(shadows.len(), 1);
        let PaintCommand::DrawShadow {
            rect,
            color,
            blur,
            spread,
            offset,
            inset,
            ..
        } = shadows[0]
        else {
            panic!("expected a DrawShadow");
        };
        assert_eq!(
            *rect,
            Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 10.0))
        );
        assert_eq!(*color, obscur_scene::color::Color::BLACK);
        assert_eq!(*blur, 8.0);
        assert_eq!(*spread, 0.0);
        assert_eq!(*offset, (4.0, 4.0));
        assert!(!*inset);

        // The shadow paints BEFORE (behind) the background fill.
        let shadow_pos = list
            .commands()
            .iter()
            .position(|c| matches!(c, PaintCommand::DrawShadow { .. }))
            .expect("shadow present");
        let fill_pos = list
            .commands()
            .iter()
            .position(|c| matches!(c, PaintCommand::DrawPath { .. }))
            .expect("background present");
        assert!(
            shadow_pos < fill_pos,
            "drop shadow paints behind the background"
        );
        assert!(list.is_balanced());
    }

    #[test]
    fn no_box_shadow_emits_no_draw_shadow() {
        // Regression: a node with a background but no `box-shadow` emits no shadow.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, with_background(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 10.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert!(shadow_commands(&list).is_empty());
    }

    /// A `ComputedStyle` carrying a `background-color` and a `filter` drop-shadow
    /// list.
    fn with_background_and_filter(bg: StyleColor, shadows: Vec<BoxShadow>) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::BackgroundColor, ComputedValue::color(bg));
        let ops: Vec<FilterOp> = shadows.into_iter().map(FilterOp::DropShadow).collect();
        style.set(
            PropertyId::Filter,
            ComputedValue::from_concrete(PropertyValue::Filter(ops.into_boxed_slice()))
                .expect("a filter op list is a concrete computed value"),
        );
        style
    }

    #[test]
    fn filter_drop_shadow_emits_draw_shadow_behind_background() {
        // A node with `filter: drop-shadow(4px 4px 8px black)` over its background.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let shadow = BoxShadow::new(
            Length::px(4.0),
            Length::px(4.0),
            Length::px(8.0),
            Length::px(0.0),
            Some(StyleColor::BLACK),
            false,
        );
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_and_filter(StyleColor::rgba(0.0, 1.0, 0.0, 1.0), vec![shadow]),
        );

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 20.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);

        // Exactly one DrawShadow, carrying the authored params (spread 0, not inset).
        let shadows = shadow_commands(&list);
        assert_eq!(shadows.len(), 1);
        let PaintCommand::DrawShadow {
            rect,
            color,
            blur,
            spread,
            offset,
            inset,
            ..
        } = shadows[0]
        else {
            panic!("expected a DrawShadow");
        };
        assert_eq!(
            *rect,
            Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 10.0))
        );
        assert_eq!(*color, obscur_scene::color::Color::BLACK);
        assert_eq!(*blur, 8.0);
        assert_eq!(*spread, 0.0);
        assert_eq!(*offset, (4.0, 4.0));
        assert!(!*inset, "a drop-shadow is never inset");

        // The filter drop-shadow paints BEFORE (behind) the background fill.
        let shadow_pos = list
            .commands()
            .iter()
            .position(|c| matches!(c, PaintCommand::DrawShadow { .. }))
            .expect("shadow present");
        let fill_pos = list
            .commands()
            .iter()
            .position(|c| matches!(c, PaintCommand::DrawPath { .. }))
            .expect("background present");
        assert!(
            shadow_pos < fill_pos,
            "filter drop-shadow paints behind the background"
        );
        assert!(list.is_balanced());
    }

    #[test]
    fn no_filter_emits_no_draw_shadow() {
        // Regression: a node with a background but no `filter` emits no shadow.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, with_background(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 10.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert!(shadow_commands(&list).is_empty());
    }

    #[test]
    fn box_shadow_and_filter_drop_shadow_both_emit() {
        // A node carrying BOTH a `box-shadow` and a `filter: drop-shadow(...)`
        // emits two DrawShadows, both behind the background.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let box_shadow = BoxShadow::new(
            Length::px(2.0),
            Length::px(2.0),
            Length::px(4.0),
            Length::px(0.0),
            Some(StyleColor::BLACK),
            false,
        );
        let filter_shadow = BoxShadow::new(
            Length::px(6.0),
            Length::px(6.0),
            Length::px(12.0),
            Length::px(0.0),
            Some(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)),
            false,
        );
        let mut style =
            with_background_and_shadow(StyleColor::rgba(0.0, 1.0, 0.0, 1.0), vec![box_shadow]);
        style.set(
            PropertyId::Filter,
            ComputedValue::from_concrete(PropertyValue::Filter(
                vec![FilterOp::DropShadow(filter_shadow)].into_boxed_slice(),
            ))
            .expect("a filter op list is a concrete computed value"),
        );

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, style);

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 20.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);

        let shadows = shadow_commands(&list);
        assert_eq!(
            shadows.len(),
            2,
            "box-shadow and filter drop-shadow both paint"
        );
        // Both shadows paint before the background fill.
        let fill_pos = list
            .commands()
            .iter()
            .position(|c| matches!(c, PaintCommand::DrawPath { .. }))
            .expect("background present");
        for (i, c) in list.commands().iter().enumerate() {
            if matches!(c, PaintCommand::DrawShadow { .. }) {
                assert!(i < fill_pos, "shadows paint behind the background");
            }
        }
        assert!(list.is_balanced());
    }

    #[test]
    fn inset_box_shadow_emits_draw_shadow_over_background() {
        // A node with `box-shadow: 0 0 0 3px black inset` over its background:
        // the inset shadow emits `DrawShadow { inset: true }` and paints AFTER
        // (over) the background fill, but its geometry stays the border box (the
        // renderer confines the inner frame to it).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let shadow = BoxShadow::new(
            Length::px(0.0),
            Length::px(0.0),
            Length::px(0.0),
            Length::px(3.0),
            Some(StyleColor::BLACK),
            true,
        );
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_and_shadow(StyleColor::rgba(0.0, 1.0, 0.0, 1.0), vec![shadow]),
        );

        let mut boxes = LayoutBoxes::new();
        let bounds = box_at(0.0, 0.0, 20.0, 10.0);
        boxes.insert(a, bounds);

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);

        // Exactly one DrawShadow, flagged inset, carrying the authored params, and
        // its rect is the border box (the inset frame is bounded to it).
        let shadows = shadow_commands(&list);
        assert_eq!(shadows.len(), 1);
        let PaintCommand::DrawShadow {
            rect,
            spread,
            inset,
            ..
        } = shadows[0]
        else {
            panic!("expected a DrawShadow");
        };
        assert!(
            *inset,
            "an `inset` shadow lowers to DrawShadow{{inset:true}}"
        );
        assert_eq!(*spread, 3.0);
        assert_eq!(
            *rect,
            Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 10.0))
        );

        // The inset shadow paints AFTER (over) the background fill.
        let shadow_pos = list
            .commands()
            .iter()
            .position(|c| matches!(c, PaintCommand::DrawShadow { .. }))
            .expect("shadow present");
        let fill_pos = list
            .commands()
            .iter()
            .position(|c| matches!(c, PaintCommand::DrawPath { .. }))
            .expect("background present");
        assert!(
            fill_pos < shadow_pos,
            "inset shadow paints over the background"
        );
        assert!(list.is_balanced());
    }

    /// A `ComputedStyle` carrying a `background-color` and a `filter: blur(r)`.
    fn with_background_and_blur(bg: StyleColor, radius: f32) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::BackgroundColor, ComputedValue::color(bg));
        style.set(
            PropertyId::Filter,
            ComputedValue::from_concrete(PropertyValue::Filter(
                vec![FilterOp::Blur(radius)].into_boxed_slice(),
            ))
            .expect("a filter op list is a concrete computed value"),
        );
        style
    }

    #[test]
    fn filter_blur_brackets_subtree_with_a_blur_layer() {
        // A node with `filter: blur(6px)` over a background brackets its subtree
        // paint with a `PushLayer(blur)` … `PopLayer`, the layer clipped to its
        // border box and carrying the blur effect.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_and_blur(StyleColor::rgba(0.0, 1.0, 0.0, 1.0), 6.0),
        );

        let mut boxes = LayoutBoxes::new();
        let bounds = box_at(0.0, 0.0, 20.0, 10.0);
        boxes.insert(a, bounds);

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert!(list.is_balanced());

        // Exactly one blur PushLayer, clipped to the border box, carrying blur(6).
        let blur_layers: Vec<&SceneLayer> = list
            .commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::PushLayer(layer) if layer.effects().iter().any(|e| e.is_blur()) => {
                    Some(layer)
                }
                _ => None,
            })
            .collect();
        assert_eq!(blur_layers.len(), 1, "one blur group layer");
        let layer = blur_layers[0];
        assert_eq!(
            layer.effects(),
            &[LayerEffect::blur(6.0)],
            "the layer carries the authored blur radius"
        );
        assert_eq!(
            layer.clip().and_then(Clip::bounds),
            Some(bounds.bounds),
            "the blur layer is clipped to the element's border box"
        );

        // The blur PushLayer opens BEFORE the background fill, and the matching
        // PopLayer closes AFTER it (the layer wraps the subtree paint).
        let push = list
            .commands()
            .iter()
            .position(|c| matches!(c, PaintCommand::PushLayer(l) if l.effects().iter().any(|e| e.is_blur())))
            .expect("blur push present");
        let fill = list
            .commands()
            .iter()
            .position(|c| matches!(c, PaintCommand::DrawPath { .. }))
            .expect("background present");
        let pop = list
            .commands()
            .iter()
            .rposition(|c| matches!(c, PaintCommand::PopLayer))
            .expect("pop present");
        assert!(
            push < fill && fill < pop,
            "blur layer wraps the subtree paint"
        );
    }

    #[test]
    fn no_blur_filter_emits_no_layer() {
        // Regression: a background with only a `drop-shadow` filter (no blur) emits
        // no blur PushLayer — byte-identical to the pre-blur path.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let shadow = BoxShadow::new(
            Length::px(4.0),
            Length::px(4.0),
            Length::px(8.0),
            Length::px(0.0),
            Some(StyleColor::BLACK),
            false,
        );
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_and_filter(StyleColor::rgba(0.0, 1.0, 0.0, 1.0), vec![shadow]),
        );

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 20.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert!(
            !list
                .commands()
                .iter()
                .any(|c| matches!(c, PaintCommand::PushLayer(l) if l.effects().iter().any(|e| e.is_blur()))),
            "a drop-shadow-only filter emits no blur layer"
        );
    }

    /// A `ComputedStyle` carrying a `background-color` and a `filter` op list.
    fn with_background_and_filter_ops(bg: StyleColor, ops: Vec<FilterOp>) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::BackgroundColor, ComputedValue::color(bg));
        style.set(
            PropertyId::Filter,
            ComputedValue::from_concrete(PropertyValue::Filter(ops.into_boxed_slice()))
                .expect("a filter op list is a concrete computed value"),
        );
        style
    }

    #[test]
    fn filter_color_op_brackets_subtree_with_a_color_matrix_layer() {
        // A node with `filter: grayscale(1)` (a color op, no blur) brackets its
        // subtree with a PushLayer carrying a ColorMatrix effect and NO blur.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_and_filter_ops(
                StyleColor::rgba(0.0, 1.0, 0.0, 1.0),
                vec![FilterOp::Grayscale(1.0)],
            ),
        );

        let mut boxes = LayoutBoxes::new();
        let bounds = box_at(0.0, 0.0, 20.0, 10.0);
        boxes.insert(a, bounds);

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert!(list.is_balanced());

        let filter_layers: Vec<&SceneLayer> = list
            .commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::PushLayer(layer)
                    if layer.effects().iter().any(|e| e.is_color_matrix()) =>
                {
                    Some(layer)
                }
                _ => None,
            })
            .collect();
        assert_eq!(filter_layers.len(), 1, "one color-matrix group layer");
        let layer = filter_layers[0];
        assert!(
            !layer.effects().iter().any(|e| e.is_blur()),
            "a color-only filter carries no blur effect"
        );
        let matrix = layer
            .effects()
            .iter()
            .find_map(|e| e.as_color_matrix())
            .expect("a color matrix effect");
        // grayscale(1) is the luma matrix.
        assert!((matrix[0] - 0.2126).abs() < 1e-6);
        assert_eq!(
            layer.clip().and_then(Clip::bounds),
            Some(bounds.bounds),
            "the filter layer is clipped to the element's border box"
        );
    }

    #[test]
    fn filter_blur_and_color_op_coexist_on_one_layer() {
        // `filter: grayscale(1) blur(4px)` puts BOTH a blur and a ColorMatrix effect
        // on the single filter layer.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_and_filter_ops(
                StyleColor::rgba(0.0, 1.0, 0.0, 1.0),
                vec![FilterOp::Grayscale(1.0), FilterOp::Blur(4.0)],
            ),
        );

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 20.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let layer = list
            .commands()
            .iter()
            .find_map(|c| match c {
                PaintCommand::PushLayer(layer)
                    if layer.effects().iter().any(|e| e.is_color_matrix()) =>
                {
                    Some(layer)
                }
                _ => None,
            })
            .expect("one filter layer");
        assert!(layer.effects().iter().any(|e| e.is_blur()), "blur present");
        assert!(
            layer.effects().iter().any(|e| e.is_color_matrix()),
            "color matrix present"
        );
    }

    #[test]
    fn no_color_op_emits_no_color_matrix_layer() {
        // Regression: a plain background (no filter) emits no ColorMatrix effect —
        // byte-identical to before color filters existed.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::BackgroundColor,
            ComputedValue::color(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)),
        );
        computed.insert(a, style);

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 20.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert!(
            !list
                .commands()
                .iter()
                .any(|c| matches!(c, PaintCommand::PushLayer(l) if l.effects().iter().any(|e| e.is_color_matrix()))),
            "no filter emits no color-matrix layer"
        );
    }

    /// A `ComputedStyle` carrying a `background-color` and a
    /// `backdrop-filter: blur(r)` (reusing the `Filter` op list).
    fn with_background_and_backdrop(bg: StyleColor, radius: f32) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::BackgroundColor, ComputedValue::color(bg));
        style.set(
            PropertyId::BackdropFilter,
            ComputedValue::from_concrete(PropertyValue::Filter(
                vec![FilterOp::Blur(radius)].into_boxed_slice(),
            ))
            .expect("a filter op list is a concrete computed value"),
        );
        style
    }

    #[test]
    fn backdrop_filter_emits_command_before_background() {
        // A node with `backdrop-filter: blur(12px)` over a background emits a
        // BackdropFilter command with its border box + radius, positioned BEFORE
        // (behind) its own background fill so the bg draws over the frosted backdrop.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            a,
            with_background_and_backdrop(StyleColor::rgba(1.0, 1.0, 1.0, 0.3), 12.0),
        );

        let mut boxes = LayoutBoxes::new();
        let bounds = box_at(2.0, 3.0, 20.0, 10.0);
        boxes.insert(a, bounds);

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert!(list.is_balanced());

        // Exactly one BackdropFilter, carrying the border box + blur radius.
        let backdrops: Vec<_> = list
            .commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::BackdropFilter { bounds, blur, .. } => Some((*bounds, *blur)),
                _ => None,
            })
            .collect();
        assert_eq!(backdrops.len(), 1, "one backdrop-filter command");
        assert_eq!(
            backdrops[0].0, bounds.bounds,
            "frost region is the border box"
        );
        assert_eq!(backdrops[0].1, 12.0, "carries the authored blur radius");

        // The BackdropFilter is emitted BEFORE the node's own background fill.
        let backdrop_pos = list
            .commands()
            .iter()
            .position(|c| matches!(c, PaintCommand::BackdropFilter { .. }))
            .expect("backdrop present");
        let fill_pos = list
            .commands()
            .iter()
            .position(|c| matches!(c, PaintCommand::DrawPath { .. }))
            .expect("background present");
        assert!(
            backdrop_pos < fill_pos,
            "backdrop-filter frosts the backdrop before the element's own bg draws over it"
        );
    }

    #[test]
    fn no_backdrop_filter_emits_no_command() {
        // Regression: a node with a background but no `backdrop-filter` emits no
        // BackdropFilter command — byte-identical to before backdrop-filter existed.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::BackgroundColor,
            ComputedValue::color(StyleColor::rgba(0.0, 0.0, 1.0, 1.0)),
        );
        computed.insert(a, style);

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 20.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert!(
            !list
                .commands()
                .iter()
                .any(|c| matches!(c, PaintCommand::BackdropFilter { .. })),
            "no backdrop-filter emits no command"
        );
    }

    /// A `ComputedStyle` with a `background-color`, `position: absolute`, and an
    /// optional numeric `z-index` — the shape the stacking sort keys on.
    fn positioned_bg(color: StyleColor, z: Option<f32>) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::BackgroundColor, ComputedValue::color(color));
        style.set(
            PropertyId::Position,
            ComputedValue::keyword(obscur_style::property::layout_keyword::POSITION_ABSOLUTE),
        );
        if let Some(z) = z {
            style.set(PropertyId::ZIndex, ComputedValue::number(z));
        }
        style
    }

    #[test]
    fn z_index_reorders_positioned_fills_above_tree_order() {
        // root -> [a (z=2), b (no z ⇒ 0), c (z=-1)] in tree order a, b, c. Despite
        // c being LAST in the tree, its z=-1 paints it FIRST; a's z=2 paints it
        // LAST (on top), overriding pure tree order.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);
        let b = add_child(&mut tree, root, 11);
        let c = add_child(&mut tree, root, 12);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new()); // root: no background.
        computed.insert(
            a,
            positioned_bg(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), Some(2.0)),
        );
        computed.insert(b, positioned_bg(StyleColor::rgba(0.0, 1.0, 0.0, 1.0), None));
        computed.insert(
            c,
            positioned_bg(StyleColor::rgba(0.0, 0.0, 1.0, 1.0), Some(-1.0)),
        );

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 10.0, 10.0));
        boxes.insert(b, box_at(1.0, 1.0, 10.0, 10.0));
        boxes.insert(c, box_at(2.0, 2.0, 10.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let rects = fill_rects(&list);
        assert_eq!(rects.len(), 3);
        // c (z=-1) first, b (z=0) middle, a (z=2) last (topmost).
        assert_eq!(
            rects[0],
            Rect::new(Point::new(2.0, 2.0), Size::new(10.0, 10.0))
        );
        assert_eq!(
            rects[1],
            Rect::new(Point::new(1.0, 1.0), Size::new(10.0, 10.0))
        );
        assert_eq!(
            rects[2],
            Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0))
        );
        assert!(list.is_balanced());
    }

    #[test]
    fn no_z_index_keeps_tree_paint_order() {
        // Regression guard: with no authored `z-index`, three positioned fills
        // paint in exact pre-order (a, b, c) — byte-identical to pre-stacking.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let a = add_child(&mut tree, root, 10);
        let b = add_child(&mut tree, root, 11);
        let c = add_child(&mut tree, root, 12);

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(a, positioned_bg(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), None));
        computed.insert(b, positioned_bg(StyleColor::rgba(0.0, 1.0, 0.0, 1.0), None));
        computed.insert(c, positioned_bg(StyleColor::rgba(0.0, 0.0, 1.0, 1.0), None));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(a, box_at(0.0, 0.0, 10.0, 10.0));
        boxes.insert(b, box_at(1.0, 1.0, 10.0, 10.0));
        boxes.insert(c, box_at(2.0, 2.0, 10.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let rects = fill_rects(&list);
        assert_eq!(
            rects,
            vec![
                Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0)),
                Rect::new(Point::new(1.0, 1.0), Size::new(10.0, 10.0)),
                Rect::new(Point::new(2.0, 2.0), Size::new(10.0, 10.0)),
            ]
        );
    }

    #[test]
    fn positioned_subtree_stacks_together_above_lower_siblings() {
        // root -> [ sib (z=0, bg), panel (z=1, positioned, bg) -> text ]. The panel
        // and its text share stacking level 1 (the text inherits the panel's z), so
        // both paint above the z=0 sibling, and the text paints above the panel's
        // OWN background (equal z, later in tree).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let sib = add_child(&mut tree, root, 20);
        let panel = add_child(&mut tree, root, 21);
        let text = add_text_child(&mut tree, panel, "hi");

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            sib,
            positioned_bg(StyleColor::rgba(0.0, 1.0, 0.0, 1.0), None),
        );
        computed.insert(
            panel,
            positioned_bg(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), Some(1.0)),
        );
        computed.insert(text, ComputedStyle::new());

        let mut boxes = LayoutBoxes::new();
        boxes.insert(sib, box_at(0.0, 0.0, 20.0, 20.0));
        boxes.insert(panel, box_at(5.0, 5.0, 20.0, 20.0));
        boxes.insert(text, box_at(6.0, 6.0, 20.0, 16.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);

        // Command kinds in paint order: sib fill, panel fill, then the text run.
        let kinds: Vec<&str> = list
            .commands()
            .iter()
            .map(|c| match c {
                PaintCommand::DrawPath { .. } => "fill",
                PaintCommand::DrawText { .. } => "text",
                _ => "other",
            })
            .collect();
        assert_eq!(kinds, vec!["fill", "fill", "text"]);

        // The two fills: the z=0 sibling first, the z=1 panel above it.
        let rects = fill_rects(&list);
        assert_eq!(
            rects[0],
            Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 20.0))
        );
        assert_eq!(
            rects[1],
            Rect::new(Point::new(5.0, 5.0), Size::new(20.0, 20.0))
        );
        // The text run sits at its box origin, painted last (on top of everything).
        let runs = text_runs(&list);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].0, Point::new(6.0, 6.0));
        assert!(list.is_balanced());
    }

    #[test]
    fn empty_tree_composes_empty_balanced_list() {
        let coordinator = SceneCoordinator::new();
        let tree = RetainedTree::new();
        let computed = ComputedStyles::new();
        let boxes = LayoutBoxes::new();

        let list = coordinator.compose(&tree, &computed, &boxes, None);
        assert!(list.is_empty());
        assert!(list.is_balanced());
    }

    #[test]
    fn root_and_two_children_fill_in_preorder() {
        // root (bg) -> [child_a (bg), child_b (no bg)].
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let child_a = add_child(&mut tree, root, 10);
        let child_b = add_child(&mut tree, root, 11);

        let root_bg = StyleColor::rgba(1.0, 0.0, 0.0, 1.0);
        let child_a_bg = StyleColor::rgba(0.0, 1.0, 0.0, 1.0);

        let mut computed = ComputedStyles::new();
        computed.insert(root, with_background(root_bg));
        computed.insert(child_a, with_background(child_a_bg));
        // child_b has a style but no background-color.
        computed.insert(child_b, ComputedStyle::new());

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 100.0, 100.0));
        boxes.insert(child_a, box_at(0.0, 0.0, 100.0, 50.0));
        boxes.insert(child_b, box_at(0.0, 50.0, 100.0, 50.0));

        let coordinator = SceneCoordinator::new();
        let list = coordinator.compose(&tree, &computed, &boxes, None);

        // Two nodes have a non-transparent background -> two fills.
        assert_eq!(fill_count(&list), 2);
        assert_eq!(list.len(), 2);
        assert!(list.is_balanced());

        // Paint order is pre-order: the root's fill is emitted first, then the
        // child's. Assert the rectangles in order.
        let rects: Vec<Rect> = list
            .commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawPath { path, .. } => path.bounds(),
                _ => None,
            })
            .collect();
        assert_eq!(rects.len(), 2);
        assert_eq!(
            rects[0],
            Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 100.0))
        );
        assert_eq!(
            rects[1],
            Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 50.0))
        );
    }

    #[test]
    fn fully_transparent_background_emits_no_fill() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let transparent = StyleColor::rgba(1.0, 0.0, 0.0, 0.0);
        let mut computed = ComputedStyles::new();
        computed.insert(root, with_background(transparent));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 10.0, 10.0));

        let coordinator = SceneCoordinator::new();
        let list = coordinator.compose(&tree, &computed, &boxes, None);

        assert!(list.is_empty());
        assert!(list.is_balanced());
    }

    #[test]
    fn node_without_a_box_emits_no_fill() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let mut computed = ComputedStyles::new();
        computed.insert(root, with_background(StyleColor::rgba(0.0, 0.0, 1.0, 1.0)));

        // No arranged box supplied for the root.
        let boxes = LayoutBoxes::new();

        let coordinator = SceneCoordinator::new();
        let list = coordinator.compose(&tree, &computed, &boxes, None);
        assert!(list.is_empty());
    }

    #[test]
    fn text_node_emits_run_at_box_origin_with_default_white() {
        // root (bg) -> text child (no color set -> default white).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "hello");

        let mut computed = ComputedStyles::new();
        computed.insert(root, with_background(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)));
        // Text node has no computed color property.
        computed.insert(text, ComputedStyle::new());

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 100.0, 100.0));
        // Box wide enough for all five 9.6px glyphs (48px) so the default `clip`
        // policy has nothing to truncate — this test exercises shaping, not overflow.
        boxes.insert(text, box_at(5.0, 12.0, 60.0, 16.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);

        // Background element still fills.
        assert_eq!(fill_count(&list), 1);

        let runs = text_runs(&list);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].0, Point::new(5.0, 12.0));
        // The run was shaped in the domain: five chars -> five positioned glyphs.
        assert_eq!(runs[0].1.text(), "hello");
        assert_eq!(runs[0].1.len(), 5);
        assert_eq!(runs[0].2, Brush::solid(Color::WHITE));

        // Leaf-only list stays balanced.
        assert!(list.is_balanced());
    }

    #[test]
    fn text_node_shapes_one_positioned_glyph_per_char_in_order() {
        // A four-char run yields four glyphs, laid out left-to-right at the
        // mock's fixed advance, on the real baseline (the ascent below origin,
        // 0.8·size for the mock — NOT the full em).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "abcd");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());
        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(0.0, 0.0, 40.0, 16.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let runs = text_runs(&list);
        assert_eq!(runs.len(), 1);
        let run = &runs[0].1;
        assert_eq!(run.len(), 4);
        let advance = DEFAULT_FONT_SIZE * 0.6;
        // The mock's idealized ascent baseline: 0.8 * 16 = 12.8, below the em.
        let baseline = DEFAULT_FONT_SIZE * 0.8;
        assert_ne!(baseline, DEFAULT_FONT_SIZE);
        for (i, glyph) in run.glyphs().iter().enumerate() {
            assert_eq!(glyph.glyph_id(), u32::from(['a', 'b', 'c', 'd'][i]));
            assert_eq!(glyph.cluster(), i);
            assert_eq!(glyph.x(), advance * i as f32);
            assert_eq!(glyph.y(), baseline);
            assert_eq!(glyph.advance(), advance);
        }
    }

    #[test]
    fn composed_glyphs_sit_on_the_ascent_baseline_not_the_full_em() {
        // Requirement (d): the baseline is the font's ascent (0.8·size for the
        // idealized mock), strictly less than the full em `DEFAULT_FONT_SIZE`.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "hi");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());
        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(0.0, 0.0, 40.0, 16.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let runs = text_runs(&list);
        let run = &runs[0].1;
        let ascent = DEFAULT_FONT_SIZE * 0.8; // 12.8
        for glyph in run.glyphs() {
            assert_eq!(glyph.y(), ascent);
            assert!(glyph.y() < DEFAULT_FONT_SIZE);
        }
    }

    #[test]
    fn text_shaping_is_deterministic_across_composes() {
        // The same input tree must produce byte-identical positioned glyphs on
        // every compose (the property both backends' agreement rests on).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "Count: 7");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());
        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(2.0, 3.0, 80.0, 16.0));

        let coordinator = SceneCoordinator::new();
        let first = text_runs(&coordinator.compose(&tree, &computed, &boxes, None));
        let second = text_runs(&coordinator.compose(&tree, &computed, &boxes, None));
        assert_eq!(first, second);
    }

    #[test]
    fn text_node_uses_its_computed_color() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "tinted");

        let color = StyleColor::rgba(0.1, 0.2, 0.3, 1.0);
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(text, with_color(color));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(1.0, 2.0, 10.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);

        let runs = text_runs(&list);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].2, Brush::solid(Color::rgba(0.1, 0.2, 0.3, 1.0)));
    }

    #[test]
    fn zero_size_text_box_still_emits_a_run() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "z");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());

        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(3.0, 4.0, 0.0, 0.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);

        let runs = text_runs(&list);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].0, Point::new(3.0, 4.0));
        assert_eq!(runs[0].1.text(), "z");
        assert_eq!(runs[0].1.len(), 1);
    }

    #[test]
    fn text_node_without_a_box_emits_no_run() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "orphan");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());

        // No arranged box for the text node.
        let boxes = LayoutBoxes::new();

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert!(text_runs(&list).is_empty());
        assert!(list.is_empty());
    }

    #[test]
    fn long_text_emits_one_run_per_wrapped_line_agreeing_with_box_height() {
        // Requirement (c): three 4-char words in a 50px-wide box wrap to three
        // lines, so the scene emits exactly three `DrawText` runs — one per line —
        // at cumulative tops (0, 16, 32) and a shared x_offset of 0. The box
        // height (48) equals N · line_height, so layout and scene AGREE.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "aaaa bbbb cccc");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());

        let mut boxes = LayoutBoxes::new();
        // Box as the layout phase would size it: 50px wide, 3·16 = 48 tall.
        boxes.insert(text, box_at(10.0, 20.0, 50.0, 48.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let runs = text_runs(&list);
        assert_eq!(runs.len(), 3, "one DrawText per wrapped line");
        // Line count · line_height (16) equals the arranged box height (48).
        assert_eq!(runs.len() as f32 * 16.0, boxes[&text].size().height);

        // Origins stack by line height, all at the box's x (x_offset 0 for Start).
        assert_eq!(runs[0].0, Point::new(10.0, 20.0));
        assert_eq!(runs[1].0, Point::new(10.0, 36.0));
        assert_eq!(runs[2].0, Point::new(10.0, 52.0));

        // Glyph counts per line: "aaaa " (5), "bbbb " (5), "cccc" (4) = 14 total.
        assert_eq!(runs[0].1.len(), 5);
        assert_eq!(runs[1].1.len(), 5);
        assert_eq!(runs[2].1.len(), 4);

        // Every glyph sits on the line baseline (ascent 0.8·16 = 12.8), and each
        // line lays its glyphs out from pen x = 0.
        for (_, run, _) in &runs {
            for (i, glyph) in run.glyphs().iter().enumerate() {
                assert_eq!(glyph.y(), 12.8);
                assert_eq!(glyph.x(), 9.6 * i as f32);
            }
        }
        assert!(list.is_balanced());
    }

    #[test]
    fn line_height_two_widens_wrapped_line_spacing() {
        // `line-height: 2` makes each wrapped line box 2·16 = 32 tall, so the three
        // wrapped lines stack by 32 (not the 16px font-metric height) and the extra
        // leading (16px) is split half above / half below — the glyph baseline
        // shifts down to 12.8 + (32-16)/2 = 20.8.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "aaaa bbbb cccc");

        let mut style = ComputedStyle::new();
        style.set(PropertyId::LineHeight, ComputedValue::number(2.0));
        let mut computed = ComputedStyles::new();
        computed.insert(text, style);

        let mut boxes = LayoutBoxes::new();
        // Box as layout sizes it under line-height:2 — 50px wide, 3·32 = 96 tall.
        boxes.insert(text, box_at(10.0, 20.0, 50.0, 96.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let runs = text_runs(&list);
        assert_eq!(runs.len(), 3, "one DrawText per wrapped line");
        // Lines spaced by the resolved 32px line height, not the 16px metric.
        assert_eq!(runs[0].0, Point::new(10.0, 20.0));
        assert_eq!(runs[1].0, Point::new(10.0, 52.0));
        assert_eq!(runs[2].0, Point::new(10.0, 84.0));
        // Half-leading centres each line's glyphs on the shifted baseline.
        for (_, run, _) in &runs {
            for glyph in run.glyphs() {
                assert_eq!(glyph.y(), 20.8);
            }
        }
        assert!(list.is_balanced());
    }

    #[test]
    fn explicit_newline_forces_a_second_line_under_pre_line() {
        // Under `white-space: pre-line` a `\n` is a preserved mandatory break (CSS
        // Text 3 §3): even in a box wide enough that nothing wraps, "ab\ncd" splits
        // into two lines, so the scene emits two runs stacked by the 16px line
        // height. (Under the initial `normal` the break would collapse to a space —
        // see `explicit_newline_collapses_to_a_space_under_normal`.)
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "ab\ncd");

        let mut computed = ComputedStyles::new();
        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::WhiteSpace,
            ComputedValue::keyword(obscur_style::property::layout_keyword::WHITE_SPACE_PRE_LINE),
        );
        computed.insert(text, style);
        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(0.0, 0.0, 1000.0, 32.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let runs = text_runs(&list);
        assert_eq!(runs.len(), 2, "\\n forces a second line under pre-line");
        assert_eq!(runs[0].0.y, 0.0);
        assert_eq!(runs[1].0.y, 16.0);
        assert!(list.is_balanced());
    }

    #[test]
    fn explicit_newline_collapses_to_a_space_under_normal() {
        // The spec-correct initial behaviour (CSS Text 3 §4.1.1): under
        // `white-space: normal` a segment break collapses to a space, so "ab\ncd"
        // in a wide box is a SINGLE line — the pre-`white-space` code path forced a
        // hard break here, which was `pre-line`-ish and wrong for the initial value.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "ab\ncd");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());
        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(0.0, 0.0, 1000.0, 32.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let runs = text_runs(&list);
        assert_eq!(runs.len(), 1, "\\n collapses to a space under normal");
        assert!(list.is_balanced());
    }

    #[test]
    fn empty_text_emits_a_single_empty_run_without_panicking() {
        // Requirement (d): an empty string yields one line with zero glyphs; the
        // scene still emits a single (empty) run at the box origin and never panics.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());
        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(3.0, 4.0, 0.0, 16.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let runs = text_runs(&list);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].0, Point::new(3.0, 4.0));
        assert_eq!(runs[0].1.len(), 0);
        assert!(list.is_balanced());
    }

    /// Builds a `ComputedStyle` with a uniform solid border of `width` px and the
    /// given per-side `color`.
    fn with_border(width: f32, color: StyleColor) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        for id in [
            PropertyId::BorderTopWidth,
            PropertyId::BorderRightWidth,
            PropertyId::BorderBottomWidth,
            PropertyId::BorderLeftWidth,
        ] {
            style.set(id, ComputedValue::length(Length::px(width)));
        }
        for id in [
            PropertyId::BorderTopStyle,
            PropertyId::BorderRightStyle,
            PropertyId::BorderBottomStyle,
            PropertyId::BorderLeftStyle,
        ] {
            style.set(
                id,
                ComputedValue::keyword(layout_keyword::BORDER_STYLE_SOLID),
            );
        }
        for id in [
            PropertyId::BorderTopColor,
            PropertyId::BorderRightColor,
            PropertyId::BorderBottomColor,
            PropertyId::BorderLeftColor,
        ] {
            style.set(id, ComputedValue::color(color));
        }
        style
    }

    /// Collects the `DrawPath` fills of a display list as `(bounds, solid color)`
    /// pairs, in paint order.
    fn path_fills(list: &DisplayList) -> Vec<(Rect, Color)> {
        list.commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawPath { path, paint } => {
                    Some((path.bounds()?, paint.brush().as_solid()?.color()))
                }
                _ => None,
            })
            .collect()
    }

    /// Collects the `DrawPath` fills of a display list as `(polygon vertices,
    /// solid color)` pairs, in paint order. The vertices are the `MoveTo`/`LineTo`
    /// anchor points of the path (a `Close` adds no point), so a mitered border
    /// quad yields its four corners and a degenerate side yields the triangle's
    /// four listed points (two of which coincide at the collapsed apex).
    fn path_polygons(list: &DisplayList) -> Vec<(Vec<Point>, Color)> {
        use obscur_scene::path::PathSegment;
        list.commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawPath { path, paint } => {
                    let pts = path
                        .segments()
                        .iter()
                        .filter_map(|s| match s {
                            PathSegment::MoveTo(p) | PathSegment::LineTo(p) => Some(*p),
                            _ => None,
                        })
                        .collect();
                    Some((pts, paint.brush().as_solid()?.color()))
                }
                _ => None,
            })
            .collect()
    }

    /// The set of DISTINCT vertices of a polygon, sorted for stable comparison.
    /// Collapsed corners (equal points) fold together, so a degenerate border
    /// quad reduces to its three triangle vertices.
    fn distinct_vertices(pts: &[Point]) -> Vec<(f32, f32)> {
        let mut v: Vec<(f32, f32)> = pts.iter().map(|p| (p.x, p.y)).collect();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v.dedup();
        v
    }

    #[test]
    fn border_paints_four_mitered_side_quads_covering_the_edges() {
        // A 100×50 box at (10,20) with `border: 2px solid red` paints four mitered
        // trapezoid quads in top/right/bottom/left order, each red, tiling the ring
        // with 45°-mitered corners (outer edge full, inner edge inset by 2px on
        // each adjacent side).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let red = StyleColor::rgba(1.0, 0.0, 0.0, 1.0);
        let mut computed = ComputedStyles::new();
        computed.insert(root, with_border(2.0, red));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(10.0, 20.0, 100.0, 50.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let polys = path_polygons(&list);
        assert_eq!(polys.len(), 4, "four side quads, no background");
        // Every side is red.
        for (_, color) in &polys {
            assert_eq!(*color, Color::rgba(1.0, 0.0, 0.0, 1.0));
        }
        // Outer corners (10,20)-(110,70); inner corners inset by 2px: (12,22)-(108,68).
        // top: outer TL,TR then inner TR,TL.
        assert_eq!(
            polys[0].0,
            vec![p(10.0, 20.0), p(110.0, 20.0), p(108.0, 22.0), p(12.0, 22.0)],
        );
        // right: outer TR,BR then inner BR,TR.
        assert_eq!(
            polys[1].0,
            vec![
                p(110.0, 20.0),
                p(110.0, 70.0),
                p(108.0, 68.0),
                p(108.0, 22.0)
            ],
        );
        // bottom: outer BR,BL then inner BL,BR.
        assert_eq!(
            polys[2].0,
            vec![p(110.0, 70.0), p(10.0, 70.0), p(12.0, 68.0), p(108.0, 68.0)],
        );
        // left: outer BL,TL then inner TL,BL.
        assert_eq!(
            polys[3].0,
            vec![p(10.0, 70.0), p(10.0, 20.0), p(12.0, 22.0), p(12.0, 68.0)],
        );
        // The four quads cover the whole 2px border band: their bounding boxes
        // union to the full outer ring (top/bottom span the full width, left/right
        // the full height — corners now shared by the miters, not overlapped).
        let fills = path_fills(&list);
        assert_eq!(
            fills[0].0,
            Rect::new(Point::new(10.0, 20.0), Size::new(100.0, 2.0))
        );
        assert_eq!(
            fills[1].0,
            Rect::new(Point::new(108.0, 20.0), Size::new(2.0, 50.0))
        );
        assert_eq!(
            fills[2].0,
            Rect::new(Point::new(10.0, 68.0), Size::new(100.0, 2.0))
        );
        assert_eq!(
            fills[3].0,
            Rect::new(Point::new(10.0, 20.0), Size::new(2.0, 50.0))
        );
        assert!(list.is_balanced());
    }

    /// Shorthand for a `Point` in the border-geometry tests.
    fn p(x: f32, y: f32) -> Point {
        Point::new(x, y)
    }

    /// Builds a `ComputedStyle` with per-side SOLID borders: `widths` and `colors`
    /// are in top/right/bottom/left order. Every side is `solid`; a zero-width or
    /// transparent side paints nothing regardless.
    fn triangle_border(widths: [f32; 4], colors: [StyleColor; 4]) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        let w_ids = [
            PropertyId::BorderTopWidth,
            PropertyId::BorderRightWidth,
            PropertyId::BorderBottomWidth,
            PropertyId::BorderLeftWidth,
        ];
        let s_ids = [
            PropertyId::BorderTopStyle,
            PropertyId::BorderRightStyle,
            PropertyId::BorderBottomStyle,
            PropertyId::BorderLeftStyle,
        ];
        let c_ids = [
            PropertyId::BorderTopColor,
            PropertyId::BorderRightColor,
            PropertyId::BorderBottomColor,
            PropertyId::BorderLeftColor,
        ];
        for (id, w) in w_ids.into_iter().zip(widths) {
            style.set(id, ComputedValue::length(Length::px(w)));
        }
        for id in s_ids {
            style.set(
                id,
                ComputedValue::keyword(layout_keyword::BORDER_STYLE_SOLID),
            );
        }
        for (id, c) in c_ids.into_iter().zip(colors) {
            style.set(id, ComputedValue::color(c));
        }
        style
    }

    #[test]
    fn border_triangle_trick_renders_the_four_arrow_triangles() {
        // The classic CSS border-triangle trick: a 0×0-content box whose border
        // box is exactly its border widths, with three sides transparent and one
        // side colored, collapses the colored side's trapezoid into a TRIANGLE —
        // the arrow/marker glyph used for select ▼, disclosure ▸▾, steppers ▲▼.
        // The border box is (left + right) × (top + bottom).
        let colored = StyleColor::rgba(0.2, 0.2, 0.2, 1.0); // #333
        let clear = StyleColor::rgba(0.0, 0.0, 0.0, 0.0); // transparent

        // (widths [t,r,b,l], colored side index, box w, box h, expected 3 vertices).
        let cases: [([f32; 4], usize, f32, f32, [(f32, f32); 3]); 4] = [
            // ▽ down — border-top colored, box 10×6: base at top, apex at bottom.
            (
                [6.0, 5.0, 0.0, 5.0],
                0,
                10.0,
                6.0,
                [(0.0, 0.0), (10.0, 0.0), (5.0, 6.0)],
            ),
            // △ up — border-bottom colored, box 10×6: base at bottom, apex at top.
            (
                [0.0, 5.0, 6.0, 5.0],
                2,
                10.0,
                6.0,
                [(0.0, 6.0), (10.0, 6.0), (5.0, 0.0)],
            ),
            // ▷ right — border-left colored, box 6×10: base at left, apex at right.
            (
                [5.0, 0.0, 5.0, 6.0],
                3,
                6.0,
                10.0,
                [(0.0, 0.0), (0.0, 10.0), (6.0, 5.0)],
            ),
            // ◁ left — border-right colored, box 6×10: base at right, apex at left.
            (
                [5.0, 6.0, 5.0, 0.0],
                1,
                6.0,
                10.0,
                [(6.0, 0.0), (6.0, 10.0), (0.0, 5.0)],
            ),
        ];

        for (widths, colored_side, bw, bh, expected) in cases {
            let mut colors = [clear; 4];
            colors[colored_side] = colored;

            let mut tree = RetainedTree::new();
            let root = tree.alloc(NodeKind::Root);
            tree.set_root(root);
            let mut computed = ComputedStyles::new();
            computed.insert(root, triangle_border(widths, colors));
            let mut boxes = LayoutBoxes::new();
            boxes.insert(root, box_at(0.0, 0.0, bw, bh));

            let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
            let polys = path_polygons(&list);
            // Only the one colored side is emitted; the three transparent sides
            // (and any zero-width side) paint nothing.
            assert_eq!(polys.len(), 1, "one triangle for widths {widths:?}");
            assert_eq!(
                polys[0].1,
                scene_color_from(colored),
                "the triangle is #333"
            );
            // Exactly three distinct vertices: the collapsed side's two outer (or
            // two inner) corners coincide at the apex.
            let mut want: Vec<(f32, f32)> = expected.to_vec();
            want.sort_by(|a, b| a.partial_cmp(b).unwrap());
            assert_eq!(
                distinct_vertices(&polys[0].0),
                want,
                "expected arrow triangle for widths {widths:?}"
            );
            assert!(list.is_balanced());
        }
    }

    #[test]
    fn transparent_border_side_is_not_emitted() {
        // A per-side border where one side is a transparent color paints only the
        // opaque sides — the transparent side emits no quad.
        let opaque = StyleColor::rgba(0.0, 0.0, 1.0, 1.0);
        let clear = StyleColor::rgba(0.0, 0.0, 0.0, 0.0);

        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        // Top transparent; right/bottom/left opaque blue.
        let mut computed = ComputedStyles::new();
        computed.insert(
            root,
            triangle_border([2.0, 2.0, 2.0, 2.0], [clear, opaque, opaque, opaque]),
        );
        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 40.0, 30.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let polys = path_polygons(&list);
        assert_eq!(
            polys.len(),
            3,
            "transparent top side skipped, three opaque sides"
        );
        for (_, color) in &polys {
            assert_eq!(*color, scene_color_from(opaque));
        }
    }

    /// Builds a `ComputedStyle` with a uniform DASHED border of `width` px and
    /// the given per-side `color`.
    fn with_dashed_border(width: f32, color: StyleColor) -> ComputedStyle {
        let mut style = with_border(width, color);
        for id in [
            PropertyId::BorderTopStyle,
            PropertyId::BorderRightStyle,
            PropertyId::BorderBottomStyle,
            PropertyId::BorderLeftStyle,
        ] {
            style.set(
                id,
                ComputedValue::keyword(layout_keyword::BORDER_STYLE_DASHED),
            );
        }
        style
    }

    /// Collects every `DrawPath` whose paint is a `Stroke`, as `(path, stroke)`.
    fn stroke_paths(list: &DisplayList) -> Vec<(Path, Stroke)> {
        list.commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawPath {
                    path,
                    paint: Paint::Stroke(stroke),
                } => Some((path.clone(), stroke.clone())),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn uniform_dashed_border_emits_a_dashed_stroke_not_fill_rects() {
        // A 100×50 box with `border: 2px dashed red` paints ONE dashed stroke
        // along the border-box centerline, not the four solid fill-rects.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let red = StyleColor::rgba(1.0, 0.0, 0.0, 1.0);
        let mut computed = ComputedStyles::new();
        computed.insert(root, with_dashed_border(2.0, red));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(10.0, 20.0, 100.0, 50.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        // No solid fill DrawPaths for the border — only the dashed stroke.
        let fill_count = list
            .commands()
            .iter()
            .filter(|c| {
                matches!(
                    c,
                    PaintCommand::DrawPath {
                        paint: Paint::Fill(_),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(fill_count, 0, "dashed border does not fill-rect");
        let strokes = stroke_paths(&list);
        assert_eq!(strokes.len(), 1, "one dashed border stroke");
        let (path, stroke) = &strokes[0];
        assert_eq!(stroke.width(), 2.0);
        assert!(stroke.is_dashed(), "carries a non-empty dash pattern");
        assert_eq!(
            stroke.brush().as_solid().map(|s| s.color()),
            Some(scene_color_from(red))
        );
        // The stroked path is the border box inset by width/2 (= 1px) on each side.
        assert_eq!(
            path.bounds(),
            Some(Rect::new(Point::new(11.0, 21.0), Size::new(98.0, 48.0)))
        );
        assert!(list.is_balanced());
    }

    #[test]
    fn solid_border_still_emits_fill_rects_not_a_dashed_stroke() {
        // Regression guard: a uniform SOLID border keeps the four side fills (now
        // mitered quads) and emits no dashed stroke.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let mut computed = ComputedStyles::new();
        computed.insert(root, with_border(2.0, StyleColor::rgba(1.0, 0.0, 0.0, 1.0)));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(10.0, 20.0, 100.0, 50.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert_eq!(path_fills(&list).len(), 4, "four solid side rects");
        assert!(
            stroke_paths(&list).is_empty(),
            "no dashed stroke for a solid border"
        );
    }

    /// Builds a `ComputedStyle` with an `outline` of `width` px, `offset` px, the
    /// given `style` keyword handle, and `color`.
    fn with_outline(
        width: f32,
        offset: f32,
        style: obscur_identity::InternedSymbol,
        color: StyleColor,
    ) -> ComputedStyle {
        let mut s = ComputedStyle::new();
        s.set(
            PropertyId::OutlineWidth,
            ComputedValue::length(Length::px(width)),
        );
        s.set(
            PropertyId::OutlineOffset,
            ComputedValue::length(Length::px(offset)),
        );
        s.set(PropertyId::OutlineStyle, ComputedValue::keyword(style));
        s.set(PropertyId::OutlineColor, ComputedValue::color(color));
        s
    }

    #[test]
    fn dashed_outline_strokes_a_ring_outside_the_border_box() {
        // A 100×50 box at (10,20) with `outline: 2px dashed; outline-offset: 3px`
        // strokes ONE dashed ring, centered `offset + width/2` = 4px outside the
        // border box, so its bounds exceed the border box by that inflation.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let teal = StyleColor::rgba(0.17, 0.16, 0.15, 1.0);
        let mut computed = ComputedStyles::new();
        computed.insert(
            root,
            with_outline(2.0, 3.0, layout_keyword::BORDER_STYLE_DASHED, teal),
        );

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(10.0, 20.0, 100.0, 50.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let strokes = stroke_paths(&list);
        assert_eq!(strokes.len(), 1, "one outline ring stroke");
        let (path, stroke) = &strokes[0];
        assert_eq!(stroke.width(), 2.0);
        assert!(stroke.is_dashed(), "dashed outline carries a dash pattern");
        assert_eq!(
            stroke.brush().as_solid().map(|s| s.color()),
            Some(scene_color_from(teal))
        );
        // Centerline at border box inflated by offset(3) + width/2(1) = 4 on each
        // side: origin (6,16), size (108,58). Its inner edge (7,17) is offset(3) px
        // OUTSIDE the border box (10,20) — a focus ring with a visible gap.
        let bounds = path.bounds().expect("outline path has bounds");
        assert_eq!(
            bounds,
            Rect::new(Point::new(6.0, 16.0), Size::new(108.0, 58.0))
        );
        assert!(
            bounds.origin.x < 10.0 && bounds.origin.y < 20.0,
            "outside the border box"
        );
        assert!(list.is_balanced());
    }

    #[test]
    fn solid_outline_strokes_a_ring_and_no_outline_emits_none() {
        // A solid outline strokes one solid (non-dashed) ring; with the style set
        // to `none` (and otherwise identical props) nothing is emitted.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 40.0, 40.0));
        let color = StyleColor::rgba(0.1, 0.2, 0.9, 1.0);

        // Solid outline → one non-dashed stroke.
        let mut solid = ComputedStyles::new();
        solid.insert(
            root,
            with_outline(2.0, 0.0, layout_keyword::BORDER_STYLE_SOLID, color),
        );
        let list = SceneCoordinator::new().compose(&tree, &solid, &boxes, None);
        let strokes = stroke_paths(&list);
        assert_eq!(strokes.len(), 1, "one solid outline ring");
        assert!(!strokes[0].1.is_dashed(), "solid outline is not dashed");

        // `outline-style: none` → no outline paint at all.
        let mut none = ComputedStyles::new();
        none.insert(
            root,
            with_outline(2.0, 0.0, layout_keyword::BORDER_STYLE_NONE, color),
        );
        let list = SceneCoordinator::new().compose(&tree, &none, &boxes, None);
        assert!(
            stroke_paths(&list).is_empty(),
            "outline-style:none emits nothing"
        );
        assert!(
            list.commands().is_empty(),
            "no outline, no other paint → empty list"
        );
    }

    #[test]
    fn no_outline_is_byte_identical() {
        // Regression guard: a plain background node with no outline paints exactly
        // as before (one fill, no stroke, no extra brackets).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let mut computed = ComputedStyles::new();
        computed.insert(root, with_background(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)));
        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 30.0, 30.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert_eq!(fill_count(&list), 1, "only the background");
        assert!(stroke_paths(&list).is_empty(), "no outline stroke");
        assert_eq!(
            list.commands().len(),
            1,
            "exactly one command (byte-identical)"
        );
    }

    #[test]
    fn rounded_node_gets_a_rounded_outline_path() {
        // A node with a border radius gets a ROUNDED outline path (its bounding box
        // is still the inflated border box), whose corner radii grow by the
        // inflation — concentric with the rounded border box.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let color = StyleColor::rgba(0.17, 0.16, 0.15, 1.0);
        let mut style = with_outline(2.0, 3.0, layout_keyword::BORDER_STYLE_SOLID, color);
        for id in [
            PropertyId::BorderTopLeftRadius,
            PropertyId::BorderTopRightRadius,
            PropertyId::BorderBottomRightRadius,
            PropertyId::BorderBottomLeftRadius,
        ] {
            style.set(id, ComputedValue::length(Length::px(8.0)));
        }
        let mut computed = ComputedStyles::new();
        computed.insert(root, style);
        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(10.0, 20.0, 100.0, 50.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let strokes = stroke_paths(&list);
        assert_eq!(strokes.len(), 1, "one rounded outline stroke");
        let (path, _) = &strokes[0];
        // The rounded path is not a plain rect (it has cubic corner segments), and
        // its bounds are the inflated border box (inflated by 4 on each side).
        assert!(
            path.segments()
                .iter()
                .any(|s| matches!(s, obscur_scene::path::PathSegment::CubicTo { .. })),
            "rounded outline uses a rounded-rect (cubic) path"
        );
        assert_eq!(
            path.bounds(),
            Some(Rect::new(Point::new(6.0, 16.0), Size::new(108.0, 58.0)))
        );
    }

    #[test]
    fn border_paints_over_background_under_no_op() {
        // Background + border on the same node: the background fills first, then
        // the four border edges paint over it (all within one node's paint unit).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let mut style = with_border(1.0, StyleColor::rgba(0.0, 0.0, 1.0, 1.0));
        style.set(
            PropertyId::BackgroundColor,
            ComputedValue::color(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)),
        );
        let mut computed = ComputedStyles::new();
        computed.insert(root, style);

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 20.0, 20.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let fills = path_fills(&list);
        assert_eq!(fills.len(), 5, "one background + four border edges");
        // The background is emitted first, covering the whole border box.
        assert_eq!(
            fills[0].0,
            Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 20.0))
        );
        assert_eq!(fills[0].1, Color::rgba(0.0, 1.0, 0.0, 1.0));
        // The remaining four are the blue border edges.
        for (_, color) in &fills[1..] {
            assert_eq!(*color, Color::rgba(0.0, 0.0, 1.0, 1.0));
        }
    }

    #[test]
    fn no_authored_border_emits_no_border_paint() {
        // Regression guard: a node with a background but no border paints exactly
        // one fill (the background) — byte-identical to before borders existed.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let mut computed = ComputedStyles::new();
        computed.insert(root, with_background(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 30.0, 30.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert_eq!(fill_count(&list), 1, "only the background, no border rects");
    }

    #[test]
    fn border_color_falls_back_to_current_color_when_unset() {
        // Width + solid style but NO border color: each edge paints in the node's
        // computed `color` (CSS `currentColor`), here blue.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let blue = StyleColor::rgba(0.0, 0.0, 1.0, 1.0);
        let mut style = ComputedStyle::new();
        style.set(PropertyId::Color, ComputedValue::color(blue));
        for id in [
            PropertyId::BorderTopWidth,
            PropertyId::BorderRightWidth,
            PropertyId::BorderBottomWidth,
            PropertyId::BorderLeftWidth,
        ] {
            style.set(id, ComputedValue::length(Length::px(2.0)));
        }
        for id in [
            PropertyId::BorderTopStyle,
            PropertyId::BorderRightStyle,
            PropertyId::BorderBottomStyle,
            PropertyId::BorderLeftStyle,
        ] {
            style.set(
                id,
                ComputedValue::keyword(layout_keyword::BORDER_STYLE_SOLID),
            );
        }
        let mut computed = ComputedStyles::new();
        computed.insert(root, style);

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 40.0, 40.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let fills = path_fills(&list);
        assert_eq!(fills.len(), 4);
        for (_, color) in &fills {
            assert_eq!(
                *color,
                Color::rgba(0.0, 0.0, 1.0, 1.0),
                "border uses currentColor"
            );
        }
    }

    #[test]
    fn border_style_none_paints_nothing_even_with_width() {
        // A width with no solid style (style unset ⇒ `none`) paints no edge, even
        // though the width still insets the box model in the layout phase.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::BorderTopWidth,
            ComputedValue::length(Length::px(4.0)),
        );
        // No border-top-style ⇒ treated as `none`.
        let mut computed = ComputedStyles::new();
        computed.insert(root, style);

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 10.0, 10.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert_eq!(fill_count(&list), 0, "no solid style ⇒ no border painted");
    }

    /// A `ComputedStyle` with a uniform solid border and a uniform `border-radius`.
    fn with_border_and_radius(width: f32, color: StyleColor, radius: f32) -> ComputedStyle {
        let mut style = with_border(width, color);
        for id in [
            PropertyId::BorderTopLeftRadius,
            PropertyId::BorderTopRightRadius,
            PropertyId::BorderBottomRightRadius,
            PropertyId::BorderBottomLeftRadius,
        ] {
            style.set(id, ComputedValue::length(Length::px(radius)));
        }
        style
    }

    #[test]
    fn uniform_rounded_border_emits_one_even_odd_ring_with_rounded_inner() {
        // A 100×50 box at (10,20) with `5px solid` + `border-radius: 14px` paints
        // ONE even-odd ring — NOT four abutting rects — so the border's INNER
        // corners round (inner radius = 14 − 5 = 9) instead of staying square.
        use obscur_scene::path::{FillRule, Path, PathSegment};

        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let red = StyleColor::rgba(1.0, 0.0, 0.0, 1.0);
        let mut computed = ComputedStyles::new();
        computed.insert(root, with_border_and_radius(5.0, red, 14.0));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(10.0, 20.0, 100.0, 50.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        // Exactly one border fill (no background) — the ring, not four rects.
        let paths: Vec<&Path> = list
            .commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawPath { path, .. } => Some(path),
                _ => None,
            })
            .collect();
        assert_eq!(paths.len(), 1, "one ring fill, not four side rects");
        let ring = paths[0];
        assert_eq!(ring.fill_rule(), FillRule::EvenOdd, "ring is even-odd");
        // Two closed rounded-rect subpaths (outer + inner) ⇒ two `MoveTo`s.
        let moves = ring
            .segments()
            .iter()
            .filter(|s| matches!(s, PathSegment::MoveTo(_)))
            .count();
        assert_eq!(moves, 2, "outer + inner subpaths");
        // The outer subpath is the border-box rounded rect; the inner subpath is
        // the border box inset by 5px on every side with radius 14 − 5 = 9.
        let outer = Path::rounded_rect(
            Rect::new(Point::new(10.0, 20.0), Size::new(100.0, 50.0)),
            CornerRadii::uniform(14.0),
        );
        let inner = Path::rounded_rect(
            Rect::new(Point::new(15.0, 25.0), Size::new(90.0, 40.0)),
            CornerRadii::uniform(9.0),
        );
        assert_eq!(&ring.segments()[..outer.len()], outer.segments());
        assert_eq!(&ring.segments()[outer.len()..], inner.segments());
        assert!(list.is_balanced());
    }

    #[test]
    fn non_uniform_rounded_border_keeps_four_rects() {
        // A rounded border with a DIFFERENTLY colored side is non-uniform, so it
        // stays the four-rect approximation (no even-odd ring).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let red = StyleColor::rgba(1.0, 0.0, 0.0, 1.0);
        let blue = StyleColor::rgba(0.0, 0.0, 1.0, 1.0);
        let mut style = with_border_and_radius(5.0, red, 14.0);
        style.set(PropertyId::BorderTopColor, ComputedValue::color(blue));
        let mut computed = ComputedStyles::new();
        computed.insert(root, style);

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(10.0, 20.0, 100.0, 50.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert_eq!(fill_count(&list), 4, "non-uniform ⇒ four side rects");
    }

    /// A `ComputedStyle` with a `background-color` and a uniform `border-radius`.
    fn with_background_and_radius(color: StyleColor, radius: f32) -> ComputedStyle {
        let mut style = with_background(color);
        for id in [
            PropertyId::BorderTopLeftRadius,
            PropertyId::BorderTopRightRadius,
            PropertyId::BorderBottomRightRadius,
            PropertyId::BorderBottomLeftRadius,
        ] {
            style.set(id, ComputedValue::length(Length::px(radius)));
        }
        style
    }

    /// Whether a `DrawPath` path has any curved (rounded) corner segment.
    fn path_has_curve(path: &obscur_scene::path::Path) -> bool {
        path.segments()
            .iter()
            .any(|s| matches!(s, obscur_scene::path::PathSegment::CubicTo { .. }))
    }

    /// A `ComputedStyle` with a background and the same **percentage** radius on
    /// every corner — the `border-radius: 50%` circle idiom.
    fn with_background_and_percent_radius(color: StyleColor, percent: f32) -> ComputedStyle {
        let mut style = with_background(color);
        for id in [
            PropertyId::BorderTopLeftRadius,
            PropertyId::BorderTopRightRadius,
            PropertyId::BorderBottomRightRadius,
            PropertyId::BorderBottomLeftRadius,
        ] {
            style.set(id, ComputedValue::length(Length::Percent(percent)));
        }
        style
    }

    /// The on-curve (anchor) points of a path, in order — `MoveTo` / `LineTo` /
    /// `CubicTo::end`. Control points are excluded: the anchors are where the
    /// corner curves meet the edges, so they carry the radii exactly.
    fn anchor_points(path: &obscur_scene::path::Path) -> Vec<Point> {
        use obscur_scene::path::PathSegment;
        path.segments()
            .iter()
            .filter_map(|s| match *s {
                PathSegment::MoveTo(p) | PathSegment::LineTo(p) => Some(p),
                PathSegment::CubicTo { end, .. } | PathSegment::QuadTo { end, .. } => Some(end),
                PathSegment::Close => None,
            })
            .collect()
    }

    #[test]
    fn border_radius_fifty_percent_paints_an_ellipse_on_a_nonsquare_box() {
        // Audit finding 4-A, the circle idiom. `border-radius: 50%` on a 200×100 box
        // must paint the ellipse CSS Backgrounds 3 §4.1 asks for: the horizontal
        // radius resolves against the WIDTH (→ 100) and the vertical against the
        // HEIGHT (→ 50). Two different numbers from one authored value.
        //
        // This is deliberately a case NO scalar radius can express, so it cannot pass
        // by accident: any implementation carrying one radius per corner must report
        // rx == ry and fail here, whichever axis (or min/max/mean) it picked. It also
        // pins the whole production chain — the percentage must survive the style
        // resolver to computed-value time, be resolved against the arranged box here
        // in paint, and reach the path builder's two axes.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let mut computed = ComputedStyles::new();
        computed.insert(
            root,
            with_background_and_percent_radius(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), 50.0),
        );

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 200.0, 100.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let fill = list
            .commands()
            .iter()
            .find_map(|c| match c {
                PaintCommand::DrawPath { path, .. } => Some(path),
                _ => None,
            })
            .expect("the rounded background fill");

        assert!(path_has_curve(fill), "a 50% radius must round the corners");

        // Read the radii straight off the anchors. `Path::rounded_rect` starts the
        // top edge at `(x0 + tl.x, y0)` and each corner curve ends where the next
        // edge begins, so on a 200×100 box at the origin the ellipse's four extreme
        // points are exactly the edge midpoints: (100,0) top, (200,50) right,
        // (100,100) bottom, (0,50) left. A circle of ANY radius r would instead put
        // them at (r,0), (200, r), … with the same r on both axes.
        let anchors = anchor_points(fill);
        let near = |p: Point, x: f32, y: f32| (p.x - x).abs() < 1e-3 && (p.y - y).abs() < 1e-3;
        for (x, y) in [(100.0, 0.0), (200.0, 50.0), (100.0, 100.0), (0.0, 50.0)] {
            assert!(
                anchors.iter().any(|&p| near(p, x, y)),
                "50% on a 200×100 box must put an on-curve point at ({x}, {y}) — \
                 rx = w/2 = 100, ry = h/2 = 50. Anchors: {anchors:?}"
            );
        }

        // The corner points themselves are NOT on the path: the ellipse rounds them
        // fully away. This is what makes it an ellipse rather than a rounded rect.
        for (x, y) in [(0.0, 0.0), (200.0, 0.0), (200.0, 100.0), (0.0, 100.0)] {
            assert!(
                !anchors.iter().any(|&p| near(p, x, y)),
                "the box corner ({x}, {y}) must be rounded away by a 50% radius"
            );
        }

        // The ellipse still exactly inscribes the border box.
        assert_eq!(
            fill.bounds(),
            Some(Rect::new(Point::new(0.0, 0.0), Size::new(200.0, 100.0)))
        );
    }

    #[test]
    fn percent_radius_resolves_per_axis_against_the_border_box() {
        // The resolution half of the rule above, isolated from the path builder:
        // `50%` on a 200×100 box is (100, 50) per corner — width for x, height for y
        // — and NOT the same number twice.
        let mut computed = ComputedStyles::new();
        let node = NodeId::from_raw(1);
        computed.insert(
            node,
            with_background_and_percent_radius(StyleColor::rgba(0.0, 0.0, 0.0, 1.0), 50.0),
        );
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(200.0, 100.0));
        let radii = corner_radii_of(&computed, node, Some(bounds));
        for c in radii.corners() {
            assert_eq!(c, CornerRadius::new(100.0, 50.0));
        }
        // §4.5 must leave the idiom alone: every side sum equals that side's length
        // exactly, so f == 1 and the ellipse survives normalization untouched.
        assert_eq!(radii.normalized(bounds), radii, "50% is §4.5's fixed point");

        // Without an arranged box a percentage has no basis and yields a square
        // corner — it must never silently resolve against zero or a stale box.
        assert_eq!(
            corner_radii_of(&computed, node, None),
            CornerRadii::default()
        );
    }

    #[test]
    fn border_radius_emits_rounded_fill_and_clip_bracketing_children() {
        // root (bg + radius 8) -> child (bg, no radius). The rounded root fills a
        // curved path and its subtree is bracketed by a clip; the child paints
        // inside the clip.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let child = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(
            root,
            with_background_and_radius(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), 8.0),
        );
        computed.insert(child, with_background(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 100.0, 100.0));
        boxes.insert(child, box_at(10.0, 10.0, 20.0, 20.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let cmds = list.commands();
        // PushClip, rounded root fill, child fill, PopClip — the clip brackets the
        // whole subtree.
        assert!(
            matches!(cmds[0], PaintCommand::PushClip(_)),
            "clip opens first"
        );
        assert!(
            matches!(cmds.last(), Some(PaintCommand::PopClip)),
            "clip closes last"
        );
        assert!(list.is_balanced(), "the clip bracket is balanced");

        // The rounded root background is a curved path (not a plain rectangle),
        // and its bounds still equal the border box.
        match &cmds[1] {
            PaintCommand::DrawPath { path, .. } => {
                assert!(
                    path_has_curve(path),
                    "rounded background uses a curved path"
                );
                assert_eq!(
                    path.bounds(),
                    Some(Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 100.0)))
                );
            }
            other => panic!("expected the rounded background fill, got {other:?}"),
        }
        // The child's plain-rect fill sits between the push and the pop.
        match &cmds[2] {
            PaintCommand::DrawPath { path, .. } => {
                assert!(!path_has_curve(path), "child has no radius ⇒ plain rect");
            }
            other => panic!("expected the child fill, got {other:?}"),
        }
    }

    #[test]
    fn no_radius_emits_plain_fill_and_no_clip() {
        // Regression guard: a background with no authored radius paints exactly one
        // plain-rectangle fill and emits NO clip — byte-identical to pre-radius.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let mut computed = ComputedStyles::new();
        computed.insert(root, with_background(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 30.0, 30.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let cmds = list.commands();
        assert_eq!(cmds.len(), 1, "just the background, no clip");
        match &cmds[0] {
            PaintCommand::DrawPath { path, .. } => {
                assert!(!path_has_curve(path), "no radius ⇒ plain rectangle");
                assert_eq!(
                    path,
                    &obscur_scene::path::Path::rect(Rect::new(
                        Point::new(0.0, 0.0),
                        Size::new(30.0, 30.0)
                    )),
                    "byte-identical plain-rect fill"
                );
            }
            other => panic!("expected a plain fill, got {other:?}"),
        }
        assert!(
            !cmds
                .iter()
                .any(|c| matches!(c, PaintCommand::PushClip(_) | PaintCommand::PopClip)),
            "no clip commands with no radius"
        );
    }

    /// A `ComputedStyle` carrying a `background-color` and an `overflow` keyword.
    fn with_background_and_overflow(
        color: StyleColor,
        overflow: obscur_identity::InternedSymbol,
    ) -> ComputedStyle {
        let mut style = with_background(color);
        style.set(PropertyId::Overflow, ComputedValue::keyword(overflow));
        style
    }

    /// A `root (overflow:auto, 100×60)` with a child overflowing to 120 tall, and
    /// the matching `(tree, computed, boxes)` — the fixture the scroll tests share.
    fn scroll_fixture() -> (RetainedTree, ComputedStyles, LayoutBoxes, NodeId, NodeId) {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let child = add_child(&mut tree, root, 10);
        let mut computed = ComputedStyles::new();
        computed.insert(
            root,
            with_background_and_overflow(
                StyleColor::rgba(1.0, 1.0, 1.0, 1.0),
                layout_keyword::OVERFLOW_AUTO,
            ),
        );
        computed.insert(child, with_background(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)));
        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 100.0, 60.0));
        boxes.insert(child, box_at(0.0, 0.0, 100.0, 120.0)); // content overflows (120 > 60)
        (tree, computed, boxes, root, child)
    }

    #[test]
    fn empty_scroll_map_is_byte_identical_to_compose() {
        // Byte-identical guard: composing with an EMPTY scroll map reproduces
        // `compose` command-for-command (no scroll transform, no scrollbar).
        let (tree, computed, boxes, _root, _child) = scroll_fixture();
        let coord = SceneCoordinator::new();
        let plain = coord.compose(&tree, &computed, &boxes, None);
        let empty = coord.compose_with_images_scrolled(
            &tree,
            &computed,
            &boxes,
            None,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(
            plain.commands(),
            empty.commands(),
            "empty scroll map ⇒ identical list"
        );
    }

    #[test]
    fn scroll_container_brackets_content_in_a_translate_inside_the_clip() {
        // With a scroll entry the coordinator wraps the DESCENDANTS in a
        // translate(0,-80) nested INSIDE the overflow clip, and paints a scrollbar
        // (track + thumb) AFTER the clip closes (on top, unscrolled).
        let (tree, computed, boxes, root, _child) = scroll_fixture();
        let mut scroll = HashMap::new();
        scroll.insert(
            root,
            ScrollPaint {
                scroll_top: 80.0,
                content_height: 120.0,
                shows_scrollbar: true,
            },
        );
        let list = SceneCoordinator::new().compose_with_images_scrolled(
            &tree,
            &computed,
            &boxes,
            None,
            &HashMap::new(),
            &HashMap::new(),
            &scroll,
            &HashMap::new(),
        );
        let cmds = list.commands();
        assert!(list.is_balanced(), "scroll + clip brackets stay balanced");

        let clip_open = cmds
            .iter()
            .position(|c| matches!(c, PaintCommand::PushClip(_)))
            .unwrap();
        let scroll_push = cmds
            .iter()
            .position(|c| matches!(c, PaintCommand::PushTransform(t) if (t.coeffs[5] + 80.0).abs() < 0.01))
            .expect("a translate(0,-80) scroll transform");
        let scroll_pop = cmds
            .iter()
            .rposition(|c| matches!(c, PaintCommand::PopTransform))
            .unwrap();
        let clip_close = cmds
            .iter()
            .rposition(|c| matches!(c, PaintCommand::PopClip))
            .unwrap();
        // Clip opens outside the scroll transform; the scroll transform closes
        // inside the clip (scroll_pop before clip_close).
        assert!(
            clip_open < scroll_push,
            "scroll transform nests inside the clip"
        );
        assert!(
            scroll_pop < clip_close,
            "scroll transform closes before the clip"
        );

        // The scrollbar: two fills AFTER the clip closes (on top, not scrolled).
        let fills_after_clip = cmds[clip_close + 1..]
            .iter()
            .filter(|c| matches!(c, PaintCommand::DrawPath { .. }))
            .count();
        assert_eq!(
            fills_after_clip, 2,
            "a scrollbar track + thumb paint on top"
        );
    }

    #[test]
    fn overflow_hidden_brackets_subtree_with_rect_clip() {
        // root (bg + overflow:hidden) -> child (bg) offset BEYOND the root box.
        // The subtree is bracketed by a plain-rect clip to the root's border box,
        // clipping the overflowing child.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let child = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(
            root,
            with_background_and_overflow(
                StyleColor::rgba(1.0, 0.0, 0.0, 1.0),
                layout_keyword::OVERFLOW_HIDDEN,
            ),
        );
        computed.insert(child, with_background(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 100.0, 60.0));
        // The child extends past the root's right/bottom edges.
        boxes.insert(child, box_at(80.0, 40.0, 60.0, 60.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let cmds = list.commands();
        // PushClip first, PopClip last — the clip brackets the whole subtree.
        match &cmds[0] {
            PaintCommand::PushClip(Clip::Shape(shape)) => {
                assert!(
                    matches!(shape, obscur_scene::clip::ClipShape::Rect(_)),
                    "overflow with no radius ⇒ a plain rect clip, got {shape:?}"
                );
                assert_eq!(
                    shape.bounds(),
                    Some(Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 60.0))),
                    "clip is the root's border box"
                );
            }
            other => panic!("expected a PushClip rect, got {other:?}"),
        }
        assert!(
            matches!(cmds.last(), Some(PaintCommand::PopClip)),
            "clip closes last"
        );
        assert!(list.is_balanced(), "the clip bracket is balanced");
    }

    #[test]
    fn overflow_visible_adds_no_clip() {
        // Regression guard: `overflow: visible` (the initial) adds NO clip — the
        // overflowing child is not clipped, byte-identical to before overflow.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let child = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(
            root,
            with_background_and_overflow(
                StyleColor::rgba(1.0, 0.0, 0.0, 1.0),
                layout_keyword::OVERFLOW_VISIBLE,
            ),
        );
        computed.insert(child, with_background(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 100.0, 60.0));
        boxes.insert(child, box_at(80.0, 40.0, 60.0, 60.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        assert!(
            !list
                .commands()
                .iter()
                .any(|c| matches!(c, PaintCommand::PushClip(_) | PaintCommand::PopClip)),
            "overflow: visible adds no clip"
        );
    }

    #[test]
    fn overflow_with_radius_emits_single_rounded_clip() {
        // A node with BOTH a border-radius and overflow:hidden emits exactly ONE
        // clip — the rounded border box — not two.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let child = add_child(&mut tree, root, 10);

        let mut root_style = with_background_and_radius(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), 8.0);
        root_style.set(
            PropertyId::Overflow,
            ComputedValue::keyword(layout_keyword::OVERFLOW_HIDDEN),
        );
        let mut computed = ComputedStyles::new();
        computed.insert(root, root_style);
        computed.insert(child, with_background(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 100.0, 60.0));
        boxes.insert(child, box_at(80.0, 40.0, 60.0, 60.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let cmds = list.commands();
        let pushes = cmds
            .iter()
            .filter(|c| matches!(c, PaintCommand::PushClip(_)))
            .count();
        assert_eq!(pushes, 1, "radius + overflow ⇒ a single clip, not two");
        match &cmds[0] {
            PaintCommand::PushClip(Clip::Shape(shape)) => assert!(
                matches!(shape, obscur_scene::clip::ClipShape::RoundedRect { .. }),
                "the single clip is the ROUNDED border box, got {shape:?}"
            ),
            other => panic!("expected a rounded PushClip, got {other:?}"),
        }
        assert!(list.is_balanced(), "the clip bracket is balanced");
    }

    /// A `ComputedStyle` carrying a `background-color` and an `opacity`.
    fn with_background_and_opacity(color: StyleColor, opacity: f32) -> ComputedStyle {
        let mut style = with_background(color);
        style.set(PropertyId::Opacity, ComputedValue::number(opacity));
        style
    }

    #[test]
    fn opacity_below_one_brackets_subtree_with_push_pop_layer() {
        // root (bg + opacity 0.5) -> child (bg). The subtree is bracketed by a
        // PushLayer(opacity=0.5) … PopLayer wrapping BOTH fills as one group.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let child = add_child(&mut tree, root, 10);

        let mut computed = ComputedStyles::new();
        computed.insert(
            root,
            with_background_and_opacity(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), 0.5),
        );
        computed.insert(child, with_background(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)));

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 100.0, 100.0));
        boxes.insert(child, box_at(10.0, 10.0, 20.0, 20.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let cmds = list.commands();
        assert!(list.is_balanced(), "the layer bracket is balanced");
        // PushLayer opens first, PopLayer closes last, with both fills between.
        match &cmds[0] {
            PaintCommand::PushLayer(layer) => {
                assert!(
                    (layer.opacity() - 0.5).abs() < 1e-6,
                    "layer opacity is 0.5, got {}",
                    layer.opacity()
                );
            }
            other => panic!("expected a leading PushLayer, got {other:?}"),
        }
        assert!(
            matches!(cmds.last(), Some(PaintCommand::PopLayer)),
            "layer closes last"
        );
        // Exactly one layer pair wraps the whole group (not one per descendant).
        assert_eq!(
            cmds.iter()
                .filter(|c| matches!(c, PaintCommand::PushLayer(_)))
                .count(),
            1,
            "one group layer for the subtree"
        );
        assert_eq!(fill_count(&list), 2, "both fills are inside the one group");
    }

    #[test]
    fn opacity_one_and_absent_emit_no_layer() {
        // Regression: opacity 1.0 (and an absent opacity) emit NO PushLayer, so the
        // command stream is byte-identical to before opacity layers existed.
        for opacity in [None, Some(1.0_f32)] {
            let mut tree = RetainedTree::new();
            let root = tree.alloc(NodeKind::Root);
            tree.set_root(root);

            let mut computed = ComputedStyles::new();
            let style = match opacity {
                Some(a) => with_background_and_opacity(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), a),
                None => with_background(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)),
            };
            computed.insert(root, style);

            let mut boxes = LayoutBoxes::new();
            boxes.insert(root, box_at(0.0, 0.0, 30.0, 30.0));

            let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
            let cmds = list.commands();
            assert!(
                !cmds
                    .iter()
                    .any(|c| matches!(c, PaintCommand::PushLayer(_) | PaintCommand::PopLayer)),
                "opacity {opacity:?} emits no layer commands"
            );
            assert_eq!(
                cmds.len(),
                1,
                "just the one background fill for {opacity:?}"
            );
        }
    }

    #[test]
    fn opacity_nests_transform_outer_layer_middle_clip_inner() {
        // A single node carrying transform + border-radius + opacity 0.5. The three
        // wrappers must nest transform (outer) → opacity layer → rounded clip
        // (inner) on enter, and unwind clip → layer → transform on exit.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let mut style = with_background_and_radius(StyleColor::rgba(1.0, 0.0, 0.0, 1.0), 8.0);
        // translate(10, 0) — a non-identity transform.
        style.set(
            PropertyId::Transform,
            ComputedValue::from_concrete(PropertyValue::Transform(Transform::new([
                1.0, 0.0, 0.0, 1.0, 10.0, 0.0,
            ])))
            .expect("a transform is a concrete computed value"),
        );
        style.set(PropertyId::Opacity, ComputedValue::number(0.5));

        let mut computed = ComputedStyles::new();
        computed.insert(root, style);

        let mut boxes = LayoutBoxes::new();
        boxes.insert(root, box_at(0.0, 0.0, 100.0, 100.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let cmds = list.commands();
        assert!(list.is_balanced(), "the three brackets are balanced");

        // Enter order: transform (outer), layer (middle), clip (inner).
        assert!(
            matches!(cmds[0], PaintCommand::PushTransform(_)),
            "transform opens outermost, got {:?}",
            cmds[0]
        );
        assert!(
            matches!(cmds[1], PaintCommand::PushLayer(_)),
            "layer opens inside the transform, got {:?}",
            cmds[1]
        );
        assert!(
            matches!(cmds[2], PaintCommand::PushClip(_)),
            "clip opens innermost, got {:?}",
            cmds[2]
        );
        // Exit order (reversed): clip closes first, then layer, then transform.
        let n = cmds.len();
        assert!(
            matches!(cmds[n - 3], PaintCommand::PopClip),
            "clip closes first, got {:?}",
            cmds[n - 3]
        );
        assert!(
            matches!(cmds[n - 2], PaintCommand::PopLayer),
            "layer closes after the clip, got {:?}",
            cmds[n - 2]
        );
        assert!(
            matches!(cmds[n - 1], PaintCommand::PopTransform),
            "transform closes outermost, got {:?}",
            cmds[n - 1]
        );
    }

    #[test]
    fn scene_color_from_copies_components() {
        let style_color = StyleColor::rgba(0.25, 0.5, 0.75, 0.9);
        let color = scene_color_from(style_color);
        assert_eq!(color, Color::rgba(0.25, 0.5, 0.75, 0.9));
    }

    /// A stub [`TextShaper`] that emits a fixed glyph id per source char,
    /// independent of the character, so an installed shaper is observable in the
    /// emitted [`GlyphRun`].
    struct FixedIdShaper(u32);

    impl obscur_text::shaping::TextShaper for FixedIdShaper {
        fn shape(
            &self,
            request: &obscur_text::shaping::ShapingRequest,
        ) -> obscur_text::shaping::GlyphRun {
            let glyphs = request
                .text()
                .char_indices()
                .map(|(byte, _)| obscur_text::shaping::ShapedGlyph::new(self.0, byte, 8.0))
                .collect();
            obscur_text::shaping::GlyphRun::new(
                glyphs,
                request.direction(),
                request.font_size(),
                obscur_text::span::TextRange::new(0, request.text().len()),
            )
        }

        fn can_shape(&self, _face: &FontFace) -> bool {
            true
        }
    }

    #[test]
    fn installed_shaper_changes_the_emitted_glyph_run() {
        // A single text node with an arranged box, so a run is emitted.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "ab");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());
        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(0.0, 0.0, 40.0, 16.0));

        // Default mock shaper: glyph ids are the Unicode scalars, one segment on
        // the primary face (font 0).
        let mut coordinator = SceneCoordinator::new();
        let mock_runs = text_runs(&coordinator.compose(&tree, &computed, &boxes, None));
        assert_eq!(mock_runs.len(), 1);
        assert_eq!(mock_runs[0].1.font(), 0);
        let mock_ids: Vec<u32> = mock_runs[0]
            .1
            .glyphs()
            .iter()
            .map(|g| g.glyph_id())
            .collect();
        assert_eq!(mock_ids, vec![u32::from('a'), u32::from('b')]);

        // Install a fixed-id shaper through the runtime seam: the emitted run
        // reflects it (ids all 42), still one primary-face segment.
        coordinator.set_text_shaper(Box::new(FixedIdShaper(42)));
        let stub_runs = text_runs(&coordinator.compose(&tree, &computed, &boxes, None));
        assert_eq!(stub_runs.len(), 1);
        assert_eq!(stub_runs[0].1.font(), 0);
        let stub_ids: Vec<u32> = stub_runs[0]
            .1
            .glyphs()
            .iter()
            .map(|g| g.glyph_id())
            .collect();
        assert_eq!(stub_ids, vec![42, 42]);
        assert_ne!(mock_ids, stub_ids);
    }

    /// A stub itemizer that cuts the fixed two-script string "abВГ" into a Latin
    /// run `[0..2)` and a Cyrillic run `[2..6)` (each Cyrillic char is 2 bytes).
    struct TwoScriptItemizer;

    impl TextItemizer for TwoScriptItemizer {
        fn itemize(
            &self,
            _text: &str,
            _base: TextDirection,
        ) -> Vec<obscur_text::shaping::TextItem> {
            use obscur_text::shaping::{ScriptTag, TextItem};
            use obscur_text::span::TextRange;
            vec![
                TextItem::new(
                    TextRange::new(0, 2),
                    ScriptTag::LATIN,
                    TextDirection::LeftToRight,
                    0,
                ),
                TextItem::new(
                    TextRange::new(2, 6),
                    ScriptTag::CYRILLIC,
                    TextDirection::LeftToRight,
                    0,
                ),
            ]
        }
    }

    #[test]
    fn multi_font_text_emits_one_font_tagged_run_per_segment() {
        use obscur_text::font::FontId;
        use obscur_text::shaping::ScriptTag;

        // A two-script text node "abВГ" in a wide box: one line, two same-font
        // segments (Latin→font 0, Cyrillic→font 1), so compose emits two DrawText
        // runs carrying the two distinct fonts.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "abВГ");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());
        let mut boxes = LayoutBoxes::new();
        // Wide box, one line tall (line_height 16 at the default 16px font size).
        boxes.insert(text, box_at(3.0, 4.0, 200.0, 16.0));

        let mut coordinator = SceneCoordinator::new();
        coordinator.set_itemizer(Box::new(TwoScriptItemizer));
        let mut resolver = MockFontResolver::new();
        let face = coordinator.text().default_face().clone();
        resolver.register(ScriptTag::LATIN, FontId::new(0), face.clone());
        resolver.register(ScriptTag::CYRILLIC, FontId::new(1), face);
        coordinator.set_font_resolver(Box::new(resolver));

        let runs = text_runs(&coordinator.compose(&tree, &computed, &boxes, None));
        // Two font-tagged runs on a single line: fonts 0 then 1.
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].1.font(), 0);
        assert_eq!(runs[1].1.font(), 1);
        // Both segments paint on the SAME line (same y origin) — one wrapped line,
        // so box height (16) == painted line count (1) × line_height (16).
        assert_eq!(runs[0].0.y, runs[1].0.y);
        assert_eq!(boxes[&text].size().height, 16.0);
        // The second segment starts to the right of the first by the first
        // segment's advance (2 glyphs × 9.6), continuing the pen across fonts.
        assert!((runs[1].0.x - (runs[0].0.x + 2.0 * 9.6)).abs() < 1e-3);
        // Each segment carries its own script's glyphs.
        assert_eq!(runs[0].1.text(), "ab");
        assert_eq!(runs[1].1.text(), "ВГ");
        assert!(
            coordinator
                .compose(&tree, &computed, &boxes, None)
                .is_balanced()
        );
    }

    /// A stub [`Hyphenator`] that proposes one fixed byte offset in any word long
    /// enough to hold it.
    struct StubHyphenator {
        offset: usize,
    }

    impl obscur_text::layout::Hyphenator for StubHyphenator {
        fn hyphenate(&self, word: &str) -> Vec<usize> {
            if self.offset < word.len() {
                vec![self.offset]
            } else {
                Vec::new()
            }
        }
    }

    #[test]
    fn installed_hyphenator_breaks_over_wide_word_and_renders_a_hyphen() {
        // "hyphenation" (eleven 9.6px mock glyphs = 105.6) in a 60px box. Without
        // a hyphenator the whole word overflows on one line; with a stub proposing
        // a break before byte 5 ("hyphe|nation"), compose emits two lines and the
        // head line ends with a rendered hyphen glyph (mock id 45 = '-').
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "hyphenation");

        let mut computed = ComputedStyles::new();
        computed.insert(text, ComputedStyle::new());
        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(0.0, 0.0, 60.0, 16.0));

        let mut coordinator = SceneCoordinator::new();
        // Default (no hyphenator): the over-wide word stays on one line.
        assert_eq!(
            text_runs(&coordinator.compose(&tree, &computed, &boxes, None)).len(),
            1
        );

        coordinator.set_hyphenator(Box::new(StubHyphenator { offset: 5 }));
        let runs = text_runs(&coordinator.compose(&tree, &computed, &boxes, None));
        assert_eq!(runs.len(), 2, "the word breaks into a head and a tail line");
        // The head line ("hyphe" + hyphen) ends with the mock hyphen glyph (id 45)
        // and fits the 60px box.
        let head = &runs[0].1;
        assert_eq!(
            head.glyphs().last().map(PositionedGlyph::glyph_id),
            Some(45)
        );
        let head_width: f32 = head.glyphs().iter().map(PositionedGlyph::advance).sum();
        assert!(head_width <= 60.0, "head width {head_width} must fit 60");
        // The tail line ("nation") starts on the next line, no hyphen.
        assert!(runs[1].0.y > runs[0].0.y);
        assert!(!runs[1].1.glyphs().iter().any(|g| g.glyph_id() == 45));
        assert!(
            coordinator
                .compose(&tree, &computed, &boxes, None)
                .is_balanced()
        );
    }

    // ---- Text caret + selection highlight (this step) ------------------------

    /// Collects the fill rects (path bounds + brush) of a display list, in paint
    /// order — the caret and selection highlight are the fills over a text node
    /// that carries no background.
    fn caret_fills(list: &DisplayList) -> Vec<(Rect, Brush)> {
        list.commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawPath { path, paint } => {
                    path.bounds().map(|r| (r, paint.brush().clone()))
                }
                _ => None,
            })
            .collect()
    }

    /// A tree with a single text node carrying `content` under the root, its box
    /// at the origin (a wide 200x16 line so the text never wraps), and a red text
    /// color so the caret brush is identifiable — the caret/selection fixture.
    fn editing_fixture(content: &str) -> (RetainedTree, ComputedStyles, LayoutBoxes, NodeId) {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, content);
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(text, with_color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)));
        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(0.0, 0.0, 200.0, 16.0));
        (tree, computed, boxes, text)
    }

    #[test]
    fn collapsed_caret_paints_a_thin_bar_at_the_pen_x() {
        // (a) A collapsed caret over "hi" paints one thin vertical bar at the sum
        // of the advances before the caret index: 0 at index 0, one advance at 1,
        // two at 2 (the end). Advance is the mock's fixed 16·0.6 = 9.6.
        let advance = 16.0 * MockTextShaper::ADVANCE_RATIO;
        for (index, expected_x) in [(0_usize, 0.0_f32), (1, advance), (2, advance * 2.0)] {
            let (tree, computed, boxes, text) = editing_fixture("hi");
            let view = EditingView::new(text, "hi", TextSelection::caret(index));
            let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, Some(view));

            let fills = caret_fills(&list);
            assert_eq!(fills.len(), 1, "a collapsed caret paints exactly one fill");
            let (rect, brush) = &fills[0];
            assert!(
                (rect.origin.x - expected_x).abs() < 1e-4,
                "caret x at index {index}: {} != {expected_x}",
                rect.origin.x
            );
            assert_eq!(rect.size.width, CARET_WIDTH, "the caret is a thin bar");
            assert_eq!(rect.size.height, 16.0, "the caret spans the line height");
            assert_eq!(
                *brush,
                Brush::solid(Color::rgba(1.0, 0.0, 0.0, 1.0)),
                "the caret is painted in the text color"
            );
            // The caret paints ON TOP: it is the last command (after the DrawText).
            assert!(
                matches!(list.commands().last(), Some(PaintCommand::DrawPath { .. })),
                "the caret is the last (topmost) command"
            );
            assert_eq!(text_runs(&list).len(), 1, "the glyphs still render");
        }
    }

    #[test]
    fn nonempty_selection_paints_a_highlight_behind_the_text() {
        // (b) A non-empty selection [0,1) paints one highlight rect spanning x
        // 0..advance behind the glyphs (no caret).
        let advance = 16.0 * MockTextShaper::ADVANCE_RATIO;
        let (tree, computed, boxes, text) = editing_fixture("hi");
        let view = EditingView::new(text, "hi", TextSelection::new(0, 1));
        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, Some(view));

        let fills = caret_fills(&list);
        assert_eq!(
            fills.len(),
            1,
            "a non-empty selection paints one highlight, no caret"
        );
        let (rect, brush) = &fills[0];
        assert!(
            (rect.origin.x - 0.0).abs() < 1e-4,
            "highlight starts at x(start)=0"
        );
        assert!(
            (rect.size.width - advance).abs() < 1e-4,
            "highlight spans x(start)..x(end) = one advance"
        );
        assert_eq!(rect.size.height, 16.0, "highlight spans the line height");
        assert_eq!(
            *brush,
            Brush::solid(SELECTION_COLOR),
            "documented selection color"
        );

        // The highlight paints BEHIND the text: its fill is emitted before the run.
        let fill_idx = list
            .commands()
            .iter()
            .position(|c| matches!(c, PaintCommand::DrawPath { .. }))
            .expect("a highlight fill");
        let text_idx = list
            .commands()
            .iter()
            .position(|c| matches!(c, PaintCommand::DrawText { .. }))
            .expect("a text run");
        assert!(
            fill_idx < text_idx,
            "the highlight is emitted before the glyphs"
        );
    }

    /// A text node carrying `content` under the root, box at the origin sized
    /// `w`×`h` (a narrow box wraps the text), red text — the multi-line editing
    /// fixture for wrapped selection/caret geometry.
    fn wrapped_editing_fixture(
        content: &str,
        w: f32,
        h: f32,
    ) -> (RetainedTree, ComputedStyles, LayoutBoxes, NodeId) {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, content);
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(text, with_color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)));
        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(0.0, 0.0, w, h));
        (tree, computed, boxes, text)
    }

    /// The origin of the single collapsed-caret fill (the thin `CARET_WIDTH` bar).
    fn caret_bar_origin(list: &DisplayList) -> Point {
        caret_fills(list)
            .into_iter()
            .find(|(r, _)| (r.size.width - CARET_WIDTH).abs() < 1e-4)
            .expect("a caret bar")
            .0
            .origin
    }

    #[test]
    fn single_line_selection_still_paints_exactly_one_rect() {
        // (b) A span confined to ONE line (a wide, non-wrapping box) paints a single
        // rect from x(start) to x(end) — byte-identical to the old first-line path.
        let adv = 16.0 * MockTextShaper::ADVANCE_RATIO;
        let (tree, computed, boxes, text) = editing_fixture("abcd");
        let view = EditingView::new(text, "abcd", TextSelection::new(1, 3));
        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, Some(view));

        let fills = caret_fills(&list);
        assert_eq!(fills.len(), 1, "one line → exactly one highlight rect");
        let (rect, brush) = &fills[0];
        assert_eq!(*brush, Brush::solid(SELECTION_COLOR));
        assert!(
            (rect.origin.x - adv).abs() < 1e-4,
            "starts at x(1) = one advance"
        );
        assert!(
            (rect.size.width - adv * 2.0).abs() < 1e-4,
            "spans bytes 1..3 = two advances"
        );
        assert_eq!(rect.size.height, 16.0, "spans the line height");
        assert_eq!(rect.origin.y, 0.0, "on the first line");
    }

    #[test]
    fn selection_spanning_three_wrapped_lines_paints_three_per_line_rects() {
        // (a) A 50px box wraps "aaaa bbbb cccc" to three lines; a selection [2,12)
        // covers all three, so the highlight is THREE rects — the start line's TAIL,
        // the middle line FULL, the end line's HEAD — each the domain
        // `SelectionGeometry` rect translated by the box origin (0,0, no scroll).
        let coordinator = SceneCoordinator::new();
        let (tree, computed, boxes, text) = wrapped_editing_fixture("aaaa bbbb cccc", 50.0, 48.0);
        let view = EditingView::new(text, "aaaa bbbb cccc", TextSelection::new(2, 12));
        let list = coordinator.compose(&tree, &computed, &boxes, Some(view));

        let highlights: Vec<Rect> = caret_fills(&list)
            .into_iter()
            .filter(|(_, brush)| *brush == Brush::solid(SELECTION_COLOR))
            .map(|(r, _)| r)
            .collect();

        // The painted rects equal the domain geometry over the same wrapped layout.
        let layout = coordinator.text().shape_text_lines(
            &tree,
            &computed,
            &LayoutBoxes::new(),
            text,
            "aaaa bbbb cccc",
            50.0,
        );
        let expected = SelectionGeometry::from_paragraph(&layout, 2, 12);
        assert_eq!(
            expected.len(),
            3,
            "the selection covers three wrapped lines"
        );
        assert_eq!(highlights.len(), 3, "one highlight rect per covered line");
        for (got, want) in highlights.iter().zip(expected.rects()) {
            assert_eq!(
                got, want,
                "painted rect == domain geometry (origin 0, no scroll)"
            );
        }

        // Per-line extents: start TAIL begins mid-line-0 (after 2 glyphs), the
        // interior line is FULL from x 0, the end HEAD begins at x 0 and spans 2
        // glyphs. Lines stack by the 16px line height.
        let adv = 16.0 * MockTextShaper::ADVANCE_RATIO;
        assert!(
            (highlights[0].origin.x - adv * 2.0).abs() < 1e-3,
            "start tail after 2 glyphs"
        );
        assert_eq!(highlights[0].origin.y, 0.0, "start line");
        assert_eq!(highlights[1].origin.x, 0.0, "interior full width from x 0");
        assert_eq!(highlights[1].origin.y, 16.0, "middle line");
        assert_eq!(highlights[2].origin.x, 0.0, "end head from x 0");
        assert_eq!(highlights[2].origin.y, 32.0, "end line");
        assert!(
            (highlights[2].size.width - adv * 2.0).abs() < 1e-3,
            "end head spans 2 glyphs"
        );
        assert!(list.is_balanced(), "the display list stays balanced");
    }

    #[test]
    fn caret_at_soft_wrap_boundary_honors_affinity() {
        // (c) "aaaa bbbb cccc" wraps at 50px; byte 5 (the 'b' starting line 1) is a
        // SOFT-wrap boundary shared by line 0's end and line 1's start. Downstream
        // (the default) places the caret at the LEADING edge of the next line
        // (line 1, x 0); Upstream at the TRAILING edge of the previous line (line 0,
        // its used pen width).
        let coordinator = SceneCoordinator::new();
        let (tree, computed, boxes, text) = wrapped_editing_fixture("aaaa bbbb cccc", 50.0, 48.0);
        let layout = coordinator.text().shape_text_lines(
            &tree,
            &computed,
            &LayoutBoxes::new(),
            text,
            "aaaa bbbb cccc",
            50.0,
        );

        // Downstream (default affinity) → start of line 1.
        let down = EditingView::new(text, "aaaa bbbb cccc", TextSelection::caret(5));
        let d = caret_bar_origin(&coordinator.compose(&tree, &computed, &boxes, Some(down)));
        assert_eq!(
            d.y, 16.0,
            "downstream → line 1 top (byte-identical to before)"
        );
        assert!(
            (d.x - 0.0).abs() < 1e-4,
            "downstream → line 1 leading edge (x 0)"
        );

        // Upstream affinity → end of line 0.
        let up = EditingView::new(
            text,
            "aaaa bbbb cccc",
            TextSelection::caret(5).with_affinity(CaretAffinity::Upstream),
        );
        let u = caret_bar_origin(&coordinator.compose(&tree, &computed, &boxes, Some(up)));
        assert_eq!(u.y, 0.0, "upstream → line 0 top");
        let line0_trailing = line_caret_x(&layout.lines()[0], 5);
        assert!(line0_trailing > 0.0, "line 0 has a non-zero trailing edge");
        assert!(
            (u.x - line0_trailing).abs() < 1e-3,
            "upstream → line 0 trailing pen x ({line0_trailing}), got {}",
            u.x
        );
    }

    #[test]
    fn active_preedit_splices_display_text_and_underlines_it() {
        // (IME §5.12) A composition over "ab" (caret at end, at=2) with preedit
        // "ni": compose SPLICES the preedit for display ("abni"), underlines the
        // preedit's x-range with a thin (height 1) horizontal fill spanning two
        // advances at x∈[2·adv, 4·adv), and places the caret at the preedit caret
        // (end of "ni" → display offset 4). The buffer text ("ab") is untouched.
        let advance = 16.0 * MockTextShaper::ADVANCE_RATIO;
        let (tree, computed, boxes, text) = editing_fixture("ab");
        // Collapsed anchor (at == end == 2): the preedit is a pure insertion.
        let view = EditingView::new(text, "ab", TextSelection::caret(2))
            .with_preedit(crate::orchestration::PreeditView::new("ni", 2, 2, 2));
        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, Some(view));

        // The displayed text is the spliced "abni", not the buffer "ab".
        let drawn: String = text_runs(&list)
            .iter()
            .map(|(_, run, _)| run.text().to_owned())
            .collect();
        assert_eq!(
            drawn, "abni",
            "the preedit is spliced into the display text"
        );

        let fills = caret_fills(&list);
        // Exactly two fills: the preedit underline and the caret.
        let underline = fills
            .iter()
            .find(|(r, _)| {
                (r.size.height - PREEDIT_UNDERLINE_THICKNESS).abs() < 1e-4 && r.size.width > 1.0
            })
            .expect("a thin preedit underline");
        assert!(
            (underline.0.origin.x - advance * 2.0).abs() < 1e-4,
            "underline starts at the preedit left edge (2·advance)"
        );
        assert!(
            (underline.0.size.width - advance * 2.0).abs() < 1e-4,
            "underline spans the two preedit glyphs"
        );
        let caret = fills
            .iter()
            .find(|(r, _)| (r.size.width - CARET_WIDTH).abs() < 1e-4)
            .expect("a caret bar");
        assert!(
            (caret.0.origin.x - advance * 4.0).abs() < 1e-4,
            "caret sits at the preedit caret (display offset 4)"
        );
        assert!(list.is_balanced(), "the display list stays balanced");
    }

    #[test]
    fn preedit_over_a_selection_replaces_the_selected_span_in_the_display() {
        // (IME §5.12) Composing over a non-empty selection `[1, 3)` ("bc" of
        // "abcd") must show ONLY the preedit where the selection was: the DISPLAY
        // replaces the composition anchor span, so preedit "ni" renders as "a·ni·d"
        // = "anid", NOT "a·ni·bcd" = "anibcd" (preedit beside the still-selected
        // text). The buffer text ("abcd") is untouched — this is display-only.
        let (tree, computed, boxes, text) = editing_fixture("abcd");
        let view = EditingView::new(text, "abcd", TextSelection::new(1, 3))
            .with_preedit(crate::orchestration::PreeditView::new("ni", 1, 3, 2));
        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, Some(view));

        let drawn: String = text_runs(&list)
            .iter()
            .map(|(_, run, _)| run.text().to_owned())
            .collect();
        assert_eq!(
            drawn, "anid",
            "the preedit replaces the selected span in the display, hiding it"
        );
        assert!(list.is_balanced(), "the display list stays balanced");
    }

    #[test]
    fn a_preserved_newline_paints_no_visible_glyph() {
        // (Fix 3, CSS Text 3 §4.1.1) Under `white-space: pre-line` a preserved `\n`
        // is a zero-width *forced* break: "ab\ncd" still splits into two lines, but
        // the segment break must PAINT no glyph (a control char otherwise shapes to
        // a `.notdef` tofu box). The first line's run carries glyphs for 'a' and
        // 'b' only — no positioned glyph maps to the `\n` at byte 2 — so advances
        // and caret byte offsets are untouched while the line still breaks.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, "ab\ncd");

        let mut computed = ComputedStyles::new();
        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::WhiteSpace,
            ComputedValue::keyword(obscur_style::property::layout_keyword::WHITE_SPACE_PRE_LINE),
        );
        computed.insert(text, style);
        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(0.0, 0.0, 1000.0, 32.0));

        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
        let runs = text_runs(&list);
        assert_eq!(runs.len(), 2, "the \\n still forces a second line");
        // No positioned glyph anywhere maps back to the `\n` byte (cluster 2).
        let newline_glyphs = runs
            .iter()
            .flat_map(|(_, run, _)| run.glyphs())
            .filter(|glyph| glyph.cluster() == 2)
            .count();
        assert_eq!(newline_glyphs, 0, "the \\n cluster paints no glyph");
        assert_eq!(runs[0].1.len(), 2, "line 0 draws only 'a' and 'b'");
        assert_eq!(runs[1].1.len(), 2, "line 1 draws only 'c' and 'd'");
        assert!(list.is_balanced(), "the display list stays balanced");
    }

    // ---- Caret geometry under letter-spacing + text-align (this step) --------

    /// A single text node carrying `content` at the origin in a `box_w`×16 box,
    /// red text, with `mutate` applied to its computed style so a test can author
    /// `letter-spacing` / `text-align`. Mirrors [`editing_fixture`] but exercises
    /// the spacing/alignment the caret + click geometry must honor.
    fn styled_editing_fixture(
        content: &str,
        box_w: f32,
        mutate: impl FnOnce(&mut ComputedStyle),
    ) -> (RetainedTree, ComputedStyles, LayoutBoxes, NodeId) {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let text = add_text_child(&mut tree, root, content);
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        let mut style = with_color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0));
        mutate(&mut style);
        computed.insert(text, style);
        let mut boxes = LayoutBoxes::new();
        boxes.insert(text, box_at(0.0, 0.0, box_w, 16.0));
        (tree, computed, boxes, text)
    }

    /// The painted x of the single collapsed-caret fill in `list`.
    fn caret_paint_x(list: &DisplayList) -> f32 {
        let fills = caret_fills(list);
        assert_eq!(fills.len(), 1, "exactly one caret fill");
        fills[0].0.origin.x
    }

    /// The sum of every drawn glyph advance across the display list's text runs —
    /// the painted line's used content width (single line, left-aligned).
    fn drawn_run_width(list: &DisplayList) -> f32 {
        list.commands()
            .iter()
            .filter_map(|c| match c {
                PaintCommand::DrawText { run, .. } => Some(
                    run.glyphs()
                        .iter()
                        .map(PositionedGlyph::advance)
                        .sum::<f32>(),
                ),
                _ => None,
            })
            .sum()
    }

    #[test]
    fn caret_x_honors_letter_spacing_and_click_is_the_exact_inverse() {
        // A `letter-spacing: 4px` field (CSS Text 3 §8.1): the caret at the end
        // sits on the painted run's right edge — the SPACED glyph advances, not the
        // un-spaced sum — and a click there resolves back to the last byte (an exact
        // inverse). §8.1 adds 4px after each of the 4 clusters, so the painted run
        // width is 4·9.6 + 4·4 = 54.4 (NOT the spacing-blind 4·9.6 = 38.4). The
        // caret is derived from the SAME composed line, so it equals the run right
        // edge by construction.
        let advance = 16.0 * MockTextShaper::ADVANCE_RATIO; // 9.6
        let ls = 4.0_f32;
        let coordinator = SceneCoordinator::new();
        let author = |s: &mut ComputedStyle| {
            s.set(
                PropertyId::LetterSpacing,
                ComputedValue::length(Length::px(ls)),
            );
        };
        let (tree, computed, boxes, text) = styled_editing_fixture("abcd", 200.0, author);
        let view = EditingView::new(text, "abcd", TextSelection::caret(4));
        let list = coordinator.compose(&tree, &computed, &boxes, Some(view));

        let caret_x = caret_paint_x(&list);
        let run_right = drawn_run_width(&list);
        let expected = advance * 4.0 + ls * 4.0; // 54.4
        assert!(
            (caret_x - expected).abs() < 0.5,
            "caret x at end is the spaced run width ~{expected}, got {caret_x}",
        );
        assert!(
            (caret_x - run_right).abs() < 0.5,
            "caret sits on the painted run right edge {run_right}, got {caret_x}",
        );
        assert!(
            (caret_x - advance * 4.0).abs() > 1.0,
            "caret x is NOT the spacing-blind sum {}",
            advance * 4.0,
        );
        // The click at the caret x is the exact inverse: it resolves to text.len().
        let byte = caret_offset_at_point(
            coordinator.text(),
            &tree,
            &computed,
            &LayoutBoxes::new(),
            text,
            "abcd",
            200.0,
            caret_x,
            8.0,
        );
        assert_eq!(byte, 4, "click at the end caret x returns text.len()");
        // A caret at a middle boundary and its click are also exact inverses.
        let (tree2, computed2, boxes2, text2) = styled_editing_fixture("abcd", 200.0, author);
        let mid = coordinator.compose(
            &tree2,
            &computed2,
            &boxes2,
            Some(EditingView::new(text2, "abcd", TextSelection::caret(2))),
        );
        let mid_x = caret_paint_x(&mid);
        assert!(
            (mid_x - (advance + ls) * 2.0).abs() < 0.5,
            "caret at byte 2 is two spaced advances ~{}, got {mid_x}",
            (advance + ls) * 2.0,
        );
        assert_eq!(
            caret_offset_at_point(
                coordinator.text(),
                &tree2,
                &computed2,
                &LayoutBoxes::new(),
                text2,
                "abcd",
                200.0,
                mid_x,
                8.0
            ),
            2,
            "click at the byte-2 caret x returns 2",
        );
    }

    #[test]
    fn caret_x_honors_text_align_right_and_click_is_the_exact_inverse() {
        // A `text-align: right` field (CSS 2.1 §16.2): the line is shifted right by
        // `x_offset = box − line_width`, so the end caret sits at the box's right
        // edge (the right-aligned last glyph) and a click there maps back to the
        // last byte. No spacing → line width 4·9.6 = 38.4, box 200 → x_offset 161.6,
        // end caret x = 161.6 + 38.4 = 200.
        let advance = 16.0 * MockTextShaper::ADVANCE_RATIO; // 9.6
        let box_w = 200.0_f32;
        let coordinator = SceneCoordinator::new();
        let author = |s: &mut ComputedStyle| {
            s.set(
                PropertyId::TextAlign,
                ComputedValue::keyword(layout_keyword::TEXT_ALIGN_RIGHT),
            );
        };
        let (tree, computed, boxes, text) = styled_editing_fixture("abcd", box_w, author);
        let view = EditingView::new(text, "abcd", TextSelection::caret(4));
        let list = coordinator.compose(&tree, &computed, &boxes, Some(view));

        let caret_x = caret_paint_x(&list);
        assert!(
            (caret_x - box_w).abs() < 0.5,
            "right-aligned end caret sits at the box right edge {box_w}, got {caret_x}",
        );
        assert_eq!(
            caret_offset_at_point(
                coordinator.text(),
                &tree,
                &computed,
                &LayoutBoxes::new(),
                text,
                "abcd",
                box_w,
                caret_x,
                8.0
            ),
            4,
            "click at the right-aligned end caret x returns text.len()",
        );
        // A middle boundary is still an exact inverse: caret at 1 → x_offset + 1·adv.
        let (t1, c1, b1, n1) = styled_editing_fixture("abcd", box_w, author);
        let mid = coordinator.compose(
            &t1,
            &c1,
            &b1,
            Some(EditingView::new(n1, "abcd", TextSelection::caret(1))),
        );
        let mid_x = caret_paint_x(&mid);
        let x_offset = box_w - advance * 4.0; // 161.6
        assert!(
            (mid_x - (x_offset + advance)).abs() < 0.5,
            "right-aligned caret at byte 1 is x_offset + 1 advance ~{}, got {mid_x}",
            x_offset + advance,
        );
        assert_eq!(
            caret_offset_at_point(
                coordinator.text(),
                &t1,
                &c1,
                &LayoutBoxes::new(),
                n1,
                "abcd",
                box_w,
                mid_x,
                8.0
            ),
            1,
            "click at the right-aligned byte-1 caret x returns 1",
        );
    }

    #[test]
    fn caret_x_honors_text_align_center_and_click_is_the_exact_inverse() {
        // A `text-align: center` field (CSS 2.1 §16.2): `x_offset = (box −
        // line_width)/2`. No spacing → line width 38.4, box 200 → x_offset 80.8, end
        // caret x = 80.8 + 38.4 = 119.2; a click there maps back to text.len().
        let advance = 16.0 * MockTextShaper::ADVANCE_RATIO;
        let box_w = 200.0_f32;
        let coordinator = SceneCoordinator::new();
        let author = |s: &mut ComputedStyle| {
            s.set(
                PropertyId::TextAlign,
                ComputedValue::keyword(layout_keyword::TEXT_ALIGN_CENTER),
            );
        };
        let (tree, computed, boxes, text) = styled_editing_fixture("abcd", box_w, author);
        let view = EditingView::new(text, "abcd", TextSelection::caret(4));
        let list = coordinator.compose(&tree, &computed, &boxes, Some(view));

        let caret_x = caret_paint_x(&list);
        let x_offset = (box_w - advance * 4.0) / 2.0; // 80.8
        let expected = x_offset + advance * 4.0; // 119.2
        assert!(
            (caret_x - expected).abs() < 0.5,
            "centered end caret is x_offset + run width ~{expected}, got {caret_x}",
        );
        assert_eq!(
            caret_offset_at_point(
                coordinator.text(),
                &tree,
                &computed,
                &LayoutBoxes::new(),
                text,
                "abcd",
                box_w,
                caret_x,
                8.0
            ),
            4,
            "click at the centered end caret x returns text.len()",
        );
    }

    #[test]
    fn caret_x_is_byte_identical_for_a_plain_left_aligned_field() {
        // The default field — no letter/word-spacing, no tab, `text-align: start`
        // (spacing 0, x_offset 0) — is byte-identical: the caret x at each boundary
        // is exactly `index · advance`, the same flat un-spaced sum as before, and
        // the click is its exact inverse.
        let advance = 16.0 * MockTextShaper::ADVANCE_RATIO;
        let coordinator = SceneCoordinator::new();
        for index in 0..=4usize {
            let (tree, computed, boxes, text) = styled_editing_fixture("abcd", 200.0, |_| {});
            let view = EditingView::new(text, "abcd", TextSelection::caret(index));
            let list = coordinator.compose(&tree, &computed, &boxes, Some(view));
            let caret_x = caret_paint_x(&list);
            #[allow(clippy::cast_precision_loss)]
            let expected = advance * index as f32;
            assert!(
                (caret_x - expected).abs() < 1e-4,
                "plain field caret at {index} is index·advance {expected}, got {caret_x}",
            );
            assert_eq!(
                caret_offset_at_point(
                    coordinator.text(),
                    &tree,
                    &computed,
                    &LayoutBoxes::new(),
                    text,
                    "abcd",
                    200.0,
                    caret_x,
                    8.0,
                ),
                index,
                "plain field click at the byte-{index} caret x returns {index}",
            );
        }
    }

    #[test]
    fn session_text_overrides_retained_content() {
        // (c) The retained node carries "old" but the session says "new"; compose
        // draws the session text so a just-typed character is visible immediately.
        let (tree, computed, boxes, text) = editing_fixture("old");
        let view = EditingView::new(text, "new", TextSelection::caret(3));
        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, Some(view));

        let drawn: String = text_runs(&list)
            .iter()
            .map(|(_, run, _)| run.text().to_owned())
            .collect();
        assert_eq!(
            drawn, "new",
            "the scene shows the session text, not the retained content"
        );
    }

    #[test]
    fn no_editing_session_shows_retained_content_with_no_caret() {
        // (d) Regression: with `editing: None` the retained content renders and no
        // caret/highlight fill is emitted — byte-identical to the pre-caret path.
        let (tree, computed, boxes, _text) = editing_fixture("old");
        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);

        assert!(
            caret_fills(&list).is_empty(),
            "no caret or highlight without a session"
        );
        let drawn: String = text_runs(&list)
            .iter()
            .map(|(_, run, _)| run.text().to_owned())
            .collect();
        assert_eq!(drawn, "old", "the retained content renders unchanged");
    }

    #[test]
    fn caret_only_paints_on_the_editing_field_not_other_text() {
        // A second, non-editing text node emits no caret even while a session is
        // active for a different node.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let editing_text = add_text_child(&mut tree, root, "hi");
        let other_text = add_text_child(&mut tree, root, "yo");

        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(
            editing_text,
            with_color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)),
        );
        computed.insert(other_text, with_color(StyleColor::rgba(0.0, 1.0, 0.0, 1.0)));
        let mut boxes = LayoutBoxes::new();
        boxes.insert(editing_text, box_at(0.0, 0.0, 200.0, 16.0));
        boxes.insert(other_text, box_at(0.0, 20.0, 200.0, 16.0));

        let view = EditingView::new(editing_text, "hi", TextSelection::caret(1));
        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, Some(view));
        // Exactly one caret, over the editing node only.
        assert_eq!(
            caret_fills(&list).len(),
            1,
            "only the editing field gets a caret"
        );
        assert_eq!(text_runs(&list).len(), 2, "both text nodes still render");
    }

    /// Sets the reserved `class` list on a retained element, the way the
    /// reconciler would from a `CLASS` prop.
    fn set_class(tree: &mut RetainedTree, node: NodeId, class: &str) {
        use obscur_view::view::{CLASS, StyleAttributes, ViewProperty, ViewValue};
        let attrs =
            StyleAttributes::extract(&[ViewProperty::new(CLASS, ViewValue::Text(class.into()))]);
        if let Some(n) = tree.get_mut(node) {
            n.set_attributes(attrs);
        }
    }

    /// An empty-valued field: a `field` element whose ONLY text is a `.placeholder`
    /// overlay hint (the shape the widget emits while `:placeholder-shown` holds).
    /// The field carries the normal `ink` `color`; the overlay + hint carry the
    /// muted placeholder `color`. Returns the tree, styles, boxes (all one wide
    /// line so nothing wraps), the field node, and the hint text node.
    fn placeholder_fixture(
        hint: &str,
        ink: StyleColor,
        placeholder: StyleColor,
    ) -> (RetainedTree, ComputedStyles, LayoutBoxes, NodeId, NodeId) {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let field = add_child(&mut tree, root, 700);
        let overlay = add_child(&mut tree, field, 701);
        set_class(&mut tree, overlay, "placeholder");
        let hint_text = add_text_child(&mut tree, overlay, hint);
        let mut computed = ComputedStyles::new();
        computed.insert(root, ComputedStyle::new());
        computed.insert(field, with_color(ink));
        computed.insert(overlay, with_color(placeholder));
        computed.insert(hint_text, with_color(placeholder));
        let mut boxes = LayoutBoxes::new();
        boxes.insert(field, box_at(0.0, 0.0, 200.0, 16.0));
        boxes.insert(overlay, box_at(0.0, 0.0, 200.0, 16.0));
        boxes.insert(hint_text, box_at(0.0, 0.0, 200.0, 16.0));
        (tree, computed, boxes, field, hint_text)
    }

    /// The FIELD's normal ink and the muted `.placeholder` hint color used by the
    /// placeholder-override tests.
    const FIELD_INK: StyleColor = StyleColor::rgba(1.0, 0.0, 0.0, 1.0);
    const PLACEHOLDER_MUTED: StyleColor = StyleColor::rgba(0.6, 0.6, 0.6, 1.0);

    #[test]
    fn empty_valued_field_paints_typed_session_over_placeholder_in_field_ink() {
        // (a) Regression, CSS `:placeholder-shown`: focus an empty-valued field
        // (whose only text node is the `.placeholder` overlay) and type "abc" via the
        // session. The effective displayed text is now the SESSION buffer (non-empty),
        // so `:placeholder-shown` is FALSE — the session text paints OVER the
        // placeholder slot, in the FIELD's ink (NOT the muted hint color), with a
        // caret. Before the fix the muted placeholder showed and the typed text was
        // never painted.
        let (tree, computed, boxes, field, _hint) =
            placeholder_fixture("Type here…", FIELD_INK, PLACEHOLDER_MUTED);
        let view = EditingView::new(field, "abc", TextSelection::caret(3));
        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, Some(view));

        let runs = text_runs(&list);
        assert_eq!(
            runs.len(),
            1,
            "one painted run: the session text over the slot"
        );
        assert_eq!(
            runs[0].1.text(),
            "abc",
            "the placeholder slot paints the typed session text"
        );
        assert_eq!(
            runs[0].2.as_solid().map(|b| b.color()),
            Some(scene_color_from(FIELD_INK)),
            "typed text over the placeholder uses the field's ink, not the muted hint color",
        );
        let carets = caret_fills(&list);
        assert_eq!(
            carets.len(),
            1,
            "a caret paints into the overridden placeholder slot"
        );
        assert_eq!(
            carets[0].1.as_solid().map(|b| b.color()),
            Some(scene_color_from(FIELD_INK)),
            "the caret paints in the field's ink too",
        );
    }

    #[test]
    fn empty_valued_field_with_empty_session_keeps_the_muted_placeholder() {
        // (b) An empty focused field with an EMPTY session: `:placeholder-shown` still
        // holds (effective displayed text is empty), so the muted placeholder hint is
        // painted in its OWN muted color and NO caret is drawn into it — the session
        // does not override the slot when it has no typed text.
        let (tree, computed, boxes, field, _hint) =
            placeholder_fixture("Type here…", FIELD_INK, PLACEHOLDER_MUTED);
        let view = EditingView::new(field, "", TextSelection::caret(0));
        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, Some(view));

        let runs = text_runs(&list);
        assert_eq!(runs.len(), 1, "the muted placeholder hint still paints");
        assert_eq!(
            runs[0].1.text(),
            "Type here…",
            "the hint text, not session text"
        );
        assert_eq!(
            runs[0].2.as_solid().map(|b| b.color()),
            Some(scene_color_from(PLACEHOLDER_MUTED)),
            "an empty session leaves the placeholder in its muted color",
        );
        assert!(
            caret_fills(&list).is_empty(),
            "no caret is painted into a placeholder-shown slot",
        );
    }

    #[test]
    fn backspacing_a_field_to_empty_mid_edit_returns_the_muted_placeholder() {
        // (c) Backspace to empty mid-edit: with a non-empty session the placeholder
        // slot shows the typed text (field ink); once the session buffer is emptied
        // the SAME slot returns to the muted placeholder hint and drops the caret —
        // the override is driven live by the session buffer, frame to frame.
        let (tree, computed, boxes, field, _hint) =
            placeholder_fixture("Type here…", FIELD_INK, PLACEHOLDER_MUTED);
        let coordinator = SceneCoordinator::new();

        let typed = coordinator.compose(
            &tree,
            &computed,
            &boxes,
            Some(EditingView::new(field, "a", TextSelection::caret(1))),
        );
        assert_eq!(
            text_runs(&typed)[0].1.text(),
            "a",
            "typed char shows before backspace"
        );
        assert_eq!(caret_fills(&typed).len(), 1);

        let emptied = coordinator.compose(
            &tree,
            &computed,
            &boxes,
            Some(EditingView::new(field, "", TextSelection::caret(0))),
        );
        assert_eq!(
            text_runs(&emptied)[0].1.text(),
            "Type here…",
            "backspacing to empty returns the muted placeholder",
        );
        assert_eq!(
            text_runs(&emptied)[0].2.as_solid().map(|b| b.color()),
            Some(scene_color_from(PLACEHOLDER_MUTED)),
        );
        assert!(
            caret_fills(&emptied).is_empty(),
            "no caret once the placeholder is shown again"
        );
    }

    #[test]
    fn non_empty_value_field_is_byte_identical_regardless_of_the_placeholder_path() {
        // (d) A field with a NON-EMPTY value renders/edits exactly as before this fix:
        // its value text node (role `Value`, no `.placeholder`) is overridden by the
        // session in the value node's OWN color, with exactly one caret. The
        // placeholder override never fires — proving the change is confined to the
        // empty-value case. The composed list is byte-identical to the plain editing
        // fixture (same value node, same session).
        let (tree, computed, boxes, value) = editing_fixture("hi");
        let view = EditingView::new(value, "hi", TextSelection::caret(1));
        let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, Some(view));

        let runs = text_runs(&list);
        assert_eq!(runs.len(), 1, "one value run");
        assert_eq!(runs[0].1.text(), "hi");
        // `editing_fixture` paints red text; the run and caret keep the value node's
        // own color (no field-ink placeholder detour).
        let value_color = scene_color_from(StyleColor::rgba(1.0, 0.0, 0.0, 1.0));
        assert_eq!(runs[0].2.as_solid().map(|b| b.color()), Some(value_color));
        let carets = caret_fills(&list);
        assert_eq!(carets.len(), 1, "exactly one caret on the value node");
        assert_eq!(carets[0].1.as_solid().map(|b| b.color()), Some(value_color));
    }

    // ---- SVG stays vector, and follows the computed color (§5.17) -----------

    mod vector_content {
        use super::{add_child, box_at};
        use crate::orchestration::scene_coordinator::SceneCoordinator;
        use crate::orchestration::{ComputedStyles, LayoutBoxes};
        use obscur_geometry::Point;
        use obscur_geometry::Size;
        use obscur_identity::NodeId;
        use obscur_scene::color::Color;
        use obscur_scene::command::{DisplayList, PaintCommand};
        use obscur_scene::paint::{Brush, Paint, SceneVector, VectorPaint, VectorShape};
        use obscur_scene::path::Path;
        use obscur_style::computed::{ComputedStyle, ComputedValue};
        use obscur_style::property::{PropertyId, StyleColor};
        use obscur_tree::node::NodeKind;
        use obscur_tree::tree::RetainedTree;
        use obscur_view::view::{SRC, StyleAttributes, ViewProperty, ViewValue};
        use std::collections::HashMap;

        const SRC_KEY: &str = "icon.svg";

        /// A 10x10 document: one square filled with `currentColor`.
        fn icon() -> SceneVector {
            let mut path = Path::new();
            path.move_to(Point::new(0.0, 0.0));
            path.line_to(Point::new(10.0, 0.0));
            path.line_to(Point::new(10.0, 10.0));
            path.close();
            SceneVector::new(
                Size::new(10.0, 10.0),
                vec![VectorShape {
                    path,
                    fill: Some(VectorPaint::CurrentColor),
                    stroke: None,
                }],
            )
        }

        /// A tree with one element carrying `src="icon.svg"` and a computed `color`.
        fn fixture(color: StyleColor) -> (RetainedTree, ComputedStyles, LayoutBoxes, NodeId) {
            let mut tree = RetainedTree::new();
            let root = tree.alloc(NodeKind::Root);
            tree.set_root(root);
            let child = add_child(&mut tree, root, 10);
            if let Some(node) = tree.get_mut(child) {
                node.set_attributes(StyleAttributes::extract(&[ViewProperty::new(
                    SRC,
                    ViewValue::Text(SRC_KEY.into()),
                )]));
            }
            let mut computed = ComputedStyles::new();
            let mut style = ComputedStyle::new();
            style.set(PropertyId::Color, ComputedValue::color(color));
            computed.insert(child, style);
            let mut boxes = LayoutBoxes::new();
            boxes.insert(root, box_at(0.0, 0.0, 40.0, 40.0));
            boxes.insert(child, box_at(0.0, 0.0, 20.0, 20.0));
            (tree, computed, boxes, child)
        }

        fn vectors() -> HashMap<Box<str>, SceneVector> {
            let mut map = HashMap::new();
            map.insert(Box::<str>::from(SRC_KEY), icon());
            map
        }

        /// The fill colours of every `DrawPath` in the composed list.
        fn drawn_fills(list: &DisplayList) -> Vec<Color> {
            list.commands()
                .iter()
                .filter_map(|cmd| match cmd {
                    PaintCommand::DrawPath {
                        paint: Paint::Fill(Brush::Solid(solid)),
                        ..
                    } => Some(solid.color()),
                    _ => None,
                })
                .collect()
        }

        fn compose(color: StyleColor) -> DisplayList {
            let (tree, computed, boxes, _) = fixture(color);
            SceneCoordinator::new().compose_with_images_scrolled(
                &tree,
                &computed,
                &boxes,
                None,
                &HashMap::new(),
                &vectors(),
                &HashMap::new(),
                &HashMap::new(),
            )
        }

        #[test]
        fn an_svg_source_paints_as_vector_paths_not_an_image() {
            // §5.17 "SVG remains vector through the scene pipeline": the icon must
            // reach the renderers as outlines, never as a rasterized DrawImage.
            let list = compose(StyleColor::rgba(1.0, 0.0, 0.0, 1.0));
            assert!(
                list.commands()
                    .iter()
                    .any(|c| matches!(c, PaintCommand::DrawPath { .. })),
                "the document's outline reached the scene"
            );
            assert!(
                !list
                    .commands()
                    .iter()
                    .any(|c| matches!(c, PaintCommand::DrawImage { .. })),
                "a vector source must never be rasterized into an image"
            );
        }

        #[test]
        fn current_color_follows_the_computed_color_across_themes() {
            // THE §5.17 property, end to end: ONE parsed document, composed twice
            // under different computed `color`, painting different colours. No
            // reparse is involved — the same `SceneVector` value is reused.
            let red = compose(StyleColor::rgba(1.0, 0.0, 0.0, 1.0));
            let blue = compose(StyleColor::rgba(0.0, 0.0, 1.0, 1.0));

            assert_eq!(drawn_fills(&red), vec![Color::rgba(1.0, 0.0, 0.0, 1.0)]);
            assert_eq!(drawn_fills(&blue), vec![Color::rgba(0.0, 0.0, 1.0, 1.0)]);
        }

        #[test]
        fn the_document_is_placed_by_a_transform_that_scales_it_onto_the_box() {
            // The 10x10 document is placed into a 20x20 box: scaled x2, not
            // re-pointed. Vector geometry survives untouched.
            let list = compose(StyleColor::rgba(1.0, 0.0, 0.0, 1.0));
            let pushed = list
                .commands()
                .iter()
                .find_map(|c| match c {
                    PaintCommand::PushTransform(t) => Some(*t),
                    _ => None,
                })
                .expect("the placement is a transform");
            // A unit vector along x maps to 2 units: the scale reached the command.
            let scaled = pushed.apply(Point::new(1.0, 1.0));
            let origin = pushed.apply(Point::new(0.0, 0.0));
            assert!((scaled.x - origin.x - 2.0).abs() < 1e-4, "{pushed:?}");
            assert!((scaled.y - origin.y - 2.0).abs() < 1e-4, "{pushed:?}");
            assert!(
                list.commands()
                    .iter()
                    .any(|c| matches!(c, PaintCommand::PopTransform)),
                "the transform is popped: the stream stays balanced"
            );
        }

        #[test]
        fn an_unresolved_src_paints_nothing() {
            let (tree, computed, boxes, _) = fixture(StyleColor::rgba(1.0, 0.0, 0.0, 1.0));
            let list = SceneCoordinator::new().compose_with_images_scrolled(
                &tree,
                &computed,
                &boxes,
                None,
                &HashMap::new(),
                &HashMap::new(),
                &HashMap::new(),
                &HashMap::new(),
            );
            assert!(drawn_fills(&list).is_empty(), "no document, no paint");
        }
    }
}
