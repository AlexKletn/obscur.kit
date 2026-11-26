//! The frame's paint phase: [`ComposePhase`].

use obscur_diagnostics::DiagnosticSink;
use obscur_geometry::AffineTransform;
use obscur_identity::{FastMap, NodeId};
use obscur_scene::clip::Clip;
use obscur_scene::command::{DisplayList, PaintCommand};
use obscur_scene::scene::Scene;
use obscur_tree::tree::RetainedTree;

use super::{
    FramePhase, FramePhaseClock, FrameTelemetry, HitTesting, MotionDriver, ResourceBridge,
    ScrollOffsets, ScrollTargets, SurfaceGeometry, TreeInspector,
};
use crate::frame::FrameContext;
use crate::orchestration::{
    ComputedStyles, EditingView, GeneratedBoxes, GeneratedContent, LayoutBoxes, SceneCoordinator,
};

#[doc = include_str!("../../docs/api/runtime/compose_phase.doc/ComposePhase.md")]
pub(crate) struct ComposePhase<'a> {
    /// The scene this phase installs its display list into.
    scene: &'a mut Scene,
    /// Composes the paint-ordered display list.
    scene_composer: &'a SceneCoordinator,
    /// The retained tree being painted.
    tree: &'a RetainedTree,
    /// This frame's arranged boxes, in unscrolled layout space.
    boxes: &'a LayoutBoxes,
    hits: &'a mut HitTesting,
    /// The decoded-image and parsed-vector caches phase 3c filled.
    resources: &'a ResourceBridge,
    /// Every scroll container's live vertical offset (CSS Overflow 3).
    scroll: &'a ScrollOffsets,
    /// The §4.16 generated / synthetic decoration resolved by the style phase.
    generated_content: &'a FastMap<NodeId, GeneratedContent>,
    generated_boxes: Option<&'a GeneratedBoxes>,
    #[doc = include_str!("../../docs/api/runtime/compose_phase.doc/canvases.md")]
    canvases: &'a FastMap<NodeId, Vec<PaintCommand>>,
    /// The active timelines, read for their group-compositing presentation values.
    motion: &'a MotionDriver,
    /// The §7.6 DevTools projection, which owns the inspect highlight.
    inspector: &'a TreeInspector,
    /// The surface's root transform: the device scale and the content offset.
    surface: &'a SurfaceGeometry,
    /// Where the compose phase's span and command count land (spec §7.9).
    telemetry: &'a mut FrameTelemetry,
    diagnostics: &'a dyn DiagnosticSink,
}

impl<'a> ComposePhase<'a> {
    /// Bundles the state one compose phase reads and writes.
    // A borrow bundle's constructor is exactly as wide as the state the phase
    // touches; folding these into a struct would only move the same eleven names.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        scene: &'a mut Scene,
        scene_composer: &'a SceneCoordinator,
        tree: &'a RetainedTree,
        boxes: &'a LayoutBoxes,
        hits: &'a mut HitTesting,
        resources: &'a ResourceBridge,
        scroll: &'a ScrollOffsets,
        generated_content: &'a FastMap<NodeId, GeneratedContent>,
        generated_boxes: Option<&'a GeneratedBoxes>,
        canvases: &'a FastMap<NodeId, Vec<PaintCommand>>,
        motion: &'a MotionDriver,
        inspector: &'a TreeInspector,
        surface: &'a SurfaceGeometry,
        telemetry: &'a mut FrameTelemetry,
        diagnostics: &'a dyn DiagnosticSink,
    ) -> Self {
        Self {
            scene,
            scene_composer,
            tree,
            boxes,
            hits,
            resources,
            scroll,
            generated_content,
            generated_boxes,
            canvases,
            motion,
            inspector,
            surface,
            telemetry,
            diagnostics,
        }
    }

    #[doc = include_str!("../../docs/api/runtime/compose_phase.doc/run.md")]
    pub(crate) fn run(
        &mut self,
        ctx: &FrameContext,
        computed: &mut ComputedStyles,
        editing: Option<EditingView<'_>>,
        hovered: Option<NodeId>,
        presentation_index_current: bool,
        phase_clock: &FramePhaseClock,
    ) {
        // Images this frame references were already scored for visibility and
        // resolved into the decoded caches in phase 3c (before layout, so a
        // replaced element could be sized from its content and the §5.19 cache
        // is touched once per frame); the coordinator below simply paints the
        // ready ones.
        //
        // Resolve each scroll container's live scroll offset + content height
        // (CSS Overflow 3 §scroll-container) from the retained boxes, so the
        // coordinator paints the scroll offset + scrollbar. Empty when no
        // container overflows → the compose is byte-identical to before scroll
        // containers existed. `paint` finds the containers from the styles handed
        // to it rather than from the ones the bundle holds, so both are this
        // frame's own.
        let scroll =
            ScrollTargets::new(self.tree, computed, self.boxes, self.scroll).paint(computed);
        // Motion's group-compositing presentation values (`opacity`/`transform`)
        // reach paint and must never be observable through `computed_style`
        // (spec §4.17: motion is a separate subsystem). They are written into
        // the real map for the length of the compose and taken out again — the
        // throwaway copy this replaced was a deep clone of every node's style,
        // paid on every frame of every animation to carry a handful of samples.
        let undo = self.motion.apply_presentation(computed, self.boxes);
        if !presentation_index_current {
            self.hits
                .update_presentation(self.tree, computed, self.boxes, &undo);
        }
        let (tree, boxes, placeholders) = if let Some(generated) = self.generated_boxes {
            computed.extend(
                generated
                    .styles
                    .iter()
                    .map(|(&node, style)| (node, style.clone())),
            );
            let placeholders = self
                .generated_content
                .iter()
                .filter_map(|(&node, content)| {
                    content.placeholder.as_ref().map(|placeholder| {
                        (
                            node,
                            GeneratedContent {
                                placeholder: Some(placeholder.clone()),
                                ..GeneratedContent::default()
                            },
                        )
                    })
                })
                .collect();
            (&generated.tree, &generated.boxes, Some(placeholders))
        } else {
            (self.tree, self.boxes, None)
        };
        let viewport = self.surface.viewport();
        let window_clip = super::viewport_shape::clip(
            viewport,
            self.surface.device_scale(),
            super::viewport_shape::radii(self.tree, self.boxes, computed, viewport),
        );
        let display_list = self.scene_composer.compose_at_surface(
            tree,
            computed,
            boxes,
            editing,
            self.resources.images(),
            self.resources.vectors(),
            &scroll,
            placeholders.as_ref().unwrap_or(self.generated_content),
            self.canvases,
            self.surface
                .root_transform()
                .unwrap_or(AffineTransform::IDENTITY),
            Some(self.surface.visible_content_rect()),
        );
        // Before anything else can read them, and before the early returns
        // below: what `computed_style` answers must be what the cascade said.
        if let Some(generated) = self.generated_boxes {
            for node in generated.styles.keys() {
                computed.remove(node);
            }
        }
        MotionDriver::restore_presentation(computed, undo);
        // Inspect overlay (spec §7.6): when inspect mode is on, append a
        // highlight over the hovered node's border box on top of the scene.
        // A no-op (byte-identical) when inspect mode is off or nothing is
        // hovered. Appended before the device-scale root so the highlight is
        // in the same logical space as the layout boxes it traces.
        let display_list = self
            .inspector
            .apply_overlay(self.boxes, hovered, display_list);
        // Telemetry: mark the compose phase and count paint commands (§7.9).
        let count = display_list.len() as u64;
        self.telemetry
            .record_phase(FramePhase::Compose, count, ctx.id(), self.diagnostics);
        self.scene
            .set_root(Self::scaled_root(self.surface, display_list, window_clip));
        phase_clock.mark_compose();
    }

    #[doc = include_str!("../../docs/api/runtime/compose_phase.doc/scaled_root.md")]
    fn scaled_root(
        surface: &SurfaceGeometry,
        display_list: DisplayList,
        window_clip: Option<Clip>,
    ) -> DisplayList {
        let transform = surface.root_transform();
        if transform.is_none() && window_clip.is_none() {
            return display_list;
        }
        let inner = display_list.into_commands();
        let mut commands = Vec::with_capacity(inner.len() + 4);
        let clipped = window_clip.is_some();
        if let Some(clip) = window_clip {
            commands.push(PaintCommand::PushClip(clip));
        }
        if let Some(transform) = transform {
            commands.push(PaintCommand::PushTransform(transform));
        }
        commands.extend(inner);
        if transform.is_some() {
            commands.push(PaintCommand::PopTransform);
        }
        if clipped {
            commands.push(PaintCommand::PopClip);
        }
        DisplayList::from_commands(commands)
    }
}

#[cfg(test)]
mod tests {
    use obscur_geometry::{Point, Rect, Size};
    use obscur_scene::color::Color;
    use obscur_scene::command::{DisplayList, PaintCommand};
    use obscur_scene::paint::Paint;
    use obscur_scene::path::Path;

    use super::{ComposePhase, SurfaceGeometry};

    fn long_path() -> PaintCommand {
        // A path with a heap buffer big enough that its identity is unambiguous:
        // the thing `extend_from_slice` used to duplicate is not the command
        // struct, it is what the command points at.
        let mut path = Path::new();
        path.move_to(Point::new(0.0, 0.0));
        for step in 1..64u8 {
            path.line_to(Point::new(f32::from(step), f32::from(step)));
        }
        PaintCommand::DrawPath {
            path,
            paint: Paint::solid_fill(Color::WHITE),
        }
    }

    fn segments_ptr(list: &DisplayList, index: usize) -> *const obscur_scene::path::PathSegment {
        match &list.commands()[index] {
            PaintCommand::DrawPath { path, .. } => path.segments().as_ptr(),
            other => panic!("expected a path command, got {other:?}"),
        }
    }

    #[test]
    fn the_device_scale_wrapper_moves_the_frames_commands_instead_of_copying_them() {
        // The cost this removes runs on every frame on any HiDPI display, because
        // `root_transform` is `Some` exactly when the surface is not 1:1.
        // `extend_from_slice` deep-cloned the whole frame — every path, glyph run
        // and image handle — out of a list that was owned by value and dropped on
        // the next line.
        //
        // Equality cannot see the difference, so the assertion is pointer identity
        // of a command's own heap buffer: a clone would have had to allocate a new
        // one.
        let mut surface = SurfaceGeometry::new(Size::new(100.0, 100.0));
        surface.set_device_scale(2.0);
        assert!(
            surface.root_transform().is_some(),
            "the wrapper only runs when the surface is not 1:1"
        );

        let source = DisplayList::from_commands(vec![long_path()]);
        let expected = segments_ptr(&source, 0);
        let wrapped = ComposePhase::scaled_root(&surface, source, None);

        assert!(matches!(
            wrapped.commands().first(),
            Some(PaintCommand::PushTransform(_))
        ));
        assert!(matches!(
            wrapped.commands().last(),
            Some(PaintCommand::PopTransform)
        ));
        assert!(wrapped.is_balanced());
        assert!(
            core::ptr::eq(segments_ptr(&wrapped, 1), expected),
            "the wrapped list must hold the frame's own commands, not copies of them"
        );
    }

    #[test]
    fn an_unscaled_surface_hands_the_list_straight_back() {
        // The regression guard the doc states: at scale 1.0 and offset (0, 0) the
        // output is the input, brackets and all — now literally the same buffer.
        let surface = SurfaceGeometry::new(Size::new(100.0, 100.0));
        let source = DisplayList::from_commands(vec![
            long_path(),
            PaintCommand::PushClip(obscur_scene::clip::Clip::rect(Rect::default())),
            PaintCommand::PopClip,
        ]);
        let expected = segments_ptr(&source, 0);
        let out = ComposePhase::scaled_root(&surface, source, None);
        assert_eq!(out.len(), 3);
        assert!(core::ptr::eq(segments_ptr(&out, 0), expected));
    }
}
