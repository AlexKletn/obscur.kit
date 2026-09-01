#![doc = include_str!("compose_phase.doc/_module.md")]

use obscur_identity::{FastMap, NodeId};
use obscur_scene::command::{DisplayList, PaintCommand};
use obscur_scene::scene::Scene;
use obscur_tree::tree::RetainedTree;

use super::{
    FramePhase, FramePhaseClock, FrameTelemetry, MotionDriver, ResourceBridge, ScrollOffsets,
    ScrollTargets, SurfaceGeometry, TreeInspector,
};
use crate::frame::FrameContext;
use crate::orchestration::{
    ComputedStyles, EditingView, GeneratedContent, LayoutBoxes, SceneCoordinator,
};

#[doc = include_str!("compose_phase.doc/ComposePhase.md")]
pub(crate) struct ComposePhase<'a> {
    #[doc = include_str!("compose_phase.doc/scene.md")]
    scene: &'a mut Scene,
    #[doc = include_str!("compose_phase.doc/scene_composer.md")]
    scene_composer: &'a SceneCoordinator,
    #[doc = include_str!("compose_phase.doc/tree.md")]
    tree: &'a RetainedTree,
    #[doc = include_str!("compose_phase.doc/boxes.md")]
    boxes: &'a LayoutBoxes,
    #[doc = include_str!("compose_phase.doc/resources.md")]
    resources: &'a ResourceBridge,
    #[doc = include_str!("compose_phase.doc/scroll.md")]
    scroll: &'a ScrollOffsets,
    #[doc = include_str!("compose_phase.doc/generated_content.md")]
    generated_content: &'a FastMap<NodeId, GeneratedContent>,
    #[doc = include_str!("compose_phase.doc/canvases.md")]
    canvases: &'a FastMap<NodeId, Vec<PaintCommand>>,
    #[doc = include_str!("compose_phase.doc/motion.md")]
    motion: &'a MotionDriver,
    #[doc = include_str!("compose_phase.doc/inspector.md")]
    inspector: &'a TreeInspector,
    #[doc = include_str!("compose_phase.doc/surface.md")]
    surface: &'a SurfaceGeometry,
    #[doc = include_str!("compose_phase.doc/telemetry.md")]
    telemetry: &'a mut FrameTelemetry,
}

impl<'a> ComposePhase<'a> {
    #[doc = include_str!("compose_phase.doc/new.md")]
    // A borrow bundle's constructor is exactly as wide as the state the phase
    // touches; folding these into a struct would only move the same eleven names.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        scene: &'a mut Scene,
        scene_composer: &'a SceneCoordinator,
        tree: &'a RetainedTree,
        boxes: &'a LayoutBoxes,
        resources: &'a ResourceBridge,
        scroll: &'a ScrollOffsets,
        generated_content: &'a FastMap<NodeId, GeneratedContent>,
        canvases: &'a FastMap<NodeId, Vec<PaintCommand>>,
        motion: &'a MotionDriver,
        inspector: &'a TreeInspector,
        surface: &'a SurfaceGeometry,
        telemetry: &'a mut FrameTelemetry,
    ) -> Self {
        Self {
            scene,
            scene_composer,
            tree,
            boxes,
            resources,
            scroll,
            generated_content,
            canvases,
            motion,
            inspector,
            surface,
            telemetry,
        }
    }

    #[doc = include_str!("compose_phase.doc/run.md")]
    pub(crate) fn run(
        &mut self,
        ctx: &FrameContext,
        computed: &mut ComputedStyles,
        editing: Option<EditingView<'_>>,
        hovered: Option<NodeId>,
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
        let undo = self.motion.apply_presentation(computed);
        let display_list = self.scene_composer.compose_with_images_scrolled(
            self.tree,
            computed,
            self.boxes,
            editing,
            self.resources.images(),
            self.resources.vectors(),
            &scroll,
            self.generated_content,
            self.canvases,
        );
        // Before anything else can read them, and before the early returns
        // below: what `computed_style` answers must be what the cascade said.
        MotionDriver::restore_presentation(computed, undo);
        // Inspect overlay (spec §7.6): when inspect mode is on, append a
        // highlight over the hovered node's border box on top of the scene.
        // A no-op (byte-identical) when inspect mode is off or nothing is
        // hovered. Appended before the device-scale root so the highlight is
        // in the same logical space as the layout boxes it traces.
        let display_list = self
            .inspector
            .apply_overlay(self.boxes, hovered, display_list);
        phase_clock.finish();
        // Telemetry: mark the compose phase and count paint commands (§7.9).
        let count = display_list.len() as u64;
        self.telemetry
            .record_phase(FramePhase::Compose, count, ctx.id());
        // Bracket the whole scene in a root device-scale transform so layout
        // (in logical px) rasterizes at physical resolution. At scale `1.0` this
        // is a no-op and the list is byte-identical (see `scaled_root`).
        self.scene
            .set_root(Self::scaled_root(self.surface, display_list));
    }

    #[doc = include_str!("compose_phase.doc/scaled_root.md")]
    fn scaled_root(surface: &SurfaceGeometry, display_list: DisplayList) -> DisplayList {
        let Some(root) = surface.root_transform() else {
            return display_list;
        };
        // The list is owned and about to be dropped, so its commands are MOVED
        // into the wrapper. `extend_from_slice` deep-cloned every command in the
        // frame — every path, every glyph run, every image handle — on a machine
        // whose only crime was having a HiDPI display, since that is exactly when
        // `root_transform` is `Some`.
        let inner = display_list.into_commands();
        let mut commands = Vec::with_capacity(inner.len() + 2);
        commands.push(PaintCommand::PushTransform(root));
        commands.extend(inner);
        commands.push(PaintCommand::PopTransform);
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
        let wrapped = ComposePhase::scaled_root(&surface, source);

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
        let out = ComposePhase::scaled_root(&surface, source);
        assert_eq!(out.len(), 3);
        assert!(core::ptr::eq(segments_ptr(&out, 0), expected));
    }
}
