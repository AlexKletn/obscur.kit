#![doc = include_str!("mod.doc/_module.md")]
mod ui_runtime;

pub use ui_runtime::UiRuntime;

mod image_cache;

pub use image_cache::ImageCache;

mod resource_bridge;

pub(crate) use resource_bridge::{ResourceBridge, ResourceReporting};

mod resource_outcome;

pub(crate) use resource_outcome::ResourceOutcome;

mod handler_registry;

pub(crate) use handler_registry::HandlerRegistry;

mod hit_testing;

pub(crate) use hit_testing::HitTesting;

mod tree_inspector;

pub(crate) use tree_inspector::TreeInspector;

mod frame_phase_clock;

pub(crate) use frame_phase_clock::FramePhaseClock;

mod component_render_pass;

pub(crate) use component_render_pass::ComponentRenderPass;

mod desired_view;

pub(crate) use desired_view::DesiredView;

mod reconcile_phase;

pub(crate) use reconcile_phase::ReconcilePhase;

mod arrange_phase;

pub(crate) use arrange_phase::ArrangePhase;

mod style_phase;

pub(crate) use style_phase::StylePhase;

mod compose_phase;

pub(crate) use compose_phase::ComposePhase;

mod ancestor_chain;

pub(crate) use ancestor_chain::AncestorChain;

mod command_dispatch;

pub(crate) use command_dispatch::{CommandDispatch, CommandDispatchOutcome};

mod layout_inputs;

pub(crate) use layout_inputs::LayoutInputs;

mod style_delta;

pub(crate) use style_delta::StyleDelta;

mod layout_classifier;

pub(crate) use layout_classifier::{LayoutClassifier, LayoutPhase};

mod style_key;

pub(crate) use style_key::StyleKey;

mod restyle_phase;

pub(crate) use restyle_phase::RestylePhase;

mod editable_field;

pub(crate) use editable_field::EditableField;

mod caret_scroll;

pub(crate) use caret_scroll::CaretScroll;

mod key_edit_resolution;

pub(crate) use key_edit_resolution::{KeyEdit, KeyEditResolver};

mod text_pointer_selection;

pub(crate) use text_pointer_selection::{TextPointerSelection, TextPress};

mod controlled_reseed;

pub(crate) use controlled_reseed::{ControlledReseed, ReseedDecision};

mod compose_only_edit;

pub(crate) use compose_only_edit::ComposeOnlyEdit;

mod scroll_targets;

pub(crate) use scroll_targets::ScrollTargets;

mod pointer_landing;

pub(crate) use pointer_landing::PointerLandingResolver;

mod structural_states;

pub(crate) use structural_states::StructuralStates;

mod pseudo_state_projection;

pub(crate) use pseudo_state_projection::PseudoStateProjection;

mod gesture_arming;

pub(crate) use gesture_arming::GestureArming;

mod hover_update;

pub(crate) use hover_update::HoverUpdate;

mod pointer_release;

pub(crate) use pointer_release::PointerRelease;

mod pointer_sessions;

#[cfg(test)]
pub(crate) use pointer_sessions::MULTI_CLICK_WINDOW_MICROS;
pub(crate) use pointer_sessions::{ActiveHold, ActivePan, PointerCompletion, PointerSessions};

mod app_commands;

pub use app_commands::AppCommands;

mod command_handler;

pub use command_handler::CommandHandler;

mod command_target;

pub use command_target::CommandTarget;

mod event_listener;

pub(crate) use event_listener::EventListener;

mod into_command_handler;

pub use into_command_handler::{IntoCommandHandler, WithEvent, WithoutEvent};

mod text_edit_session;

pub(crate) use text_edit_session::TextEditSession;

mod pointer_dispatch;

pub use pointer_dispatch::PointerDispatch;

mod key_dispatch;

pub use key_dispatch::KeyDispatch;

mod semantic_builder;

pub use semantic_builder::SemanticTreeBuilder;

mod input_event;
pub use input_event::InputEvent;
mod input_dispatch;
pub use input_dispatch::InputDispatch;
mod input_queue;
pub(crate) use input_queue::InputQueue;
mod surface_geometry;
pub(crate) use surface_geometry::SurfaceGeometry;
mod scroll_offsets;
pub(crate) use scroll_offsets::{FoldInputs, ScrollOffsets};
mod motion_driver;
pub(crate) use motion_driver::MotionDriver;
mod virtual_lists;
pub(crate) use virtual_lists::VirtualLists;
mod cascade_sheets;
pub(crate) use cascade_sheets::{AuthorSheets, CascadeSheets};
mod accessibility_publisher;
pub(crate) use accessibility_publisher::AccessibilityPublisher;
mod frame_plan;

pub(crate) use frame_plan::FramePlan;

mod frame_budget;

pub use frame_budget::FrameBudget;

mod frame_telemetry;

pub(crate) use frame_telemetry::{FramePhase, FrameTelemetry};

mod background_work;

pub(crate) use background_work::{BackgroundServiced, BackgroundWork};

mod runtime_config;
mod runtime_event;
mod runtime_handle;
mod runtime_state;
