# 06 — Input

Everything that points enters through one door. `UiRuntime::handle_pointer`
takes a mouse click, a finger and a pen stylus alike; hit-testing, hover, focus
and activation read the device off the event rather than living in separate
paths.

```rust
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};

let event = PointerEvent::new(PointerId::MOUSE, PointerDevice::Mouse,
                              PointerEventKind::Down, Point::new(x, y))
    .with_timestamp(micros);
rt.handle_pointer(event);
```

`App` builds these for you from winit events.

---

## `PointerEvent`

```rust
pub const fn new(pointer: PointerId, device: PointerDevice,
                 kind: PointerEventKind, position: Point) -> Self
pub const fn with_buttons(self, buttons: PointerButtons) -> Self
pub fn with_pressure(self, pressure: f32) -> Self
pub const fn with_timestamp(self, timestamp_micros: u64) -> Self
pub const fn as_trusted(self) -> Self
```

Readers: `pointer()`, `device()`, `kind()`, `position()`, `buttons()`,
`pressure()`, `timestamp()`, `is_pressed()`, `trust()`, `is_trusted()`.

| | |
|---|---|
| `PointerEventKind` | `Down`, `Up`, `Move`, `Enter`, `Leave`, `Cancel` — `is_terminal()` covers the last two ways a stream can end |
| `PointerDevice` | `Mouse`, `Touch`, `Pen`, `Trackpad` |
| `PointerId` | `PointerId::MOUSE` is `0`; `PointerId::touch(contact)` namespaces a contact so contact `0` is never the mouse |

### Timestamps are not optional

`with_timestamp` carries **microseconds on a monotonic origin the host chooses**.
Multi-click detection, long-press and fling velocity are all computed from it, so
a host that leaves it at zero gets: no double-clicks, no long presses, no
inertia. This was a real defect here — the multi-click window was vacuously true
because the runner never stamped events.

The runtime never reads a clock. Time is data, always.

### Trust

`EventTrust` distinguishes a real device event from a synthesised one.
Trust-gated default actions (§2.3) exist so injected input cannot silently
perform privileged actions. `as_trusted()` is the only way to mark an event
trusted, and a host is what decides.

## Touch and gestures

Multi-pointer is real: several contacts are tracked independently, each with its
own press, capture, pan and hold state. Resting a thumb on one control while
pressing another with the other hand works.

Recognised in the runtime, with these thresholds:

| Gesture | Constant | Value |
|---|---|---|
| Pan starts after | `TOUCH_SLOP` | 10 px |
| Velocity sample rest | `PAN_REST_MICROS` | 100 ms |
| Fling decay | `FLING_TAU` | 0.325 s |
| Fling stops below | `FLING_REST_SPEED` | 20 px/s |
| Fling launches above | `FLING_MIN_LAUNCH_SPEED` | 50 px/s |
| Long press after | `LONG_PRESS_SECONDS` | 0.5 s |

A long press fires `EVENT_LONG_PRESS`; bind it like any other event. Fling and
pan drive scroll containers and virtual lists directly — an app does not
implement either.

The domain tier also has `GestureArena`, `GestureContender`, `GestureRecognizer`,
`PanRecognizer`, `LongPressRecognizer` and `Fling` as reusable pieces.

> **partial** — the arena is built and the runtime resolves pan / long-press /
> fling through its own state rather than through the arena. A custom recognizer
> has no registration point on `UiRuntime`.

## Pointer capture

```rust
rt.capture_pointer(pointer, node);   // every later event from it targets `node`
rt.release_pointer_capture(pointer);
rt.pointer_capture(pointer);
```

Capture is per-pointer, which is what makes it correct under multi-touch: one
finger dragging a slider does not steal the other finger's events.

## Keyboard

```rust
rt.handle_key(KeyEvent::press(PhysicalKey::KeyA, Some(LogicalKey::character('a'))));
```

`KeyEvent` carries both a **physical** key (layout-independent, what modifier
folding reads) and a **logical** key (what the layout produced, what text entry
reads). `LogicalKey` is a character or a `NamedKey`; `Modifiers` is folded from
the physical transitions `handle_key` has seen and readable with
`held_modifiers()`.

A key press routes to the focused node, then resolves against the shortcut
bindings:

```rust
rt.bind_command(Shortcut::new(LogicalKey::character('s')).with(Modifier::Ctrl),
                InternedSymbol::named("save"));
rt.on_command(InternedSymbol::named("save"), || save());
```

> **Tab does not move focus.** There is no tab-order traversal on the key path;
> `TabOrder`, `FocusTree`, `SpatialNavigation` and `NavigationDirection` exist in
> the domain tier with no runtime caller. Focus moves on a pointer press.

## Scrolling

`handle_scroll(delta_y, position)` routes a wheel notch to the innermost
`overflow: scroll` container under the point and returns which node took it, or
`None`. An app that windows its own list registers `App::on_scroll` for the
leftovers.

An `overflow: hidden` box clips and scrolls but paints no scrollbar; the wheel
passes through it to whatever is beneath.

## IME and composition

`handle_ime(CompositionEvent)` applies a composition against the focused
editable. `composition_preedit()` reads the live preedit.

**A host should offer text entry only while a field has focus.** Ask
`focus_accepts_text()` and mirror the answer onto the platform — on a desktop
that arms the IME, on a tablet it is what decides whether the on-screen keyboard
covers half the screen. `App` does this after every window event.

## Drag and drop

`register_drop_target`, `begin_drag`, `drag_session`, `take_drag_outcome` — see
[03 — The runtime](03-runtime.md#drag-and-drop). Data transfer types live in
`obscur_kit::data_transfer`.

## Not built

- **Gamepad.** `GamepadEvent`, `GamepadButton`, `GamepadAxis`, `GamepadId` are
  modelled; nothing produces one.
- **Haptics.** `HapticRequest` and `HapticPattern` are modelled; nothing consumes
  one.
- **Keyboard focus traversal.** As above.
