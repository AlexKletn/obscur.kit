# 03 — The runtime

`obscur_kit::runtime::UiRuntime` — the whole toolkit minus the window. It holds
the retained tree, the computed styles, the arranged boxes, the scene, focus and
hover, the motion timelines, the command registry and the scheduler.

It owns none of the platform. No event loop, no surface, **no clock**: a host
hands it input as data, hands it elapsed time as a number, and takes a scene
snapshot back out to draw. That is what lets one runtime drive a winit window, a
headless test and a PNG without knowing which it is in.

What follows is the surface you would actually reach for, grouped by what you
would be doing at the time. It is **not exhaustive** — `UiRuntime` has around a
hundred and fifty public methods and a page that claimed to list all of them
would be wrong within a week. `cargo doc -p obscur-kit --open` is the complete
list; this is the map.

---

## Construction, viewport and scale

```rust
let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(800.0, 600.0));
```

| Method | |
|---|---|
| `new(id: RuntimeId, viewport: Size) -> Self` | identity and viewport, nothing installed |
| `runtime_id() -> RuntimeId` / `id() -> RuntimeId` | the identity; `id` is an alias |
| `viewport() -> Size` | the size frames lay out under, in logical px |
| `set_viewport(Size)` | change it |
| `set_surface(physical: Size, scale: f32)` | the one call a window host wants: sets the device scale **and** derives the logical viewport from the physical surface |
| `device_scale() -> f32` | HiDPI factor × user zoom |
| `set_device_scale(f32)` | **clamped**, not validated: anything below `0.1` — including `0.0`, a negative, and `NaN` — becomes `0.1`. See the warning below |
| `content_offset() -> (f32, f32)` / `set_content_offset(x, y)` | the content point pinned to the viewport's top-left, in logical px |
| `viewport_scroll() -> (f32, f32)` / `set_viewport_scroll(x, y)` | the document-coordinate point at the viewport's top-left |

`set_surface` is what a resizable window calls; getting scale and viewport from
one place is what keeps a HiDPI window's layout in CSS pixels while its texture
stays physical. See [08 — Rendering](08-rendering.md).

> **A bad scale is clamped, and clamping is not the same as ignoring.** This page
> used to say a non-positive or non-finite value was ignored — it is not. `0.0`,
> `-4.0` and `NaN` all set the scale to **`0.1`**, and `set_surface` then divides
> the physical size by the *clamped* value, so a 1920×1080 surface reported with
> a scale of `0` lays out at a **19200×10800** logical viewport. That is a
> plausible way to hang a frame, and it is why a host must not forward a
> platform's scale unchecked. `tests/surface_mapping.rs`'s
> `set_device_scale_clamps_to_a_sane_minimum` pins each case.

## Window lifecycle for embedded hosts

Report native window creation and closure using
`runtime.lifecycle_mut().window_opened(id)` and `window_closed(id)`.
Drain notifications with `take_events()`. Closing the final tracked window
queues `LifecycleEvent::LastWindowClosed(id)`. The host decides whether to
keep the runtime active, reopen a window, or exit.
Call `request_shutdown(ShutdownRequest::Host)` to queue
`LifecycleEvent::ShutdownRequested` for the host to handle.
Use `report_critical_stop(concern)` to report a critical failure in a named
subsystem. The standalone Winit runner reports event-loop, window and renderer
failures through the same lifecycle event. `App::run()` returns the error details.
When the host is ready to release the runtime, `runtime.release().into_events()`
returns its pending notifications followed by `ResourceReleaseComplete(id)`.
This event covers runtime-owned handles. The host manages any separately
owned windows, shared resources and tasks.

For a standalone Winit window, keep the handle from `App::control()` before
running the app. After `App::run()` returns, use
`control.take_lifecycle_events()` to read the notifications. A native close
request or `WindowAction::Close` queues `ShutdownRequested(WindowClose(id))`
before the runner exits. `LastWindowClosed(id)` follows when it closes, and
`ResourceReleaseComplete` follows after the runner drops its resources.

For standalone Winit windows, `control.capture_window_placement()` returns a
versioned Serde snapshot of the last normal logical size and presentation mode.
It remains available after closing. Use `App::with_window_placement(snapshot)`
at startup. The current size limits and the platform's placement policy apply.
The snapshot covers size and mode. Screen coordinates and minimization are
outside this format.

## Running frames

Three ways to produce a frame, in increasing order of how much the runtime does
for you:

| Method | |
|---|---|
| `render_now(&View, &[Stylesheet]) -> FrameResult` | one frame, **ignoring the scheduler**. The simplest thing that works, and what tests use |
| `run_due_frame(&View, &[Stylesheet]) -> Option<FrameResult>` | one frame *if* one was requested; `None` when nothing is due |
| `run_due_frame_reactive() -> Option<FrameResult>` | the same, but the runtime rebuilds the view itself from the installed builder |

| Supporting | |
|---|---|
| `set_root_view(impl FnMut() -> View + 'static)` | the builder `run_due_frame_reactive` calls |
| `has_root_view() -> bool` | |
| `set_stylesheets(Vec<Stylesheet>)` | the sheets the reactive path matches against |
| `request_frame(FrameReason) -> BackpressureDecision` | ask for a frame; the scheduler may coalesce or refuse |
| `has_pending_frame() -> bool`, `pending_frames() -> usize` | |
| `commit_version() -> VersionStamp` | stamped at the most recent frame that committed a real tree change |
| `set_frame_budget(FrameBudget)` / `frame_budget()` | per-phase cost budget checked after each phase |

`render_now` is not a shortcut for the others — it deliberately bypasses
admission control, so a host that calls it in a loop is choosing to render every
time rather than when something changed.

## Stylesheets and computed style

| Method | |
|---|---|
| `set_default_stylesheet(Stylesheet)` | the UA base sheet (see [04 — Styling](04-styling.md)) |
| `has_default_stylesheet() -> bool` | |
| `computed_style(NodeId) -> Option<&ComputedStyle>` | as resolved on the last produced frame |
| `set_pseudo_state(NodeId, PseudoState)` | temporarily replace the node’s interaction state for a subsequent frame |
| `pseudo_state(NodeId) -> PseudoState` | |
| `clear_pseudo_states()` | subsequent frames match as if nothing were hovered/focused/active |
| `set_inline_style_compiler(InlineStyleCompiler)` | how a `style="…"` attribute becomes a declaration block at cascade origin `Inline`. **The runtime cannot do this itself** — an application-tier crate may not depend on the CSS front end — so with none installed every `style` attribute is inert |
| `has_inline_style_compiler() -> bool` | |

The runtime rebuilds interaction pseudo-states when hover, focus, or press
changes. Set the `CHECKED` and `DISABLED` view attributes for persistent
application-owned state.

`CssPipeline::install` (so `KitBootstrap`, so `App`) installs the inline-style
compiler for you; a host driving a bare `UiRuntime` must call
`set_inline_style_compiler` or accept that `style="…"` does nothing.

The pseudo-state methods exist for hosts that drive states themselves;
`handle_pointer` maintains them for you.

## Canvas painters

Any element can be drawn by a registered painter, not only a `<canvas>`: the
runtime's map is keyed by `PainterKey`, which is either a symbol a node named
itself or a name a stylesheet's `-obscur-painter` gave it.

| Method | |
|---|---|
| `register_canvas(InternedSymbol, CanvasPainter)` | under the **symbol** a node binds in its own `canvas_painter` property — what the shipped `Slider` and `Progress` do |
| `register_named_canvas(&str, CanvasPainter)` | under the CSS-visible **name**, which is what `-obscur-painter: <ident>` reaches |
| `take_unbalanced_canvases() -> Vec<PainterKey>` | painters that left a clip or layer bracket open, taken once so a host reports each one rather than every frame |

The two keys are separate namespaces, and a node's own symbol is looked up first
— a stylesheet cannot redirect the routine that draws a progress bar. Registering
is by key and the last one wins. See
[04 — Styling](04-styling.md#vendor-properties).

## Input

| Method | |
|---|---|
| `handle_pointer(PointerEvent) -> PointerDispatch` | hit-tests, updates hover and focus, recomputes pseudo-states, schedules a frame if anything changed |
| `handle_key(KeyEvent) -> KeyDispatch` | routes to the focused node, resolves bound shortcuts, schedules a frame |
| `handle_wheel(WheelEvent) -> Option<NodeId>` | the self-describing wheel door: both axes, a unit, a timestamp and trust. Dispatches the raw `wheel` event before scrolling, so a handler can refuse the notch |
| `handle_scroll(delta_y: f32, position: Point) -> Option<NodeId>` | the two-float form: pixels, no horizontal, untrusted, unstamped. Routes to the **innermost** scroll container under the point; `None` means nothing took it |
| `UiRuntime::WHEEL_LINE_STEP` | `48.0` — how far one notched line scrolls. A constant of this toolkit's policy, deliberately not of the winit adapter |
| `handle_ime(CompositionEvent) -> bool` | applies a composition against the focused editable |
| `held_modifiers() -> Modifiers` | folded from the physical modifier transitions `handle_key` has seen |
| `set_modifiers(Modifiers)` | a platform that knows better says so — winit's `ModifiersChanged`. Without it the fold goes stale when a modifier is released while unfocused |
| `queue_input(InputEvent) -> BackpressureDecision` | queue for the frame pipeline's dispatch phase instead of dispatching now |
| `pending_input() -> usize` | |
| `take_input_dispatches() -> Vec<InputDispatch>` | what the last frame's dispatch phase produced, in queue order |

`handle_pointer` is device-agnostic: a finger, a pen and a mouse enter through
it and differ only in the `PointerDevice` they carry. See [06 — Input](06-input.md).

### Pointer capture

| Method | |
|---|---|
| `capture_pointer(PointerId, NodeId) -> bool` | every later event from that pointer targets `node` regardless of what it is over |
| `release_pointer_capture(PointerId) -> bool` | whether one was held |
| `pointer_capture(PointerId) -> Option<NodeId>` | |

## Focus

Use `capture_focus_state()` to save a focused control with a unique
`automation-id`. The versioned `FocusState` supports Serde. After layout in
the new session, `restore_focus_state(&state)` resolves the ID again and
checks tab eligibility and the active focus trap before moving focus.

| Method | |
|---|---|
| `focused() -> Option<NodeId>` | |
| `focus_node(Option<NodeId>) -> bool` | move focus programmatically; `None` clears. `false` when the node is not in the tree. Fires `blur`/`focus` like any other move, and deliberately does **not** set `:focus-visible` |
| `is_focus_visible() -> bool` | whether `:focus-visible` currently matches — true only after a keyboard-origin move |
| `focus_accepts_text() -> bool` | whether what holds focus takes typed text — what a host asks before offering an IME or raising an on-screen keyboard |
| `open_focus_scope(FocusScope)` | pushes a containment scope as the innermost |
| `close_focus_scope(FocusScopeId) -> bool` | |
| `focus_scopes() -> &[FocusScope]` | outermost first |

## Commands

The routing chain, in the order `command_route` returns it: **focused node →
focus scope → ancestors → window → application fallback.**

| Method | Link |
|---|---|
| `on_command_at(NodeId, InternedSymbol, handler)` | 1 and 3 |
| `has_node_command_handler(NodeId, InternedSymbol) -> bool` | |
| `clear_node_commands(NodeId) -> usize` | |
| `on_scope_command(FocusScopeId, InternedSymbol, handler)` | 2 |
| `has_scope_command_handler(FocusScopeId, InternedSymbol) -> bool` | |
| `on_command(InternedSymbol, handler)` | 4 — window level; this is the one an app usually wants |
| `has_command_handler(InternedSymbol) -> bool` | |
| `set_app_commands(AppCommands)` / `app_commands() -> AppCommands` | 5 |
| `add_event_listener(NodeId, InternedSymbol, ListenerPhase, listener)` | a §2.3 listener on a node, in `Capture` or `Bubble` |
| `event_listener_count(NodeId, InternedSymbol) -> usize` | |
| `dispatch_command(InternedSymbol) -> bool` | run a command through the chain from the focused node, as if something had raised it |
| `command_route(Option<NodeId>) -> Vec<CommandTarget>` | the chain itself, for inspection |

| Shortcuts and descriptors | |
|---|---|
| `bind_command(Shortcut, InternedSymbol)` | a key press at `handle_key` resolves to and dispatches this command |
| `command_router() -> &CommandRouter` | the binding table |
| `register_command(Command)` | a unified descriptor; binds its shortcut if it declares one |
| `command_registry() -> &CommandRegistry` | descriptors and their live state |
| `set_command_state(InternedSymbol, CommandState) -> bool` | `false` when nothing is registered under that name |

> **partial** — §6.7's action model names seven surfaces a `Command` should
> drive. Two exist: keyboard shortcuts and `Menu`. Custom context menus are not
> built.

### The event a handler receives (§2.3)

Every `handler` above is an `IntoCommandHandler`: either `impl FnMut()` or
`impl FnMut(&mut UiEvent)`. A one-argument closure needs its parameter annotated
(`|e: &mut UiEvent|`) — inference cannot pick the marker and the closure's
signature at once.

Dispatch runs §2.3's four phases in order — capture, target, bubble, default
action — through the domain `EventDispatcher`, honouring `stop_propagation`,
`stop_immediate_propagation` and `prevent_default`. A `UiEvent` carries `target`,
`current_target`, `trust` and a typed payload; a pointer payload carries §2.4's
pointer id, device, buttons, logical position, `delta`, pressure, `tilt`,
`rotation`, primary-pointer status and hover capability.

Until this was driven, the model existed as domain types with a spec test and no
caller at all: an application handler was an `FnMut()` given nothing, selected by
an FNV-1a hash of a name — which is the one mechanism §2.3 rules out by name.
Handler *names* are still hashed. That half is closed by the compiler rather than
the runtime, and only on the `.okui` build path (see
[09](09-authoring-formats.md#validation)).

## Text editing and IME

| Method | |
|---|---|
| `on_text_input(InternedSymbol, impl FnMut(&str))` | per-keystroke, with the resulting text |
| `has_text_input_handler(InternedSymbol) -> bool` | |
| `on_change(InternedSymbol, impl FnMut(&str))` | on **commit**, once — the `change` event |
| `has_change_handler(InternedSymbol) -> bool` | |
| `begin_text_editing(NodeId, impl Into<String>)` | seat a session explicitly, caret at the end |
| `set_editing_text(impl Into<String>)` | re-seed the live session; clears history |
| `editing_text() -> Option<&str>` | |
| `editing_selection() -> Option<TextSelection>` | caret and selection |
| `composition_preedit() -> Option<&str>` | live IME preedit, if a composition is in flight |
| `on_value_input(InternedSymbol, impl FnMut(f32, f32))` | what a **numeric** control — a slider — reports its new value to. Both ends are passed; a single-thumb slider reports the same number twice. Controlled like a field: an application that does not re-render with the new value sees the thumb spring back |

An element is a text field to the runtime when its ancestor chain binds
`EVENT_INPUT` **and** the node is not a control that holds its own value. The
second half is not a second list of editable types — it is the semantic role,
which the accessibility projection already computes: a `Slider`, `Progress`,
`Checkbox`, `RadioButton`, `Button` or `Link` is skipped. `on:input` is how every
value-bearing control reports itself, exactly as on the web, so the binding alone
cannot mean "this is text": a slider carrying one used to seat a caret inside
itself and swallow the arrow keys meant to move its thumb.

## Scrolling

| Method | |
|---|---|
| `scroll_top(NodeId) -> f32` | `0.0` for a node that is not a scroll container |
| `set_scroll_top(NodeId, f32)` | clamped to `0 ..= content_height − scrollport_height` |
| `scroll_left(NodeId) -> f32` | current horizontal offset in logical pixels |
| `set_scroll_left(NodeId, f32)` | clamps to the available horizontal range and schedules a frame |
| `scroll_by(NodeId, f32) -> f32` | returns the resulting `scroll_top` |
| `scroll_into_view(NodeId) -> bool` | scrolls the nearest scrollable ancestor; whether anything moved |
| `handle_scroll(f32, Point) -> Option<NodeId>` | see [Input](#input) |

For horizontal writing, `scroll_left` is zero at the inline start. In LTR
it increases toward the right. In RTL it becomes negative toward the left.
Horizontal scrollbars show the current position and range as an overlay.
The horizontal thumb supports mouse and direct-contact drag and keeps its
pointer capture outside the track until release or cancellation. A
`pointerdown` handler on the scroll container can prevent the drag.

### Persisting a scroll position

Call `capture_scroll_state(node)` after layout to capture both axes. The
versioned `ScrollState` supports Serde. Store it under an application-defined
key, then resolve the new container after layout and call
`restore_scroll_state(node, state)`. The runtime clamps both axes to the
current content. Unknown versions, non-finite values, and missing containers
are rejected without changing the offset.

## Virtual lists

The runtime windows the list: it builds only the rows that fit, sizes the
container from `count × row_extent` so the scrollbar is honest, and maps a click
back to the model index.

| Method | |
|---|---|
| `register_virtual_list(InternedSymbol, Box<dyn FnMut(usize) -> ViewNode>)` | the row builder |
| `on_virtual_row_click(InternedSymbol, Box<dyn FnMut(usize, u32)>)` | `(model index, click count)` |
| `on_virtual_row_long_press(InternedSymbol, Box<dyn FnMut(usize)>)` | |
| `virtual_index_at(Point) -> Option<(InternedSymbol, usize)>` | which list, which item |
| `scroll_virtual_list(InternedSymbol, dy: f32, _content: f32) -> f32` | clamped; returns the new offset |
| `virtual_window(InternedSymbol) -> Option<Range<usize>>` | what the last frame realized |

`OBSCUR_VLIST_DEBUG=1` prints the windowing decisions.

## Motion

| Method | |
|---|---|
| `animate(node, property, track, duration, kind, interruption) -> TimelineId` | |
| `animate_with(…, playback: MotionPlayback) -> TimelineId` | plus delay, iteration count, direction and fill mode |
| `advance_motion(delta: f32)` | advance every timeline by `delta` **logical seconds**; requests a frame while motion remains |
| `active_animations() -> usize` / `active_animation_count() -> usize` | |
| `set_css_motion(CssMotionRules)` | the `transition` / `@keyframes` rules the runtime drives |

`advance_motion` is the clock. Nothing animates in a host that never calls it —
which was a real defect here once: CSS animations worked in tests and were frozen
in windows, because only the test path ticked.

## Environment and insets

| Method | |
|---|---|
| `set_environment(EnvironmentValues) -> bool` | safe-area insets and keyboard geometry together; `true` when something changed |
| `environment() -> EnvironmentValues` | |
| `set_safe_area_insets(Insets) -> bool` | leaves keyboard geometry alone |
| `set_keyboard_insets(Insets) -> bool` | the region the on-screen keyboard occludes; all zeroes when hidden |

These are what CSS `env(safe-area-inset-*)` resolves against.

Standalone hosts can queue updates through `AppControl::set_safe_area_insets`
and `AppControl::set_keyboard_insets`. The application supplies the values in
logical pixels. The bundled window runners currently have no automatic source
for either inset.

## Drag and drop

| Method | |
|---|---|
| `register_drop_target(DropTarget)` / `unregister_drop_target(NodeId)` | |
| `begin_drag(DragSource) -> bool` | |
| `drag_session() -> Option<&DragSession>` | for a host drawing the drag |
| `take_drag_outcome() -> Option<DragOutcome>` | taken once |

## Resources and caches

| Method | |
|---|---|
| `set_resource_providers(loader, decoder)` | resolves `background-image: url(…)`; replacing them clears the caches |
| `set_vector_decoder(Arc<dyn VectorDecoder + Send + Sync>)` | SVG; a separate capability — geometry with `currentColor` still deferred |
| `set_blocking_executor(Arc<dyn BlockingExecutor + Send + Sync>)` | moves loads off the frame thread. Without one a load still runs, deferred to after the frame |
| `poll_resource_tasks() -> usize` | drains finished loads; returns how many applied |
| `resource_count() -> usize` | requested this session — loaded, in flight or failed |
| `cached_resource_counts() -> (usize, usize)` | decoded images, parsed vector documents |
| `set_image_cache_budget(CacheBudget) -> usize` | evicts down to it immediately; returns how many went |
| `image_cache_budget() -> CacheBudget` | |
| `set_image_eviction_policy(EvictionPolicy)` | default is `VisibilityAware` |
| `image_cache_statistics() -> CacheStatistics` | hits, misses, evictions, insertions, resident entries and bytes |

With no decoder installed an SVG `src` resolves to nothing and paints nothing —
never a crash, and never a rasterized fallback.

## Text stack ports

| Method | Default if not installed |
|---|---|
| `set_text_shaper(Box<dyn TextShaper>)` | `MockTextShaper` — deterministic metrics, no glyphs |
| `set_text_itemizer(Box<dyn TextItemizer>)` | `SingleRunItemizer` |
| `set_font_resolver(Box<dyn FontResolver>)` | `MockFontResolver` — empty |
| `set_hyphenator(Box<dyn Hyphenator>)` | `NullHyphenator` |
| `set_locale(&LanguageTag)` / `locale() -> &LanguageTag` | `und` |

The defaults are why a bare runtime lays text out sensibly and paints no glyphs.
[`KitBootstrap::install_text_stack`](07-text-and-fonts.md) replaces all of them.

## Components, signals and transactions

| Method | |
|---|---|
| `register_component(ComponentRegistration) -> Result<(), RegistryError>` | so nodes naming its `ComponentId` instantiate it |
| `mounted_component_count() -> usize` | |
| `notify_signal(SignalId) -> usize` | marks every component depending on it for re-render; returns how many |
| `apply_transaction(&TransactionCommit) -> Option<BackpressureDecision>` | classifies the phases the operations need and requests the frame |

## Background work

| Method | |
|---|---|
| `request_background()` | record one pending background unit for scheduler accounting |
| `enqueue_background(job)` | enqueue a short, nonblocking callback on the runtime owner thread |
| `enqueue_cancellable_background(token, job)` | enqueue a callback that runs only if the token is still active at dispatch |
| `enqueue_background_with_deadline(valid_for, job)` | discard the callback if its lifetime has elapsed in logical runtime time |
| `set_background_budget(BackgroundBudget)` | set separate per-runtime limits for idle polls and due frames after the fairness threshold |
| `pending_background() -> usize`, `serviced_background() -> u64` | |
| `is_background_starved() -> bool` | preempted past the scheduler's fairness threshold |

A poll with no frame due services up to its idle budget and returns `None`.
The idle and fairness budgets default to one unit each. Zero disables
service in the corresponding scheduling mode.
The fairness threshold also allows a unit to run during sustained frame
activity. The host polls to service queued callbacks. Use the configured
task executors for decoding, shaping, or blocking work.

## Output: what a host reads back

| Method | |
|---|---|
| `snapshot() -> SceneSnapshot` | the immutable hand-off a renderer consumes |
| `scene() -> &Scene` | the live scene |
| `layout_boxes() -> &LayoutBoxes` | absolute boxes from the last frame, translated into **viewport-relative** space by the scroll chain above each |
| `tree() -> &RetainedTree` | |
| `hovered() -> Option<NodeId>` | |

## Accessibility

| Method | |
|---|---|
| `semantic_tree() -> SemanticTree` | built from the live retained tree |
| `semantic_snapshot() -> SemanticSnapshot` | immutable and versioned — what a bridge consumes |
| `set_accessibility_bridge(Rc<RefCell<dyn AccessibilityBridge>>)` | publishes changed snapshots and sends the current snapshot on install |
| `has_accessibility_bridge() -> bool` | |
| `push_accessibility(&mut dyn AccessibilityBridge)` | push once, by hand |

The `accesskit` feature supplies the bridge for a host's platform adapter.
Send incoming requests through `AccessKitBridge::push_action_request` and wake
the UI loop. `poll_accessibility_actions()` drains requests automatically at
frame entry and is also available as an explicit host call. Supported actions
are activation, focus, and scrolling a whole node into view.

## Shared-element transitions

Before replacing a view, call `queue_shared_element(source, key, duration, easing)`
to capture a laid-out node. Mark one node in the next view with
`with_shared_element_key(key)`. The new node starts at the captured geometry in
the first reconciled frame and moves to its own layout bounds on the runtime
motion clock. Capturing again mid-transition uses the last displayed position.
The destination uses its own content and clipping. Its authored translation
and positive axis-aligned scaling define the endpoint. Hit testing and
accessibility bounds follow the transform submitted with each composed
frame. Layout and computed style continue to report the destination
geometry and authored values.

## Coordinate spaces

`local_to_window(node)` returns a typed mapper from the node's layout-box
origin to logical window coordinates. The mapping includes visual ancestors,
CSS transforms, scroll offsets and the surface content offset.
Compose it with `window_to_physical()` to include fractional device scale,
or call `inverse()` to map in the opposite direction when invertible.
`window_to_screen(origin)` combines device scale with the host-supplied
client-area origin in desktop physical pixels. The standalone Winit runner
publishes that origin through `AppControl::window_screen_origin()` on
backends that expose it. Wayland returns `None`.

`local_to_parent(node)` maps to the visual parent's layout-box origin,
including its scroll offset. `local_to_normalized(node)` expresses local
points as fractions of the node's width and height. A zero-sized axis
prevents normalization.

## Geometry observers

`observe_resize(node, callback)` reports the initial committed size and later
changes in logical pixels. Each observer receives at most one notification per
frame. Return `true` from the callback to request a UI commit on the next frame.
Use `unobserve_resize(node)` to detach it explicitly. Removing its node drops
the callback automatically.

`observe_layout(node, callback)` reports the full committed layout box,
including position changes at the same size. It uses logical layout
coordinates before scroll and transform adjustments. Resize and layout
subscriptions can coexist on the same node.

`observe_visibility(node, callback)` reports whether a node has visible area.
`observe_intersection(node, threshold, callback)` reports when the visible
area crosses a ratio threshold. Both use the viewport and rectangular
overflow clips in scroll-adjusted layout coordinates.

`observe_transform(node, callback)` reports changes to the composed CSS
transform, including transforms of visual ancestors. It reports the
matrix in logical layout coordinates before applying the scroll offset.

## Diagnostics and inspection

| Method | |
|---|---|
| `set_diagnostic_sink(Box<dyn DiagnosticSink>)` | |
| `report_diagnostic(&Diagnostic)` | |
| `set_telemetry_sink(Box<dyn TelemetrySink>)` | each frame's phases record into it |
| `set_inspect_mode(bool)` / `inspect_mode() -> bool` | the in-window highlight overlay |
| `hovered_inspect() -> Option<InspectInfo>` | `type#id.class`, component id, absolute box |
| `inspect_snapshot() -> TreeInspection` | one `NodeInspection` per node, pre-order — for a headless agent to assert on |

> **partial** — `PerformanceSpan` carries a name and a subsystem but **no
> duration**, so telemetry cannot answer "how long did this take". Until it
> does, `OBSCUR_FRAME_STATS=1` is the honest way to profile. See
> [11 — Diagnostics](11-diagnostics.md).
