//! The paired `Push*` / `Pop*` wrappers a node's whole subtree paints inside, and
//! the order they nest in: [`SubtreeBrackets`].

use std::collections::HashMap;

use obscur_geometry::{AffineTransform, Point, Rect};
use obscur_identity::{FastBuildHasher, FastMap, NodeId};
use obscur_scene::clip::Clip;
use obscur_scene::command::PaintCommand;
use obscur_scene::layer::{LayerEffect, SceneLayer};

use super::paint_unit::PaintUnit;
use super::scroll_container_paint::scrollbar_commands;

#[doc = include_str!("../../docs/api/orchestration/subtree_brackets.doc/SubtreeBrackets.md")]
#[derive(Debug, Default)]
pub(super) struct SubtreeBrackets {
    /// Each node's pre-order enter index — its own paint unit's `tree_index`, and
    /// the index every `Push*` of that node sorts at.
    enter_index: FastMap<NodeId, usize>,
    /// Each node's parent, walked in reverse pre-order to propagate subtree exit
    /// indices up the tree.
    parent_of: FastMap<NodeId, Option<NodeId>>,
    /// Each node's tree depth, which breaks the pop tie between nested brackets
    /// that end at the same exit index (the deeper one closes first).
    depth_of: FastMap<NodeId, u32>,
    /// The pre-order sequence, walked in reverse so a descendant's exit index
    /// reaches its ancestors.
    preorder: Vec<NodeId>,
    /// A rounded / overflow-clipping element's subtree clip: the ROUNDED border box
    /// when the node has a radius (which also clips overflow), else a plain rect
    /// clip to the border box. Exactly ONE clip per node.
    clips: Vec<(NodeId, Clip, i64)>,
    /// A non-identity CSS 2D `transform`, resolved about the node's transform
    /// origin. It brackets the node's WHOLE subtree paint, outside any rounded clip.
    transforms: Vec<(NodeId, AffineTransform)>,
    groups: Vec<(NodeId, SceneLayer)>,
    /// A CSS `filter` blurring and/or colour-transforming the node's whole
    /// composited subtree: `(node, border_box, blur_radius (0 = none),
    /// color_matrix)`. The clip carries the element's border box so the renderer
    /// knows the region; this filter layer is the OUTERMOST subtree wrapper, so the
    /// Vello renderer's owned wgpu passes operate on the fully-composited element.
    filters: Vec<(NodeId, Rect, f32, Option<[f32; 20]>)>,
    /// A CSS `outline` (a focus ring): ONE stroked path OUTSIDE the node's border
    /// box. It reserves no layout space and — unlike the background/border — must
    /// NOT be clipped by the node's own rounded clip, so it is emitted at the
    /// subtree's EXIT index rather than inside the node's paint unit.
    outlines: Vec<(NodeId, PaintCommand)>,
    #[doc = include_str!("../../docs/api/orchestration/subtree_brackets.doc/scrolls.md")]
    scrolls: Vec<(NodeId, Point)>,
    /// A scroll container's vertical scrollbar: `(node, scrollport, scroll_top,
    /// content_height)`. Painted as its own unit at the subtree exit index, on top
    /// of and outside the scrolled content (CSS Overflow 3 §scrollbars).
    scrollbars: Vec<(NodeId, Rect, f32, f32, bool)>,
}

impl SubtreeBrackets {
    #[doc = include_str!("../../docs/api/orchestration/subtree_brackets.doc/with_capacity.md")]
    pub(super) fn with_capacity(nodes: usize) -> Self {
        // `enter` writes one entry into each of the three tables for EVERY node
        // the walk pops, whether or not that node ever records a bracket, so
        // their final size is the node count and is known here. Reserving it
        // replaces the ~7 rehashes each table pays growing from empty with one
        // sized allocation apiece. The bracket vectors stay unreserved: how many
        // clips or transforms a document has is not knowable from the node count,
        // and a document with none must not pay for a guess.
        Self {
            enter_index: HashMap::with_capacity_and_hasher(nodes, FastBuildHasher::default()),
            parent_of: HashMap::with_capacity_and_hasher(nodes, FastBuildHasher::default()),
            depth_of: HashMap::with_capacity_and_hasher(nodes, FastBuildHasher::default()),
            preorder: Vec::with_capacity(nodes),
            ..Self::default()
        }
    }

    #[doc = include_str!("../../docs/api/orchestration/subtree_brackets.doc/enter.md")]
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

    /// Brackets `node`'s subtree with `clip` (see [`clips`](Self::clips)).
    pub(super) fn clip(&mut self, node: NodeId, clip: Clip) {
        self.clips.push((node, clip, PaintUnit::CLIP_PUSH_ORDER));
    }

    pub(super) fn css_clip(&mut self, node: NodeId, clip: Clip) {
        self.clips
            .push((node, clip, PaintUnit::CSS_CLIP_PUSH_ORDER));
    }

    /// Brackets descendants with the scrollport padding clip.
    pub(super) fn overflow_clip(&mut self, node: NodeId, clip: Clip) {
        self.clips
            .push((node, clip, PaintUnit::OVERFLOW_CLIP_PUSH_ORDER));
    }

    /// Brackets `node`'s subtree with the resolved CSS `transform` `affine`.
    pub(super) fn transform(&mut self, node: NodeId, affine: AffineTransform) {
        self.transforms.push((node, affine));
    }

    /// Records one compositing group for opacity, blend mode and isolation.
    pub(super) fn group(&mut self, node: NodeId, layer: SceneLayer) {
        self.groups.push((node, layer));
    }

    /// Brackets `node`'s subtree with a filter group layer over `bounds` (its border
    /// box), carrying a gaussian `radius` (`0.0` = none) and/or a folded
    /// `color_matrix`.
    pub(super) fn filter(
        &mut self,
        node: NodeId,
        bounds: Rect,
        radius: f32,
        color_matrix: Option<[f32; 20]>,
    ) {
        self.filters.push((node, bounds, radius, color_matrix));
    }

    /// Records the stroked `outline` ring `command` painting on top of — and
    /// outside — `node`'s whole subtree.
    pub(super) fn outline(&mut self, node: NodeId, command: PaintCommand) {
        self.outlines.push((node, command));
    }

    /// Translates `node`'s descendants up by `scroll_top` within its scrollport.
    pub(super) fn scroll(&mut self, node: NodeId, offset: Point) {
        self.scrolls.push((node, offset));
    }

    /// Records `node`'s vertical scrollbar over `scrollport`, sized from
    /// `scroll_top` and `content_height`.
    pub(super) fn scrollbar(
        &mut self,
        node: NodeId,
        scrollport: Rect,
        scroll_top: f32,
        content_height: f32,
        horizontal: bool,
    ) {
        self.scrollbars
            .push((node, scrollport, scroll_top, content_height, horizontal));
    }

    /// Whether nothing at all was bracketed, so [`emit`](Self::emit) has no work and
    /// the walk's units are exactly what a pre-bracket compose produced.
    fn is_empty(&self) -> bool {
        self.clips.is_empty()
            && self.transforms.is_empty()
            && self.groups.is_empty()
            && self.filters.is_empty()
            && self.outlines.is_empty()
            && self.scrolls.is_empty()
            && self.scrollbars.is_empty()
    }

    #[doc = include_str!("../../docs/api/orchestration/subtree_brackets.doc/emit.md")]
    pub(super) fn emit(
        self,
        zmap: &FastMap<NodeId, i32>,
        context_end: &FastMap<NodeId, i32>,
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
            groups,
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
        // The `(z, enter, exit, depth)` key every bracket of one node shares.
        let key = |node: NodeId| {
            let z = zmap.get(&node).copied().unwrap_or(0);
            let enter = enter_index.get(&node).copied().unwrap_or(0);
            let exit = exit_index.get(&node).copied().unwrap_or(enter);
            let depth = depth_of.get(&node).copied().unwrap_or(0);
            (z, enter, exit, depth)
        };

        // Scrolling moves descendants inside the stationary scrollport.
        for (node, offset) in scrolls {
            let (z, enter, exit, depth) = key(node);
            let affine = AffineTransform::translate(-offset.x, -offset.y);
            units.push(PaintUnit {
                z,
                tree_index: enter,
                order: PaintUnit::SCROLL_TRANSFORM_PUSH_ORDER,
                commands: vec![PaintCommand::PushTransform(affine)],
            });
            units.push(PaintUnit {
                z: context_end.get(&node).copied().unwrap_or(z),
                tree_index: exit,
                order: PaintUnit::pop_order(depth, PaintUnit::SCROLL_TRANSFORM_PUSH_ORDER),
                commands: vec![PaintCommand::PopTransform],
            });
        }
        for (node, clip, order) in clips {
            let (z, enter, exit, depth) = key(node);
            units.push(PaintUnit {
                z,
                tree_index: enter,
                order,
                commands: vec![PaintCommand::PushClip(clip)],
            });
            units.push(PaintUnit {
                z: context_end.get(&node).copied().unwrap_or(z),
                tree_index: exit,
                order: PaintUnit::pop_order(depth, order),
                commands: vec![PaintCommand::PopClip],
            });
        }
        for (node, layer) in groups {
            let (z, enter, exit, depth) = key(node);
            units.push(PaintUnit {
                z,
                tree_index: enter,
                order: PaintUnit::LAYER_PUSH_ORDER,
                commands: vec![PaintCommand::PushLayer(layer)],
            });
            units.push(PaintUnit {
                z: context_end.get(&node).copied().unwrap_or(z),
                tree_index: exit,
                order: PaintUnit::pop_order(depth, PaintUnit::LAYER_PUSH_ORDER),
                commands: vec![PaintCommand::PopLayer],
            });
        }
        for (node, affine) in transforms {
            let (z, enter, exit, depth) = key(node);
            units.push(PaintUnit {
                z,
                tree_index: enter,
                order: PaintUnit::TRANSFORM_PUSH_ORDER,
                commands: vec![PaintCommand::PushTransform(affine)],
            });
            units.push(PaintUnit {
                z: context_end.get(&node).copied().unwrap_or(z),
                tree_index: exit,
                order: PaintUnit::pop_order(depth, PaintUnit::TRANSFORM_PUSH_ORDER),
                commands: vec![PaintCommand::PopTransform],
            });
        }
        // The CSS clip surrounds the filtered result.
        for (node, bounds, radius, color_matrix) in filters {
            let (z, enter, exit, depth) = key(node);
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
                tree_index: enter,
                order: PaintUnit::BLUR_PUSH_ORDER,
                commands: vec![PaintCommand::PushLayer(layer)],
            });
            units.push(PaintUnit {
                z: context_end.get(&node).copied().unwrap_or(z),
                tree_index: exit,
                order: PaintUnit::pop_order(depth, PaintUnit::BLUR_PUSH_ORDER),
                commands: vec![PaintCommand::PopLayer],
            });
        }
        // Outlines and scrollbars escape the overflow clip and remain inside
        // the CSS clip, transform and compositing group.
        for (node, command) in outlines {
            let (z, _enter, exit, depth) = key(node);
            units.push(PaintUnit {
                z: context_end.get(&node).copied().unwrap_or(z),
                tree_index: exit,
                order: PaintUnit::overlay_order(depth),
                commands: vec![command],
            });
        }
        for (node, scrollport, scroll_top, content_height, horizontal) in scrollbars {
            let commands = scrollbar_commands(scrollport, scroll_top, content_height, horizontal);
            if commands.is_empty() {
                continue;
            }
            let (z, _enter, exit, depth) = key(node);
            units.push(PaintUnit {
                z: context_end.get(&node).copied().unwrap_or(z),
                tree_index: exit,
                order: PaintUnit::overlay_order(depth),
                commands,
            });
        }
    }
}
