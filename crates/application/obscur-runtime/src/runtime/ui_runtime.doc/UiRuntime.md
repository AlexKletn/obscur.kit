The embeddable UI runtime: everything the toolkit does, minus the window.

It holds the retained tree, the computed styles, the arranged boxes, the
scene, focus and hover, the motion timelines, the command registry and the
scheduler — and it owns none of the platform. There is no event loop here, no
surface, and **no clock**: a host hands it input as data, hands it elapsed
time as a number ([`advance_motion`](Self::advance_motion)), and takes a
[`SceneSnapshot`](Self::snapshot) back out to draw. That is what makes the
same runtime drive a winit window, a headless test, and a PNG.

# The shape of a host

Construct with [`new`](Self::new), install the ports you want (text stack,
resources, diagnostics — see [`KitBootstrap`] for the batteries-included
version), then drive frames:

- [`render_now`](Self::render_now) — one frame, ignoring the scheduler. The
  simplest thing that works, and what tests use.
- [`run_due_frame`](Self::run_due_frame) — one frame *if* one was requested,
  against a view the caller builds.
- [`run_due_frame_reactive`](Self::run_due_frame_reactive) — the same, but the
  runtime rebuilds the view itself from the builder installed with
  [`set_root_view`](Self::set_root_view). This is the loop
  [`App`](../../../obscur_kit/app/struct.App.html) runs.

# What it does not do

It never opens a window, never talks to a GPU, never reads the system clock,
and never spawns a thread of its own. Every one of those is a port a host
installs, and each is optional: a runtime with nothing installed still lays
out and still produces a scene — text measures on a deterministic mock shaper
and paints no glyphs, images resolve to nothing. Nothing panics for want of a
host.

# What it integrates

It is the application-core **integrator** the catalog mandates (spec §1.4,
§3.4–§3.7). Unlike the bare skeleton it replaces, this `UiRuntime` *owns and
integrates* the moving parts of a live UI:

- the retained [`RetainedTree`] and the mutable [`Scene`] carried across
  frames;
- the orchestration coordinators (reconcile → match → style → layout → scene)
  it drives in a fixed, deterministic order — reimplementing no domain logic;
- the event-driven [`FrameScheduler`] that gates when a frame actually runs
  (§3.5); and
- the per-node [`PseudoStates`] interaction map that finally lets authored
  `:hover`/`:focus` rules match (§3.10, Phase 3 input → styling).

It never owns the process or an event loop: a host drives it by
[`request_frame`](Self::request_frame)-ing as events arrive and calling
[`run_due_frame`](Self::run_due_frame) when it is ready to produce one.

It retains the last frame's absolute [`LayoutBoxes`] (see
[`layout_boxes`](Self::layout_boxes)) — unlike [`FramePipeline`], which
discards them — because hit-testing (Phase 3) needs them.

# Relationship to `HeadlessHost`

[`HeadlessHost`](../../obscur_headless/host/struct.HeadlessHost.html) is the
deterministic, windowless **test host**, and it *drives a `UiRuntime`* rather
than standing beside one: it force-renders through
[`render_now`](Self::render_now) and adds §7.4's two facilities — a
deterministic clock feeding [`advance_motion`](Self::advance_motion), and a
deterministic executor installed via
[`set_blocking_executor`](Self::set_blocking_executor). A headless frame is
therefore the *same* frame a window renders, which is the only way the
harness can test motion, resource completion, or anything else that lives
here rather than in [`FramePipeline`](crate::frame::FramePipeline).

`FramePipeline` remains the bare deterministic core — a pure function of
`(tree, view, inputs, viewport)` with no clock and no async seam. It is what
`UiRuntime` is built from, not an alternative host.

[`FramePipeline`]: crate::frame::FramePipeline
[`KitBootstrap`]: ../../../obscur_kit/bootstrap/struct.KitBootstrap.html
