#![doc = include_str!("../../../docs/api/runtime/ui_runtime/construction.doc/_module.md")]

use std::collections::HashMap;

use obscur_diagnostics::NullDiagnosticSink;
use obscur_geometry::Size;
use obscur_identity::RuntimeId;
use obscur_interaction::command::Modifiers;
use obscur_interaction::hover::HoverChain;
use obscur_localization::locale::LanguageTag;
use obscur_scene::scene::Scene;
use obscur_style::property::EnvironmentValues;
use obscur_tree::reconciliation::Reconciler;
use obscur_tree::tree::RetainedTree;

use crate::component::ComponentStore;
use crate::frame::FrameId;
use crate::orchestration::{
    ArrangeCache, ComputedStyles, DragCoordinator, InlineStyles, LayoutBoxes, LayoutCoordinator,
    MeasureCache, PseudoStates, SceneCoordinator, StyleCoordinator, StyleInputs,
    StyleMatchingCoordinator,
};
use crate::runtime::{
    AccessibilityPublisher, BackgroundWork, CanvasPainters, CascadeSheets, FrameTelemetry,
    HandlerRegistry, HitTesting, InputQueue, MotionDriver, PointerEventBindings, PointerSessions,
    ResourceBridge, ScrollOffsets, SurfaceGeometry, TreeInspector, VirtualLists,
};
use crate::scheduler::FrameScheduler;
use crate::transaction::ReconcileEnactor;

use super::UiRuntime;

impl UiRuntime {
    /// Gives the host access to this runtime’s lifecycle reports and notifications.
    pub fn lifecycle_mut(&mut self) -> &mut crate::lifecycle::RuntimeLifecycle {
        &mut self.lifecycle
    }

    /// Drops this runtime and its owned handles before returning the release
    /// notification. The host manages any separately owned windows, shared resources
    /// and tasks.
    #[must_use]
    pub fn release(mut self) -> crate::lifecycle::ReleaseResult {
        let id = self.id;
        let events = self.lifecycle.take_events();
        drop(self);
        crate::lifecycle::ReleaseResult::complete(id, events)
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/construction.doc/new.md")]
    #[must_use]
    pub fn new(id: RuntimeId, viewport: Size) -> Self {
        Self {
            id,
            tree: RetainedTree::new(),
            portal_hosts: None,
            scene: Scene::new(),
            reconciler: Reconciler::new(),
            enactor: ReconcileEnactor::new(),
            matcher: StyleMatchingCoordinator::new(),
            style: StyleCoordinator::new(),
            layout: LayoutCoordinator::new(),
            layout_styles: Default::default(),
            generated_boxes: None,
            container_queries: Default::default(),
            container_query_rounds: 0,
            measure_cache: MeasureCache::new(),
            arrange_cache: ArrangeCache::new(),
            scene_composer: SceneCoordinator::new(),
            // `und`, not `en`: an unset locale is unknown, not English. Nothing
            // changes until a host says otherwise (spec §7.2).
            locale: LanguageTag::root(),
            scheduler: FrameScheduler::new(),
            view_animation_requested: false,
            timeline_waiting_for_tick: false,
            pseudo: PseudoStates::new(),
            boxes: LayoutBoxes::default(),
            inputs: StyleInputs::new(),
            computed: ComputedStyles::default(),
            generated_content: HashMap::default(),
            overlaid: ComputedStyles::default(),
            last_style: None,
            hits: HitTesting::new(),
            hover: HoverChain::new(),
            hover_pointer: None,
            hover_correction_pending: false,
            last_layout_inputs: None,
            inspector: TreeInspector::new(),
            canvas_painters: CanvasPainters::new(),
            inline_styles: InlineStyles::new(),
            logical_time: std::time::Duration::ZERO,
            slider_drags: HashMap::new(),
            focused: None,
            focus_visible: false,
            spatial_navigation: false,
            pointers: PointerSessions::new(),
            scrollbar_drags: HashMap::new(),
            touch_gestures: Default::default(),
            system_back: None,
            scroll_gesture: None,
            scroll_gesture_blocked: false,
            native_touchpad_gesture: None,
            pointer_bindings: PointerEventBindings::none(),
            environment: EnvironmentValues::none(),
            media_preferences: obscur_style::matching::MediaPreferences::default(),
            detected_media_input: None,
            media_input_override: None,
            routing: HandlerRegistry::new(),
            editing: None,
            modifiers: Modifiers::new(),
            input: InputQueue::new(),
            drag: DragCoordinator::new(),
            drag_scroll_point: None,
            selection_scroll_point: None,
            drag_outcome: None,
            staged_plan: None,
            next_frame: FrameId::FIRST,
            surface: SurfaceGeometry::new(viewport),
            scroll: ScrollOffsets::new(),
            scroll_snaps: Default::default(),
            tracked_scroll_anchors: Default::default(),
            virtual_lists: VirtualLists::new(),
            motion: MotionDriver::new(),
            shared_elements: Vec::new(),
            animation_progress: Vec::new(),
            telemetry: FrameTelemetry::new(),
            diagnostics: Box::new(NullDiagnosticSink),
            command_trace: None,
            background: BackgroundWork::new(),
            geometry_observers: Vec::new(),
            visible_content_observer: None,
            lifecycle: crate::lifecycle::RuntimeLifecycle::default(),
            background_budget: crate::runtime::BackgroundBudget::default(),
            root_view: None,
            components: ComponentStore::new(),
            sheets: CascadeSheets::new(),
            accessibility: AccessibilityPublisher::new(),
            resources: ResourceBridge::new(),
        }
    }
}
