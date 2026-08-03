#![doc = include_str!("construction.doc/_module.md")]

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
    ArrangeCache, ComputedStyles, DragCoordinator, LayoutBoxes, LayoutCoordinator, MeasureCache,
    PseudoStates, SceneCoordinator, StyleCoordinator, StyleInputs, StyleMatchingCoordinator,
};
use crate::runtime::{
    AccessibilityPublisher, BackgroundWork, CascadeSheets, FrameTelemetry, HandlerRegistry,
    HitTesting, InputQueue, MotionDriver, PointerEventBindings, PointerSessions, ResourceBridge,
    ScrollOffsets, SurfaceGeometry, TreeInspector, VirtualLists,
};
use crate::scheduler::FrameScheduler;
use crate::transaction::ReconcileEnactor;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("construction.doc/new.md")]
    #[must_use]
    pub fn new(id: RuntimeId, viewport: Size) -> Self {
        Self {
            id,
            tree: RetainedTree::new(),
            scene: Scene::new(),
            reconciler: Reconciler::new(),
            enactor: ReconcileEnactor::new(),
            matcher: StyleMatchingCoordinator::new(),
            style: StyleCoordinator::new(),
            layout: LayoutCoordinator::new(),
            measure_cache: MeasureCache::new(),
            arrange_cache: ArrangeCache::new(),
            scene_composer: SceneCoordinator::new(),
            // `und`, not `en`: an unset locale is unknown, not English. Nothing
            // changes until a host says otherwise (spec §7.2).
            locale: LanguageTag::root(),
            scheduler: FrameScheduler::new(),
            pseudo: PseudoStates::new(),
            boxes: LayoutBoxes::new(),
            inputs: StyleInputs::new(),
            computed: ComputedStyles::new(),
            generated_content: HashMap::new(),
            overlaid: ComputedStyles::new(),
            last_style: None,
            hits: HitTesting::new(),
            hover: HoverChain::new(),
            last_layout_inputs: None,
            inspector: TreeInspector::new(),
            focused: None,
            focus_visible: false,
            pointers: PointerSessions::new(),
            pointer_bindings: PointerEventBindings::none(),
            environment: EnvironmentValues::none(),
            routing: HandlerRegistry::new(),
            editing: None,
            modifiers: Modifiers::new(),
            input: InputQueue::new(),
            drag: DragCoordinator::new(),
            drag_outcome: None,
            staged_plan: None,
            next_frame: FrameId::FIRST,
            surface: SurfaceGeometry::new(viewport),
            scroll: ScrollOffsets::new(),
            virtual_lists: VirtualLists::new(),
            motion: MotionDriver::new(),
            telemetry: FrameTelemetry::new(),
            diagnostics: Box::new(NullDiagnosticSink),
            background: BackgroundWork::new(),
            root_view: None,
            components: ComponentStore::new(),
            sheets: CascadeSheets::new(),
            accessibility: AccessibilityPublisher::new(),
            resources: ResourceBridge::new(),
        }
    }
}
