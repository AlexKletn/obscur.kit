#![doc = include_str!("devtools.doc/_module.md")]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use obscur_devtools_protocol::message::{
    ControlAction, EventKind, EventSeverity, InjectedEvent, Inspectable, SnapshotPayload, Topic,
};
use obscur_devtools_server::bridge::{InspectSource, SnapshotInspectSource};
use obscur_devtools_server::transport::EventQueue;

use crate::diagnostics::{Diagnostic, DiagnosticSeverity, DiagnosticSink};
use crate::geometry::Point;
use crate::identity::{NodeId, VersionStamp};
use crate::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use crate::perf::FrameStats;
use crate::runtime::{PointerDispatch, UiRuntime};

#[doc = include_str!("devtools.doc/inspect_source_from.md")]
#[must_use]
pub fn inspect_source_from(runtime: &UiRuntime) -> SnapshotInspectSource {
    let snapshot = runtime.inspect_snapshot();
    let node_count = snapshot.len() as u64;
    let ids: Vec<u64> = snapshot.nodes.iter().map(|node| node.node.raw()).collect();

    // Tree: the whole annotated tree as an agent-readable indented dump.
    let tree = SnapshotPayload::new(format!("{node_count} nodes"), node_count, ids.clone())
        .with_detail(snapshot.to_string());

    // Layout: one line of absolute geometry per laid-out node.
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
    let layout = SnapshotPayload::new(
        format!("{} laid-out box(es)", layout_lines.len()),
        layout_lines.len() as u64,
        ids.clone(),
    )
    .with_detail(layout_lines.join("\n"));

    // Styles: one line of key computed styles per node that resolved any.
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
    let styles = SnapshotPayload::new(
        format!("{} styled node(s)", style_lines.len()),
        style_lines.len() as u64,
        ids,
    )
    .with_detail(style_lines.join("\n"));

    // Focus: the current focus + hover targets.
    let focus = SnapshotPayload::summary(format!(
        "focused: {} · hovered: {}",
        node_label(runtime.focused().map(NodeId::raw)),
        node_label(runtime.hovered().map(NodeId::raw)),
    ));

    // Accessibility: the semantic tree the runtime already derives from the
    // retained tree — the same one a platform AX bridge consumes, so an inspector
    // and a screen reader see one truth rather than two.
    let semantic = runtime.semantic_snapshot();
    let accessibility = SnapshotPayload::new(
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
    );

    // Scene: the composed display list this frame actually paints.
    let commands = runtime.snapshot();
    let command_count = commands.display_list().commands().len() as u64;
    let scene = SnapshotPayload::new(
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
    );

    // Animations: how many motion timelines are running right now.
    let running = runtime.active_animation_count() as u64;
    let animations = SnapshotPayload::new(
        format!("{running} running timeline(s)"),
        running,
        Vec::new(),
    );

    // Resources: everything requested this session, whatever its state.
    let requested = runtime.resource_count() as u64;
    let resources = SnapshotPayload::new(
        format!("{requested} requested resource(s)"),
        requested,
        Vec::new(),
    );

    // Caches: what is resolved and held, split by kind — a raster image and a
    // parsed vector document are cached separately and cost differently. The image
    // cache is budgeted (§5.19), so its live bytes and hit/miss/eviction counters
    // are reported too: a count alone cannot answer why an image was dropped, which
    // is the question a budgeted cache exists to raise.
    let (images, vectors) = runtime.cached_resource_counts();
    let stats = runtime.image_cache_statistics();
    let caches = SnapshotPayload::new(
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
    ));

    // EventFlow and PluginState are deliberately left to their typed defaults:
    // the runtime keeps no dispatch log to report a flow from, and the plugin tier
    // is a declared-deferred skeleton with no state to show. Synthesising either
    // would give an inspector a confident answer about something that does not
    // exist — worse than an honest empty.
    SnapshotInspectSource::new()
        .with_snapshot(Inspectable::Tree, tree)
        .with_snapshot(Inspectable::Layout, layout)
        .with_snapshot(Inspectable::Styles, styles)
        .with_snapshot(Inspectable::Focus, focus)
        .with_snapshot(Inspectable::Accessibility, accessibility)
        .with_snapshot(Inspectable::Scene, scene)
        .with_snapshot(Inspectable::Animations, animations)
        .with_snapshot(Inspectable::Resources, resources)
        .with_snapshot(Inspectable::Caches, caches)
}

#[doc = include_str!("devtools.doc/node_label.md")]
fn node_label(raw: Option<u64>) -> String {
    raw.map_or_else(|| "none".to_owned(), |id| format!("#{id}"))
}

#[doc = include_str!("devtools.doc/apply_injected.md")]
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

#[doc = include_str!("devtools.doc/apply_control.md")]
pub fn apply_control(runtime: &mut UiRuntime, action: ControlAction) {
    match action {
        ControlAction::SetInspectMode(on) => runtime.set_inspect_mode(on),
        ControlAction::Pause | ControlAction::Resume | ControlAction::StepFrame => {}
    }
}

#[doc = include_str!("devtools.doc/perf_payload.md")]
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

#[doc = include_str!("devtools.doc/live_source.md")]
#[must_use]
pub fn live_source(runtime: &UiRuntime, stats: &FrameStats) -> SnapshotInspectSource {
    inspect_source_from(runtime).with_snapshot(Inspectable::Rendering, perf_payload(stats))
}

#[doc = include_str!("devtools.doc/SharedInspectSource.md")]
#[derive(Clone, Default)]
pub struct SharedInspectSource {
    inner: Arc<Mutex<SnapshotInspectSource>>,
    #[doc = include_str!("devtools.doc/clients.md")]
    clients: Arc<AtomicUsize>,
}

impl SharedInspectSource {
    #[doc = include_str!("devtools.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("devtools.doc/set.md")]
    pub fn set(&self, source: SnapshotInspectSource) {
        let mut guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *guard = source;
    }

    #[doc = include_str!("devtools.doc/has_client.md")]
    #[must_use]
    pub fn has_client(&self) -> bool {
        self.clients.load(Ordering::Relaxed) > 0
    }

    #[doc = include_str!("devtools.doc/client_connected.md")]
    pub fn client_connected(&self) {
        self.clients.fetch_add(1, Ordering::Relaxed);
    }

    #[doc = include_str!("devtools.doc/client_disconnected.md")]
    pub fn client_disconnected(&self) {
        // Saturating: an unmatched disconnect must never wrap to "many clients".
        let _ = self
            .clients
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
                Some(n.saturating_sub(1))
            });
    }
}

impl InspectSource for SharedInspectSource {
    fn snapshot(&self, target: Inspectable) -> SnapshotPayload {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .snapshot(target)
    }
}

#[doc = include_str!("devtools.doc/DiagnosticEventSink.md")]
pub struct DiagnosticEventSink {
    events: EventQueue,
}

impl DiagnosticEventSink {
    #[doc = include_str!("devtools.doc/new_2.md")]
    #[must_use]
    pub fn new(events: EventQueue) -> Self {
        Self { events }
    }
}

impl DiagnosticSink for DiagnosticEventSink {
    fn emit(&self, diagnostic: &Diagnostic) {
        // A full queue drops its oldest and counts the loss in
        // `EventQueue::dropped`, which is where a host reads it; failing the
        // emit here would only turn a reported gap into an unreported one.
        let _ = self.events.push(
            Topic::Diagnostics,
            EventKind::Diagnostic {
                severity: event_severity(diagnostic.severity),
                code: diagnostic.code.as_str().into(),
                message: diagnostic.message.as_str().into(),
            },
        );
    }
}

#[doc = include_str!("devtools.doc/event_severity.md")]
const fn event_severity(severity: DiagnosticSeverity) -> EventSeverity {
    match severity {
        DiagnosticSeverity::Trace => EventSeverity::Trace,
        DiagnosticSeverity::Info => EventSeverity::Info,
        DiagnosticSeverity::Warning => EventSeverity::Warning,
        DiagnosticSeverity::Error => EventSeverity::Error,
        DiagnosticSeverity::Critical => EventSeverity::Critical,
    }
}

#[doc = include_str!("devtools.doc/RuntimeEventPublisher.md")]
pub struct RuntimeEventPublisher {
    events: EventQueue,
    #[doc = include_str!("devtools.doc/last_commit.md")]
    last_commit: VersionStamp,
    #[doc = include_str!("devtools.doc/last_focus.md")]
    last_focus: Option<NodeId>,
}

impl RuntimeEventPublisher {
    #[doc = include_str!("devtools.doc/attach.md")]
    #[must_use]
    pub fn attach(runtime: &UiRuntime, events: EventQueue) -> Self {
        Self {
            events,
            last_commit: runtime.commit_version(),
            last_focus: runtime.focused(),
        }
    }

    #[doc = include_str!("devtools.doc/publish.md")]
    pub fn publish(&mut self, runtime: &UiRuntime) {
        let commit = runtime.commit_version();
        if commit != self.last_commit {
            self.last_commit = commit;
            // See `DiagnosticEventSink::emit` on the dropped return.
            let _ = self.events.push(
                Topic::Frames,
                EventKind::FrameCommitted { id: commit.raw() },
            );
        }
        let focus = runtime.focused();
        if focus != self.last_focus {
            self.last_focus = focus;
            let _ = self.events.push(Topic::Focus, EventKind::FocusChanged);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
