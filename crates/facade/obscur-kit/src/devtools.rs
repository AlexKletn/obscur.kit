//! Bridging a live [`UiRuntime`] into the external DevTools endpoint (spec §7.6).
//!
//! The DevTools server serves inspect requests from an
//! [`InspectSource`](obscur_devtools_server::bridge::InspectSource) the host
//! provides; [`inspect_source_from`] builds one from a runtime's
//! [`inspect_snapshot`](crate::runtime::UiRuntime::inspect_snapshot), so an
//! external agent connecting over the socket sees the *real* tree, layout and
//! styles instead of a placeholder. The runtime refreshes the source after a
//! frame and hands it to the endpoint (the host-provides-device pattern), which
//! keeps the source `Send`-able onto the serving thread without borrowing the
//! runtime.
//!
//! # The pushed half
//!
//! Inspecting is pull; a subscription is push. [`DiagnosticEventSink`] and
//! [`RuntimeEventPublisher`] are the producers that give a subscription
//! something to deliver — they turn the runtime's real diagnostics, tree commits
//! and focus changes into protocol events on an
//! [`EventQueue`](obscur_devtools_server::transport::EventQueue) that
//! [`SocketTransport::serve_with_events`](obscur_devtools_server::transport::SocketTransport::serve_with_events)
//! fans out. A host that serves with plain
//! [`serve`](obscur_devtools_server::transport::SocketTransport::serve) instead
//! has no such loop, and its endpoint therefore *denies* subscriptions rather
//! than granting one nothing would service.

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

/// Builds a [`SnapshotInspectSource`] populated from `runtime`'s current live
/// state: the full annotated tree (Tree), per-node geometry (Layout), per-node
/// computed styles (Styles), and the focus/hover summary (Focus). Targets the
/// runtime does not describe fall back to the source's typed default.
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

/// Renders an optional node id as `#N` or `none`.
fn node_label(raw: Option<u64>) -> String {
    raw.map_or_else(|| "none".to_owned(), |id| format!("#{id}"))
}

/// Applies one authorized [`InjectedEvent`] (drained from the endpoint) to
/// `runtime` — the write half of the agent loop over the protocol (spec §7.6).
///
/// Pointer presses/releases map to a synthetic [`PointerEvent`] the runtime
/// hit-tests exactly like real input, so an external agent can drive clicks and
/// focus. Key injection is not yet mapped (a stable keycode → the runtime's
/// structured `KeyEvent` needs a keymap — a follow-up); such events are ignored.
///
/// # These events are untrusted, and must stay that way
///
/// Whoever holds an `Inject`-scoped token reaches this function, so nothing it
/// builds is a user gesture. Each event is left at
/// [`EventTrust::Untrusted`](obscur_interaction::event::EventTrust::Untrusted) —
/// [`PointerEvent::new`]'s default — and the runtime reports that out through
/// [`PointerDispatch::trust`](obscur_runtime::runtime::PointerDispatch::trust),
/// which is what lets an application distinguish a driven click from a real one.
///
/// **Never call `PointerEvent::as_trusted` here.** That method exists for a
/// platform adapter holding an actual OS event; calling it on this path would
/// make injected input indistinguishable from a real gesture, which is the exact
/// condition this flag exists to rule out. `injected_pointer_is_not_trusted` in
/// `tests/devtools_live.rs` fails if it is ever added.
///
/// # Returns
///
/// The [`PointerDispatch`] the runtime produced, or [`None`] for an event this
/// path does not map (the key variants). Returning it is what makes the
/// untrusted mark *actionable*: `handle_pointer` reports trust only in its
/// return value — a direct call records nothing in
/// [`UiRuntime::take_input_dispatches`], which the queued path fills — so a
/// function that swallowed the dispatch would leave a host no way to tell that
/// the click it just applied came from an agent rather than a person.
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

/// Renders host-side frame performance ([`FrameStats`]) as the
/// [`Inspectable::Rendering`] snapshot payload (spec §7.9), so an external
/// DevTools client sees live FPS + CPU/GPU timings instead of a placeholder.
///
/// A host attaches it to the source it serves, e.g.
/// `inspect_source_from(&rt).with_snapshot(Inspectable::Rendering, perf_payload(&stats))`.
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

/// A thread-shareable [`InspectSource`] whose backing snapshot the host swaps out
/// each frame while a serving thread reads it (spec §7.6 host-provides-device).
///
/// The host holds one handle and calls [`set`](Self::set) after every frame with
/// a freshly built [`SnapshotInspectSource`] (e.g. from [`inspect_source_from`]
/// plus [`perf_payload`]); a [`clone`](Clone::clone) handed to a
/// [`DevToolsEndpoint`](obscur_devtools_server::endpoint::DevToolsEndpoint) on a
/// serving thread answers every inspect from the latest snapshot. Cloning shares
/// the same inner state (an `Arc`), so updates on one handle are visible on all.
///
/// # Pay only while someone is watching
///
/// Building a live source walks the whole tree and formats every node
/// ([`inspect_source_from`]), which is far too expensive to do on a frame nobody
/// is inspecting. The serving side brackets each session with
/// [`client_connected`](Self::client_connected)/[`client_disconnected`](Self::client_disconnected),
/// and the host gates its publish on [`has_client`](Self::has_client) — so an
/// unobserved window pays nothing.
#[derive(Clone, Default)]
pub struct SharedInspectSource {
    inner: Arc<Mutex<SnapshotInspectSource>>,
    /// Live DevTools sessions; the host only publishes while this is non-zero.
    clients: Arc<AtomicUsize>,
}

impl SharedInspectSource {
    /// Creates a shared source backed by an empty snapshot, with no clients.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces the backing snapshot with `source` (called by the host while
    /// [`has_client`](Self::has_client)).
    ///
    /// A poisoned lock (a serving thread panicked mid-read) is recovered rather
    /// than propagated, so a DevTools client fault never takes down the host.
    pub fn set(&self, source: SnapshotInspectSource) {
        let mut guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *guard = source;
    }

    /// Whether at least one DevTools session is currently connected — the host's
    /// gate for doing the (expensive) live-source build at all.
    #[must_use]
    pub fn has_client(&self) -> bool {
        self.clients.load(Ordering::Relaxed) > 0
    }

    /// Registers a newly accepted DevTools session (called by the serving side).
    pub fn client_connected(&self) {
        self.clients.fetch_add(1, Ordering::Relaxed);
    }

    /// Retires a finished DevTools session (called by the serving side).
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

/// Republishes the runtime's structured diagnostics (§7.8) as DevTools
/// [`Topic::Diagnostics`] events (§7.6), so an external subscriber sees a failure
/// as it happens rather than by polling for it.
///
/// Install it on the runtime with
/// [`set_diagnostic_sink`](crate::runtime::UiRuntime::set_diagnostic_sink) and
/// hand a [`clone`](Clone::clone) of the same [`EventQueue`] to
/// [`SocketTransport::serve_with_events`](obscur_devtools_server::transport::SocketTransport::serve_with_events).
/// The sink states only *what happened*; the serving thread's endpoint still
/// decides who — if anyone — is subscribed and may hear it, so installing this
/// grants a client nothing it was not already authorised for.
///
/// # What reaches it
///
/// Exactly what reaches the runtime's sink: component-render failures and the
/// §1.7 boundary decision taken for each. Layout, style, renderer and resource
/// failures still take their own paths and do not arrive here — see
/// `set_diagnostic_sink`'s own list. The [`DiagnosticContext`] (node, component)
/// is **not** carried onto the wire: [`EventKind::Diagnostic`] has no field for
/// it, so a subscriber gets severity, code and message only.
///
/// [`DiagnosticContext`]: crate::diagnostics::DiagnosticContext
pub struct DiagnosticEventSink {
    events: EventQueue,
}

impl DiagnosticEventSink {
    /// Creates a sink publishing onto `events`.
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

/// Maps the runtime's severity ladder onto the protocol's identical one.
const fn event_severity(severity: DiagnosticSeverity) -> EventSeverity {
    match severity {
        DiagnosticSeverity::Trace => EventSeverity::Trace,
        DiagnosticSeverity::Info => EventSeverity::Info,
        DiagnosticSeverity::Warning => EventSeverity::Warning,
        DiagnosticSeverity::Error => EventSeverity::Error,
        DiagnosticSeverity::Critical => EventSeverity::Critical,
    }
}

/// Publishes a live runtime's frame and focus transitions onto a DevTools
/// [`EventQueue`] (spec §7.6 [`Topic::Frames`] / [`Topic::Focus`]).
///
/// The host calls [`publish`](Self::publish) once per frame; each call compares
/// the runtime's real state against what was last published and emits an event
/// only on a genuine change — so the stream reports transitions the runtime
/// actually made, and an idle frame produces nothing.
///
/// # What it reports, and what it does not
///
/// * [`Topic::Frames`] — one [`EventKind::FrameCommitted`] per advance of the
///   runtime's [`commit_version`](crate::runtime::UiRuntime::commit_version),
///   carrying that version as the id. A frame whose reconcile changed nothing
///   does not advance the version, so it is not reported: the topic is
///   *tree-commit* events, not present/vsync events.
/// * [`Topic::Focus`] — one [`EventKind::FocusChanged`] per change of
///   [`focused`](crate::runtime::UiRuntime::focused).
/// * [`Topic::Lifecycle`] — **nothing**. The runtime exposes no lifecycle
///   transition stream to derive it from, so a `Lifecycle` subscriber on a host
///   that publishes only through this type is granted a subscription that stays
///   silent. That gap is open.
pub struct RuntimeEventPublisher {
    events: EventQueue,
    /// The last commit stamp published, seeded from the runtime at
    /// [`attach`](Self::attach) so the publisher reports what happens *after* it
    /// is installed and never invents a frame for state that predates it.
    last_commit: VersionStamp,
    /// The last focus target published, seeded the same way.
    last_focus: Option<NodeId>,
}

impl RuntimeEventPublisher {
    /// Starts publishing `runtime`'s transitions onto `events`, taking the
    /// runtime's current commit version and focus as the baseline.
    #[must_use]
    pub fn attach(runtime: &UiRuntime, events: EventQueue) -> Self {
        Self {
            events,
            last_commit: runtime.commit_version(),
            last_focus: runtime.focused(),
        }
    }

    /// Publishes an event for each of `runtime`'s transitions since the last
    /// call. Cheap enough for every frame: two reads and two comparisons when
    /// nothing changed.
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
