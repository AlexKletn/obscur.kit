#![doc = include_str!("compose_phase.doc/_module.md")]

use std::collections::HashMap;

use obscur_identity::NodeId;
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
    generated_content: &'a HashMap<NodeId, GeneratedContent>,
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
        generated_content: &'a HashMap<NodeId, GeneratedContent>,
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
        computed: &ComputedStyles,
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
        // Layer motion's group-compositing presentation values
        // (`opacity`/`transform`) onto a throwaway copy of the computed styles
        // for compose ONLY — they reach paint but must never be observable
        // through `computed_style` (spec §4.17: motion is a separate
        // subsystem). `None` means no such motion is active, so compose reads
        // the computed styles untouched.
        let presentation = self.motion.presentation_styles(computed);
        let paint_styles = presentation.as_ref().unwrap_or(computed);
        let display_list = self.scene_composer.compose_with_images_scrolled(
            self.tree,
            paint_styles,
            self.boxes,
            editing,
            self.resources.images(),
            self.resources.vectors(),
            &scroll,
            self.generated_content,
        );
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
        self.scene.set_root(self.scaled_root(display_list));
    }

    #[doc = include_str!("compose_phase.doc/scaled_root.md")]
    fn scaled_root(&self, display_list: DisplayList) -> DisplayList {
        let Some(root) = self.surface.root_transform() else {
            return display_list;
        };
        let inner = display_list.commands();
        let mut commands = Vec::with_capacity(inner.len() + 2);
        commands.push(PaintCommand::PushTransform(root));
        commands.extend_from_slice(inner);
        commands.push(PaintCommand::PopTransform);
        DisplayList::from_commands(commands)
    }
}
