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

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use obscur_devtools_protocol::message::{
    ControlAction, InjectedEvent, Inspectable, SnapshotPayload,
};
use obscur_devtools_server::bridge::{InspectSource, SnapshotInspectSource};

use crate::geometry::Point;
use crate::identity::NodeId;
use crate::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use crate::perf::FrameStats;
use crate::runtime::UiRuntime;

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
    // parsed vector document are cached separately and cost differently.
    let (images, vectors) = runtime.cached_resource_counts();
    let caches = SnapshotPayload::new(
        format!("{images} image(s), {vectors} vector document(s)"),
        (images + vectors) as u64,
        Vec::new(),
    )
    .with_detail(format!("image_cache: {images}\nvector_cache: {vectors}"));

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
pub fn apply_injected(runtime: &mut UiRuntime, event: InjectedEvent) {
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
            runtime.handle_pointer(pointer(PointerEventKind::Down, x, y));
        }
        InjectedEvent::PointerUp { x, y } => {
            runtime.handle_pointer(pointer(PointerEventKind::Up, x, y));
        }
        // Key injection needs a keycode→KeyEvent map; not wired yet.
        InjectedEvent::KeyDown { .. } | InjectedEvent::KeyUp { .. } => {}
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
