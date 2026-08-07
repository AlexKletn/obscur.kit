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
| `set_pseudo_state(NodeId, PseudoState)` | record an interaction state for subsequent frames |
| `pseudo_state(NodeId) -> PseudoState` | |
| `clear_pseudo_states()` | subsequent frames match as if nothing were hovered/focused/active |
| `set_inline_style_compiler(InlineStyleCompiler)` | how a `style="…"` attribute becomes a declaration block at cascade origin `Inline`. **The runtime cannot do this itself** — an application-tier crate may not depend on the CSS front end — so with none installed every `style` attribute is inert |
| `has_inline_style_compiler() -> bool` | |

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
| `scroll_by(NodeId, f32) -> f32` | returns the resulting `scroll_top` |
| `scroll_into_view(NodeId) -> bool` | scrolls the nearest scrollable ancestor; whether anything moved |
| `handle_scroll(f32, Point) -> Option<NodeId>` | see [Input](#input) |

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

> **host-wired** — the runtime consumes these correctly and CSS reads them, but
> no shipped host currently *reports* keyboard geometry: the values are whatever
> the application sets.

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
| `request_background()` | enqueue one unit of band-5 work |
| `pending_background() -> usize`, `serviced_background() -> u64` | |
| `is_background_starved() -> bool` | preempted past the scheduler's fairness threshold |

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
| `set_accessibility_bridge(Rc<RefCell<dyn AccessibilityBridge>>)` | every frame publishes to it; pushes immediately on install |
| `has_accessibility_bridge() -> bool` | |
| `push_accessibility(&mut dyn AccessibilityBridge)` | push once, by hand |

> **host-wired** — the outbound path is real, behind the non-default `accesskit`
> feature. The inbound direction (a platform AT driving the app) is not built.

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
