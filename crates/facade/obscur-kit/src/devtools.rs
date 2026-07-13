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

use obscur_devtools_protocol::message::{ControlAction, InjectedEvent, Inspectable, SnapshotPayload};
use obscur_devtools_server::bridge::SnapshotInspectSource;

use crate::geometry::Point;
use crate::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
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
            let pairs: Vec<String> =
                node.styles.iter().map(|(name, value)| format!("{name}={value}")).collect();
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
        node_label(runtime.focused().map(|n| n.raw())),
        node_label(runtime.hovered().map(|n| n.raw())),
    ));

    SnapshotInspectSource::new()
        .with_snapshot(Inspectable::Tree, tree)
        .with_snapshot(Inspectable::Layout, layout)
        .with_snapshot(Inspectable::Styles, styles)
        .with_snapshot(Inspectable::Focus, focus)
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
