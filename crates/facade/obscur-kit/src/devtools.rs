#![doc = include_str!("../docs/api/devtools.doc/_module.md")]

mod shared_inspect_source;
pub use shared_inspect_source::SharedInspectSource;
mod diagnostic_event_sink;
pub use diagnostic_event_sink::DiagnosticEventSink;
mod runtime_event_publisher;
pub use runtime_event_publisher::RuntimeEventPublisher;

use obscur_devtools_protocol::message::{
    ControlAction, InjectedEvent, Inspectable, SnapshotPayload, StableSelector,
};
use obscur_devtools_server::bridge::SnapshotInspectSource;

use crate::geometry::Point;
use crate::identity::NodeId;
use crate::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use crate::perf::FrameStats;
use crate::runtime::orchestration::TreeInspection;
use crate::runtime::{PointerDispatch, UiRuntime};
use crate::view::view::{AUTOMATION_ID, TEST_ID, ViewValue};

#[doc = include_str!("../docs/api/devtools.doc/inspect_source_from.md")]
#[must_use]
pub fn inspect_source_from(runtime: &UiRuntime) -> SnapshotInspectSource {
    let snapshot = runtime.inspect_snapshot();
    let ids: Vec<u64> = snapshot.nodes.iter().map(|node| node.node.raw()).collect();
    let semantic = runtime.semantic_snapshot();

    let mut source = SnapshotInspectSource::new()
        .with_stable_nodes(stable_nodes(runtime))
        .with_snapshot(Inspectable::Tree, tree_payload(&snapshot, ids.clone()))
        .with_snapshot(Inspectable::Layout, layout_payload(&snapshot, ids.clone()))
        .with_snapshot(Inspectable::Styles, styles_payload(&snapshot, ids))
        .with_snapshot(Inspectable::Focus, focus_payload(runtime))
        .with_snapshot(Inspectable::Accessibility, accessibility_payload(&semantic))
        .with_semantic_nodes(
            semantic
                .iter()
                .filter(|(_, node)| !node.state().hidden)
                .map(|(id, node)| {
                    (
                        id.raw(),
                        format!("{:?}", node.role()).into(),
                        node.name().map(Into::into),
                    )
                }),
        )
        .with_snapshot(Inspectable::Scene, scene_payload(runtime))
        .with_snapshot(Inspectable::Animations, animations_payload(runtime))
        .with_snapshot(Inspectable::Resources, resources_payload(runtime))
        .with_snapshot(Inspectable::Caches, caches_payload(runtime));
    if let Some(trace) = runtime.command_trace() {
        let count = trace.len() as u64;
        let mut nodes = Vec::new();
        let mut lines = Vec::new();
        for entry in trace {
            nodes.extend(entry.target.map(NodeId::raw));
            lines.push(format!(
                "{:?} {:?} target={:?} command={} handled={} default_prevented={}",
                entry.origin,
                entry.trust,
                entry.target.map(NodeId::raw),
                entry.command.raw(),
                entry.handled,
                entry.default_prevented,
            ));
        }
        source.set_snapshot(
            Inspectable::EventFlow,
            SnapshotPayload::new(format!("{count} command dispatch(es)"), count, nodes)
                .with_detail(lines.join("\n")),
        );
    }
    source
}

fn stable_nodes(runtime: &UiRuntime) -> Vec<(StableSelector, u64)> {
    let mut result = Vec::new();
    for (id, node) in runtime.tree().iter() {
        let id = id.raw();
        result.push((StableSelector::Node(id), id));
        if let Some(component) = node.kind().component_id() {
            result.push((StableSelector::Component(component.raw()), id));
        }
        for property in node.props() {
            let ViewValue::Text(value) = property.value() else {
                continue;
            };
            let selector = match property.name() {
                TEST_ID => StableSelector::TestId(value.clone()),
                AUTOMATION_ID => StableSelector::AutomationId(value.clone()),
                _ => continue,
            };
            result.push((selector, id));
        }
    }
    result
}

/// The whole annotated tree as an agent-readable indented dump.
fn tree_payload(snapshot: &TreeInspection, ids: Vec<u64>) -> SnapshotPayload {
    let node_count = snapshot.len() as u64;
    SnapshotPayload::new(format!("{node_count} nodes"), node_count, ids)
        .with_detail(snapshot.to_string())
}

/// One line of absolute geometry per laid-out node.
fn layout_payload(snapshot: &TreeInspection, ids: Vec<u64>) -> SnapshotPayload {
    let layout_lines: Vec<String> = snapshot
        .nodes
        .iter()
        .filter_map(|node| {
            node.bounds.map(|b| {
                format!(
                    "#{} @{:.0},{:.0} {:.0}×{:.0}",
                    node.node.raw(),
                    b.origin.x,
                    b.origin.y,
                    b.size.width,
                    b.size.height
                )
            })
        })
        .collect();
    SnapshotPayload::new(
        format!("{} laid-out box(es)", layout_lines.len()),
        layout_lines.len() as u64,
        ids,
    )
    .with_detail(layout_lines.join("\n"))
}

/// One line of key computed styles per node that resolved any.
fn styles_payload(snapshot: &TreeInspection, ids: Vec<u64>) -> SnapshotPayload {
    let style_lines: Vec<String> = snapshot
        .nodes
        .iter()
        .filter(|node| !node.styles.is_empty())
        .map(|node| {
            let pairs: Vec<String> = node
                .styles
                .iter()
                .map(|(name, value)| format!("{name}={value}"))
                .collect();
            format!("#{} {}", node.node.raw(), pairs.join(" "))
        })
        .collect();
    SnapshotPayload::new(
        format!("{} styled node(s)", style_lines.len()),
        style_lines.len() as u64,
        ids,
    )
    .with_detail(style_lines.join("\n"))
}

/// The current focus and hover targets.
fn focus_payload(runtime: &UiRuntime) -> SnapshotPayload {
    SnapshotPayload::summary(format!(
        "focused: {} · hovered: {}",
        node_label(runtime.focused().map(NodeId::raw)),
        node_label(runtime.hovered().map(NodeId::raw)),
    ))
}

/// The semantic tree derived from the retained tree.
fn accessibility_payload(
    semantic: &obscur_accessibility::tree::SemanticSnapshot,
) -> SnapshotPayload {
    // The semantic tree the runtime already derives from the retained tree — the
    // same one a platform AX bridge consumes, so an inspector and a screen reader
    // see one truth rather than two.
    SnapshotPayload::new(
        format!("{} semantic node(s)", semantic.len()),
        semantic.len() as u64,
        Vec::new(),
    )
    .with_detail(
        semantic
            .iter()
            .map(|(node, semantic_node)| format!("#{} {semantic_node:?}", node.raw()))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

/// The composed display list this frame actually paints.
fn scene_payload(runtime: &UiRuntime) -> SnapshotPayload {
    let commands = runtime.snapshot();
    let command_count = commands.display_list().commands().len() as u64;
    SnapshotPayload::new(
        format!("{command_count} paint command(s)"),
        command_count,
        Vec::new(),
    )
    .with_detail(
        commands
            .display_list()
            .commands()
            .iter()
            .map(|command| format!("{command:?}"))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

/// How many motion timelines are running right now.
fn animations_payload(runtime: &UiRuntime) -> SnapshotPayload {
    let running = runtime.active_animation_count() as u64;
    SnapshotPayload::new(
        format!("{running} running timeline(s)"),
        running,
        Vec::new(),
    )
}

/// Everything requested this session, whatever its state.
fn resources_payload(runtime: &UiRuntime) -> SnapshotPayload {
    let requested = runtime.resource_count() as u64;
    SnapshotPayload::new(
        format!("{requested} requested resource(s)"),
        requested,
        Vec::new(),
    )
}

/// What is resolved and held, split by kind, with the image cache's budget counters.
fn caches_payload(runtime: &UiRuntime) -> SnapshotPayload {
    // A raster image and a parsed vector document are cached separately and cost
    // differently, so the two are counted apart rather than summed away. The image
    // cache is budgeted (§5.19), so its live bytes and hit/miss/eviction
    // counters are reported too: a count alone cannot answer why an image was
    // dropped, which is the question a budgeted cache exists to raise.
    let (images, vectors) = runtime.cached_resource_counts();
    let stats = runtime.image_cache_statistics();
    SnapshotPayload::new(
        format!("{images} image(s), {vectors} vector document(s)"),
        (images + vectors) as u64,
        Vec::new(),
    )
    .with_detail(format!(
        "image_cache: {images} ({bytes} bytes)\n\
         image_cache hits: {hits}\n\
         image_cache misses: {misses}\n\
         image_cache evictions: {evictions}\n\
         vector_cache: {vectors}",
        bytes = stats.bytes(),
        hits = stats.hits(),
        misses = stats.misses(),
        evictions = stats.evictions(),
    ))
}

/// Renders an optional node id as `#N` or `none`.
fn node_label(raw: Option<u64>) -> String {
    raw.map_or_else(|| "none".to_owned(), |id| format!("#{id}"))
}

#[doc = include_str!("../docs/api/devtools.doc/apply_injected.md")]
pub fn apply_injected(runtime: &mut UiRuntime, event: InjectedEvent) -> Option<PointerDispatch> {
    // Deliberately NOT `.as_trusted()` — see the note above.
    let pointer = |kind, x: i32, y: i32| {
        PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            kind,
            Point::new(x as f32, y as f32),
        )
    };
    match event {
        InjectedEvent::PointerDown { x, y } => {
            Some(runtime.handle_pointer(pointer(PointerEventKind::Down, x, y)))
        }
        InjectedEvent::PointerUp { x, y } => {
            Some(runtime.handle_pointer(pointer(PointerEventKind::Up, x, y)))
        }
        // Key injection needs a keycode→KeyEvent map; not wired yet.
        InjectedEvent::KeyDown { .. } | InjectedEvent::KeyUp { .. } => None,
    }
}

/// Applies one authorized [`ControlAction`] (drained from the endpoint) to
/// `runtime`. `SetInspectMode` toggles the in-window overlay; frame-scheduling
/// actions (pause/resume/step) are left to the host's own frame loop policy.
pub fn apply_control(runtime: &mut UiRuntime, action: ControlAction) {
    match action {
        ControlAction::SetInspectMode(on) => runtime.set_inspect_mode(on),
        ControlAction::Pause | ControlAction::Resume | ControlAction::StepFrame => {}
    }
}

#[doc = include_str!("../docs/api/devtools.doc/perf_payload.md")]
#[must_use]
pub fn perf_payload(stats: &FrameStats) -> SnapshotPayload {
    let ms = |d: std::time::Duration| d.as_secs_f32() * 1000.0;
    let fps = stats.cost_fps();
    let detail = format!(
        "fps ~{:.0}\ncpu {:.2}ms (build→layout→compose)\ngpu {:.2}ms (present)\nframe {:.2}ms\nworst {:.2}ms\nsamples {}",
        fps,
        ms(stats.avg_cpu()),
        ms(stats.avg_present()),
        ms(stats.avg_total()),
        ms(stats.max_total()),
        stats.len(),
    );
    // A saturating cast, not `as`: an `f32` fps is finite and small in practice,
    // but a degenerate frame time could make it NaN or huge, and a wrapped count
    // in a diagnostic is worse than a clamped one.
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "clamped to a sane fps range first, so the cast cannot wrap"
    )]
    let rounded = fps.round().clamp(0.0, 1_000_000.0) as u64;
    SnapshotPayload::new(
        format!("~{fps:.0} fps · {:.2}ms/frame", ms(stats.avg_total())),
        rounded,
        Vec::new(),
    )
    .with_detail(detail)
}

/// Builds the full live inspect source a windowed host publishes each frame:
/// [`inspect_source_from`] (tree/layout/styles/focus) plus the
/// [`Inspectable::Rendering`] frame-perf snapshot from `stats`. One call so a
/// host does not re-assemble the two every frame.
#[must_use]
pub fn live_source(runtime: &UiRuntime, stats: &FrameStats) -> SnapshotInspectSource {
    inspect_source_from(runtime).with_snapshot(Inspectable::Rendering, perf_payload(stats))
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_devtools_server::bridge::InspectSource;
    use std::time::Duration;

    #[test]
    fn perf_payload_reports_fps_and_timing_breakdown() {
        let mut stats = FrameStats::default();
        // Two 8ms frames → ~125 fps sustainable, 4ms cpu + 4ms gpu average.
        stats.record(Duration::from_millis(4), Duration::from_millis(4));
        stats.record(Duration::from_millis(4), Duration::from_millis(4));
        let payload = perf_payload(&stats);
        assert!(
            payload.description.contains("125 fps"),
            "{}",
            payload.description
        );
        assert!(payload.detail.contains("gpu 4.00ms"), "{}", payload.detail);
        assert!(payload.detail.contains("cpu 4.00ms"), "{}", payload.detail);
        assert_eq!(payload.entity_count, 125);
    }

    #[test]
    fn shared_source_serves_the_latest_set_snapshot() {
        let shared = SharedInspectSource::new();
        // Before any set: the empty snapshot's typed default (never a panic).
        assert!(shared.snapshot(Inspectable::Rendering).entity_count == 0);

        let mut stats = FrameStats::default();
        stats.record(Duration::from_millis(5), Duration::from_millis(5));
        shared.set(
            SnapshotInspectSource::new()
                .with_snapshot(Inspectable::Rendering, perf_payload(&stats)),
        );
        // A clone (as handed to a serving thread) sees the update through the Arc.
        let served = shared.clone();
        assert!(
            served
                .snapshot(Inspectable::Rendering)
                .description
                .contains("fps")
        );
        assert_eq!(served.snapshot(Inspectable::Rendering).entity_count, 100);
    }
}
