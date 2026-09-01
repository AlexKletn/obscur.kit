#![doc = include_str!("subtree_brackets.doc/_module.md")]

use obscur_geometry::{AffineTransform, Rect};
use obscur_identity::{FastMap, NodeId};
use obscur_scene::clip::Clip;
use obscur_scene::command::PaintCommand;
use obscur_scene::layer::{LayerEffect, SceneLayer};
use obscur_scene::paint::BlendMode;

use super::scroll_container_paint::scrollbar_commands;
use super::stacking::{PaintLayer, PaintUnit};

#[doc = include_str!("subtree_brackets.doc/SubtreeBrackets.md")]
#[derive(Debug, Default)]
pub(super) struct SubtreeBrackets {
    #[doc = include_str!("subtree_brackets.doc/enter_index.md")]
    enter_index: FastMap<NodeId, usize>,
    #[doc = include_str!("subtree_brackets.doc/parent_of.md")]
    parent_of: FastMap<NodeId, Option<NodeId>>,
    #[doc = include_str!("subtree_brackets.doc/depth_of.md")]
    depth_of: FastMap<NodeId, u32>,
    #[doc = include_str!("subtree_brackets.doc/preorder.md")]
    preorder: Vec<NodeId>,
    #[doc = include_str!("subtree_brackets.doc/clips.md")]
    clips: Vec<(NodeId, Clip)>,
    #[doc = include_str!("subtree_brackets.doc/transforms.md")]
    transforms: Vec<(NodeId, AffineTransform)>,
    #[doc = include_str!("subtree_brackets.doc/opacities.md")]
    opacities: Vec<(NodeId, f32)>,
    #[doc = include_str!("subtree_brackets.doc/blends.md")]
    blends: Vec<(NodeId, BlendMode)>,
    #[doc = include_str!("subtree_brackets.doc/filters.md")]
    filters: Vec<(NodeId, Rect, f32, Option<[f32; 20]>)>,
    #[doc = include_str!("subtree_brackets.doc/outlines.md")]
    outlines: Vec<(NodeId, PaintCommand)>,
    #[doc = include_str!("subtree_brackets.doc/scrolls.md")]
    scrolls: Vec<(NodeId, f32)>,
    #[doc = include_str!("subtree_brackets.doc/scrollbars.md")]
    scrollbars: Vec<(NodeId, Rect, f32, f32)>,
}

impl SubtreeBrackets {
    #[doc = include_str!("subtree_brackets.doc/enter.md")]
    pub(super) fn enter(
        &mut self,
        node: NodeId,
        parent: Option<NodeId>,
        depth: u32,
        tree_index: usize,
    ) {
        self.enter_index.insert(node, tree_index);
        self.parent_of.insert(node, parent);
        self.depth_of.insert(node, depth);
        self.preorder.push(node);
    }

    #[doc = include_str!("subtree_brackets.doc/clip.md")]
    pub(super) fn clip(&mut self, node: NodeId, clip: Clip) {
        self.clips.push((node, clip));
    }

    #[doc = include_str!("subtree_brackets.doc/transform.md")]
    pub(super) fn transform(&mut self, node: NodeId, affine: AffineTransform) {
        self.transforms.push((node, affine));
    }

    #[doc = include_str!("subtree_brackets.doc/opacity.md")]
    pub(super) fn opacity(&mut self, node: NodeId, alpha: f32) {
        self.opacities.push((node, alpha));
    }

    #[doc = include_str!("subtree_brackets.doc/blend.md")]
    pub(super) fn blend(&mut self, node: NodeId, mode: BlendMode) {
        self.blends.push((node, mode));
    }

    #[doc = include_str!("subtree_brackets.doc/filter.md")]
    pub(super) fn filter(
        &mut self,
        node: NodeId,
        bounds: Rect,
        radius: f32,
        color_matrix: Option<[f32; 20]>,
    ) {
        self.filters.push((node, bounds, radius, color_matrix));
    }

    #[doc = include_str!("subtree_brackets.doc/outline.md")]
    pub(super) fn outline(&mut self, node: NodeId, command: PaintCommand) {
        self.outlines.push((node, command));
    }

    #[doc = include_str!("subtree_brackets.doc/scroll.md")]
    pub(super) fn scroll(&mut self, node: NodeId, scroll_top: f32) {
        self.scrolls.push((node, scroll_top));
    }

    #[doc = include_str!("subtree_brackets.doc/scrollbar.md")]
    pub(super) fn scrollbar(
        &mut self,
        node: NodeId,
        scrollport: Rect,
        scroll_top: f32,
        content_height: f32,
    ) {
        self.scrollbars
            .push((node, scrollport, scroll_top, content_height));
    }

    #[doc = include_str!("subtree_brackets.doc/is_empty.md")]
    fn is_empty(&self) -> bool {
        self.clips.is_empty()
            && self.transforms.is_empty()
            && self.opacities.is_empty()
            && self.blends.is_empty()
            && self.filters.is_empty()
            && self.outlines.is_empty()
            && self.scrolls.is_empty()
            && self.scrollbars.is_empty()
    }

    #[doc = include_str!("subtree_brackets.doc/emit.md")]
    pub(super) fn emit(
        self,
        zmap: &FastMap<NodeId, i32>,
        layermap: &FastMap<NodeId, PaintLayer>,
        units: &mut Vec<PaintUnit>,
    ) {
        if self.is_empty() {
            return;
        }
        let Self {
            enter_index,
            parent_of,
            depth_of,
            preorder,
            clips,
            transforms,
            opacities,
            blends,
            filters,
            outlines,
            scrolls,
            scrollbars,
        } = self;
        let mut exit_index = enter_index.clone();
        for &node in preorder.iter().rev() {
            if let (Some(&enter), Some(Some(parent))) =
                (exit_index.get(&node), parent_of.get(&node))
            {
                let parent = *parent;
                if let Some(slot) = exit_index.get_mut(&parent) {
                    *slot = (*slot).max(enter);
                }
            }
        }
        // The `(z, layer, enter, exit, depth)` key every bracket of one node shares.
        let key = |node: NodeId| {
            let z = zmap.get(&node).copied().unwrap_or(0);
            let layer = layermap
                .get(&node)
                .copied()
                .unwrap_or(PaintLayer::NonPositioned);
            let enter = enter_index.get(&node).copied().unwrap_or(0);
            let exit = exit_index.get(&node).copied().unwrap_or(enter);
            let depth = depth_of.get(&node).copied().unwrap_or(0);
            (z, layer, enter, exit, depth)
        };

        // Scroll transform brackets are emitted BEFORE the clip brackets so that
        // when a scroll container carries both (it always does — `overflow:
        // scroll/auto` clips too), the tie at the shared `(z, exit, MAX − depth)`
        // pop key resolves — via the stable sort — with the scroll transform's
        // `PopTransform` BEFORE the clip's `PopClip` (the scroll transform closes
        // *inside* the clip). The `PushTransform` sorts AFTER the node's own
        // paint (`SCROLL_TRANSFORM_PUSH_ORDER > PAINT_ORDER`) and after the clip
        // push, so only the descendants (a strictly greater `tree_index`) are
        // scrolled — the container's own background/border stay fixed in the
        // scrollport (CSS Overflow 3 §scrollport).
        for (node, scroll_top) in scrolls {
            let (z, layer, enter, exit, depth) = key(node);
            let affine = AffineTransform::translate(0.0, -scroll_top);
            units.push(PaintUnit {
                z,
                layer,
                tree_index: enter,
                order: PaintUnit::SCROLL_TRANSFORM_PUSH_ORDER,
                commands: vec![PaintCommand::PushTransform(affine)],
            });
            units.push(PaintUnit {
                z,
                layer,
                tree_index: exit,
                order: PaintUnit::transform_pop_order(depth),
                commands: vec![PaintCommand::PopTransform],
            });
        }
        for (node, clip) in clips {
            let (z, layer, enter, exit, depth) = key(node);
            units.push(PaintUnit {
                z,
                layer,
                tree_index: enter,
                order: PaintUnit::CLIP_PUSH_ORDER,
                commands: vec![PaintCommand::PushClip(clip)],
            });
            units.push(PaintUnit {
                z,
                layer,
                tree_index: exit,
                order: PaintUnit::clip_pop_order(depth),
                commands: vec![PaintCommand::PopClip],
            });
        }
        // Opacity group layers are emitted AFTER the clip brackets and BEFORE
        // the transform brackets. On enter the `PushLayer` sorts between the
        // `PushTransform` (LAYER_PUSH_ORDER > TRANSFORM_PUSH_ORDER) and the
        // `PushClip` (LAYER_PUSH_ORDER < CLIP_PUSH_ORDER), so the layer wraps
        // the subtree inside the transform and outside the rounded clip. On
        // exit the `PopLayer` ties with the clip's/transform's pop at the same
        // `(z, exit, MAX - depth)` key; emitting it after the clip bracket but
        // before the transform bracket makes the stable sort close the clip
        // first (innermost), then the layer, then the transform (outermost).
        for (node, alpha) in opacities {
            let (z, layer, enter, exit, depth) = key(node);
            units.push(PaintUnit {
                z,
                layer,
                tree_index: enter,
                order: PaintUnit::LAYER_PUSH_ORDER,
                commands: vec![PaintCommand::PushLayer(
                    SceneLayer::new().with_opacity(alpha),
                )],
            });
            units.push(PaintUnit {
                z,
                layer,
                tree_index: exit,
                order: PaintUnit::layer_pop_order(depth),
                commands: vec![PaintCommand::PopLayer],
            });
        }
        // Blend-mode group layers are emitted like opacity layers (same push /
        // pop ordering), so a node carrying both gets two nested group layers and
        // the subtree composites as one group blended onto its backdrop.
        for (node, mode) in blends {
            let (z, layer, enter, exit, depth) = key(node);
            units.push(PaintUnit {
                z,
                layer,
                tree_index: enter,
                order: PaintUnit::LAYER_PUSH_ORDER,
                commands: vec![PaintCommand::PushLayer(
                    SceneLayer::new().with_blend_mode(mode),
                )],
            });
            units.push(PaintUnit {
                z,
                layer,
                tree_index: exit,
                order: PaintUnit::layer_pop_order(depth),
                commands: vec![PaintCommand::PopLayer],
            });
        }
        // Transform brackets are emitted AFTER the clip brackets so that when
        // a node carries both, the tie at the shared `(z, exit, MAX - depth)`
        // key resolves — via the stable sort — with the clip's `PopClip`
        // before the transform's `PopTransform` (the clip closes inside the
        // transform). The `PushTransform` sorts before the `PushClip`
        // (TRANSFORM_PUSH_ORDER < CLIP_PUSH_ORDER), so the transform is the
        // outermost wrapper of the subtree.
        for (node, affine) in transforms {
            let (z, layer, enter, exit, depth) = key(node);
            units.push(PaintUnit {
                z,
                layer,
                tree_index: enter,
                order: PaintUnit::TRANSFORM_PUSH_ORDER,
                commands: vec![PaintCommand::PushTransform(affine)],
            });
            units.push(PaintUnit {
                z,
                layer,
                tree_index: exit,
                order: PaintUnit::transform_pop_order(depth),
                commands: vec![PaintCommand::PopTransform],
            });
        }
        // Blur group layers are the OUTERMOST subtree wrapper: their push sorts
        // before every other bracket (BLUR_PUSH_ORDER < TRANSFORM_PUSH_ORDER) and
        // — emitted LAST here — their pop ties are broken to close after the
        // transform/opacity/clip pops, so the blur wraps the fully-composited
        // element. The clip is the node's border box, the blur region the
        // renderer's owned gaussian pass operates over.
        for (node, bounds, radius, color_matrix) in filters {
            let (z, paint_layer, enter, exit, depth) = key(node);
            let mut layer = SceneLayer::new().with_clip(Clip::rect(bounds));
            // Blur and color-matrix can coexist on ONE filter layer; a zero
            // radius carries no blur effect (the color-only path). The renderer
            // applies the color matrix, then the gaussian, then composites.
            if radius > 0.0 {
                layer.add_effect(LayerEffect::blur(radius));
            }
            if let Some(matrix) = color_matrix {
                layer.add_effect(LayerEffect::color_matrix(matrix));
            }
            units.push(PaintUnit {
                z,
                layer: paint_layer,
                tree_index: enter,
                order: PaintUnit::BLUR_PUSH_ORDER,
                commands: vec![PaintCommand::PushLayer(layer)],
            });
            units.push(PaintUnit {
                z,
                layer: paint_layer,
                tree_index: exit,
                order: PaintUnit::blur_pop_order(depth),
                commands: vec![PaintCommand::PopLayer],
            });
        }
        // The outline (focus ring) paints as its OWN unit at the subtree's exit
        // index with `OUTLINE_ORDER` — sorting after every `Pop*` bracket at the
        // same index — so it lands OUTSIDE the node's rounded clip and ON TOP of
        // the node's whole subtree. Emitted LAST so a tie with a depth-0
        // `PopClip` (both at `i64::MAX`) breaks in the outline's favour (the ring
        // paints after the clip closes, unclipped).
        for (node, command) in outlines {
            let (z, layer, _enter, exit, _depth) = key(node);
            units.push(PaintUnit {
                z,
                layer,
                tree_index: exit,
                order: PaintUnit::OUTLINE_ORDER,
                commands: vec![command],
            });
        }
        // A scroll container's vertical scrollbar (track + thumb), painted as its
        // OWN unit at the subtree exit index with `SCROLLBAR_ORDER` (the ceiling)
        // and emitted LAST, so it sorts after the scroll transform's
        // `PopTransform` — it does NOT scroll — and after the overflow clip's
        // `PopClip`, landing on top of the container's whole scrolled subtree
        // (CSS Overflow 3 §scrollbars). Its rects lie within the scrollport, so
        // being outside the clip is visually identical to being inside it.
        for (node, scrollport, scroll_top, content_height) in scrollbars {
            let commands = scrollbar_commands(scrollport, scroll_top, content_height);
            if commands.is_empty() {
                continue;
            }
            let (z, layer, _enter, exit, _depth) = key(node);
            units.push(PaintUnit {
                z,
                layer,
                tree_index: exit,
                order: PaintUnit::SCROLLBAR_ORDER,
                commands,
            });
        }
    }
}
