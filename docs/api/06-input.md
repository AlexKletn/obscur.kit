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
pub const fn with_button(self, button: PointerButton) -> Self
pub fn with_pressure(self, pressure: f32) -> Self
pub const fn with_timestamp(self, timestamp_micros: u64) -> Self
pub const fn as_trusted(self) -> Self
```

Readers: `pointer()`, `device()`, `kind()`, `position()`, `delta()`,
`buttons()`, `button()`, `pressure()`, `tilt()`, `rotation()`, `is_primary()`,
`hover_capable()`, `timestamp()`, `is_pressed()`, `trust()`, `is_trusted()` —
which is every field §2.4 names, plus the one it does not.

| | |
|---|---|
| `PointerEventKind` | `Down`, `Up`, `Move`, `Enter`, `Leave`, `Cancel` — `is_terminal()` covers the last two ways a stream can end |
| `PointerDevice` | `Mouse`, `Touch`, `Pen`, `Trackpad` |
| `PointerId` | `PointerId::MOUSE` is `0`; `PointerId::touch(contact)` namespaces a contact so contact `0` is never the mouse |
| `PointerButton` | `Primary`, `Secondary`, `Middle`, `Back`, `Forward` |

### Buttons: the set and the one that moved

`buttons()` is the set held **after** the event; `button()` is the one whose
transition the event reports, or `None` when none did (a move, a crossing, a
finger). They are the DOM's `buttons` and `button`, and they are two facts
because a set cannot say which member changed: pressing the secondary button
while the primary is down reports `{Primary, Secondary}`, and so does the
opposite chord.

The winit runner folds winit's one-transition-per-event reports into a running
set, so a chord is a real state rather than whichever button moved last. All
five buttons are delivered; `MouseButton::Other(_)` is dropped rather than
folded onto a modelled neighbour.

**Only the primary button engages a control.** A secondary or middle press moves
focus and stops there: no `:active`, no caret, no drag-selection, no armed pan or
long press, and no activation on release. That is what stops a right click
running an `on:click`, and it is why a secondary release cannot consume the press
the primary button is still holding. An event that names no button is read as the
primary one, which is what every producer that predates `button()` meant — a
touch contact included.

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

### The raw pointer events

```rust
<row on:pointerdown="grab" on:pointermove="track" on:pointerup="drop"
     on:pointercancel="abandon"/>
```

`pointerdown`, `pointerup`, `pointermove` and `pointercancel` are reserved event
names dispatched to the **hit target** — capture-else-hit-test, the same node
hover and focus are computed from — through §2.3's capture / target / bubble
like any other. Each is dispatched **before** the runtime does its own thing
with that event, so `prevent_default()` reaches the runtime's own default
action and not only §2.6's routing chain:

| refusing | suppresses |
|---|---|
| `pointerdown` | the focus move, the caret placement, the press anchor, gesture arming — and therefore any `click` that would have followed. All five buttons dispatch one; what a refusal reaches on a non-primary press is only the focus move, because that is all such a press does |
| `pointerup` | the click, the `on:click`/`on:change` activation it dispatches, the virtualized row click |
| `pointermove` | the live drag-selection |
| `pointercancel` | **nothing.** See below |

Three things a refusal deliberately does **not** reach:

- **A pan already scrolling.** The gesture owns the pointer through capture
  before the event is delivered, and the thresholds above are what decide it. A
  refusal mid-scroll would leave the container half-scrolled with the finger
  still down.
- **The bookkeeping a release owes.** A refused `pointerup` still consumes the
  press, releases capture, completes the pointer and throws a travelling pan.
  Those are not actions to decline; they are the fact that the finger lifted.
- **A cancel, at all.** What follows a `pointercancel` is the unwinding of state
  the system has already destroyed. Honouring a refusal would leave an `:active`
  that never clears and a capture nothing releases, for a contact that no longer
  exists. The event reports the loss; it does not offer to decline it.

A raw event during a drag-and-drop in flight **is** delivered — an application
drawing its own drag affordance is exactly when it wants the moves — and
refusing one costs nothing, because the drag has already taken over every
default action a refusal could suppress.

> **Cost.** `pointermove` fires at the pointer's report rate. Nothing is walked,
> allocated or dispatched unless some node in the committed tree actually binds
> the name; the runtime keeps that answer as seven bits recomputed once per
> structural frame. Binding `pointerdown` does not make moves expensive.

### The crossings

```rust
<row on:pointerenter="highlight" on:pointerleave="dim"/>
```

`pointerenter` and `pointerleave` name **a node the hover chain crossed**, which
is not the node the event targeted: one move from a row to its sibling leaves the
first and enters the second while targeting only the second. Every node the chain
gained or dropped gets its own event, so a list and the row inside it each hear
their own.

Order, within one pointer event: **every leave first, innermost-first; then every
enter, outermost-first.** The two sequences nest, so a container is entered
before the row in it and left after it, and a teardown on leave is never
interleaved with the next node's setup. That is the order the DOM fires its
crossings in.

Two honest limits, neither of them DOM parity:

- **The binding does not bubble; the command does.** The command is resolved on
  the crossed node's own `on:pointerenter` and nowhere else — an ancestor walk
  would deliver a container's binding once per descendant that crossed with it.
  But what it resolves *is* a command, and a command routes: an `on_command_at`
  listener on an ancestor still sees it in the bubble phase. There is no way to
  make an event here not propagate, so "does not bubble" is only half true and is
  written down as half true.
- **There is no `pointerover` / `pointerout`.** The DOM's second, bubbling pair
  would be emitted identically here, so it would be a second name for the same
  event. `on:pointerover` hashes like any other author name and nothing
  dispatches it.

`prevent_default()` on a crossing **suppresses nothing, deliberately.** A hover
transition is a report of where the pointer now is: the chain is already seated
when the event is dispatched, and the restyle that follows is not the crossing's
default action — it is the crossing, stated in CSS. There was no default action
to gate, so none was invented.

Touch obeys §2.4's "touch does not synthesize persistent hover": a tap emits
`pointerenter` on contact and `pointerleave` on lift, so nothing stays entered
for a finger that is gone. A pen genuinely hovers and stays.

Same cost gate as the four above, and the same bits.

**Leaving the window is a crossing too.** A `PointerEventKind::Leave` empties the
hover chain — nothing is under a pointer that is gone, whatever the hit test says
about the last position it reported — so every node in the chain hears its
`pointerleave` and `:hover` clears. The winit runner converts `CursorLeft` into
one, which it did not before: the row the cursor exited by stayed highlighted for
the rest of the session.

`CursorEntered` is converted too, but **one event later**. winit reports the
entry without a position and follows it with a `CursorMoved` that has one, so the
runner remembers the entry and promotes that move to the `Enter` instead.
Synthesising the crossing on the spot would have to reuse the position the
pointer *left* by, entering a node it never crossed and leaving it again one event
later.

### The context request

```rust
<row on:contextmenu="showMenu"/>
```

`contextmenu` fires on a **secondary-button press** and on nothing else — there
is no context-menu key on the keyboard path and no long-press route to it, and
inventing either would be a second emitter for a name that has one. It is
dispatched to the hit node through §2.3's capture / target / bubble, after
`pointerdown` and before the press does anything.

**Refusing it suppresses the focus move**, which is the whole of what a secondary
press performs — see the button rules above. So the refusal is stated in full
rather than gestured at: there is nothing else a secondary press does for a
listener to decline.

Refusing `pointerdown` suppresses the same move one event earlier, but the two
are not alternatives: `contextmenu` is still dispatched after a refused
`pointerdown`, because declining a focus move is not declining the request for a
menu.

The runtime opens no menu. This names the request; drawing the menu is the
application's, which is why there is no `menu` name meaning "a menu opened".

> **Cost.** Not behind the raw names' binding gate, deliberately. That gate
> exists for the report rate — a move arrives hundreds of times a second — and a
> right click arrives at the rate a hand can click, so the ancestor walk is once
> per press. A bit spent here would be the eighth and last in the word.

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
reads). `LogicalKey` is a character or a `NamedKey`.

It is self-describing, the way `PointerEvent` is: `modifiers()` is the chord it
was pressed under, `is_repeat()` tells an auto-repeat from a new press,
`timestamp()` is the host clock, and `trust()` fails closed. `as_trusted()` is
conferred at the platform boundary and nowhere else — `handle_key` no longer
confers it, so a synthesised key stays untrusted.

`Modifiers` is folded from the physical transitions `handle_key` has seen and
readable with `held_modifiers()`. A platform that knows better says so with
`set_modifiers` — winit's `ModifiersChanged`. Without it the fold goes stale the
moment a modifier is released while the window is unfocused.

### The raw key events

```rust
<input on:keydown="handleKey" on:keyup="released"/>
```

`keydown` and `keyup` are reserved event names dispatched to the focused node,
through §2.3's capture / target / bubble like any other. `keydown` runs **before**
the runtime does anything with the key, so a handler calling `prevent_default()`
suppresses the text edit, the shortcut and the activation that would otherwise
follow. That ordering is the point: a `keydown` delivered afterwards could report
a key but never refuse it.

> The full reserved set is `RESERVED_EVENTS`. Any other `on:` name hashes to a
> symbol nothing dispatches — `is_reserved_event` is what tells the two apart.

A key press routes to the focused node, then resolves against the shortcut
bindings — **with or without a focused node**, since §2.6's route ends at the
window surface and the application:

```rust
rt.bind_command(Shortcut::new(LogicalKey::character('s')).with(Modifier::Ctrl),
                InternedSymbol::named("save"));
rt.on_command(InternedSymbol::named("save"), || save());
```

### Tab moves focus

~~**Tab does not move focus.** There is no tab-order traversal on the key path;
`TabOrder`, `FocusTree`, `SpatialNavigation` and `NavigationDirection` exist in
the domain tier with no runtime caller.~~ **Closed.** `TabOrder` has a caller:
Tab and Shift+Tab walk the controls, wrapping in both directions, and a Tab with
nothing focused is how focus *enters* the surface.

The ring is **not** the set a pointer can focus, and the difference is
deliberate. A press focuses whatever it hit — a plain box, the text leaf inside
a field — which is what a user expects from a click and what `spec_ch2_interaction`
pins. Tab visits only nodes whose **semantic role is interactive**, skipping
disabled ones. That is the same answer the accessibility projection publishes: a
control a screen reader is told it can focus and a control Tab refuses to reach
would be two truths about one node.

Order is document order. An authored `tabindex` is not modelled — nothing in the
view model carries one — so every stop sorts equal. `FocusNode` and `TabOrder`
already understand a tab index; giving them one here would mean inventing an
attribute no author can write.

Tab is a **default action of `keydown`**, so a handler that calls
`prevent_default()` keeps it: that is how a grid or a code editor takes Tab for
itself. A chord an application *bound* wins outright — doing both would move
focus and run a command for one press. An empty ring moves nothing rather than
blurring: "nowhere to go" must not become "nowhere".

An open `FocusScope` with `trapping(true)` narrows the ring to its own members,
which is what stops Tab wandering out of a modal.

```rust
rt.focus_node(Some(node));   // programmatic; `None` clears
rt.is_focus_visible();       // whether `:focus-visible` currently matches
```

**`:focus-visible` means keyboard.** A Tab shows the ring; a press does not; a
programmatic move does not, because it is not the user navigating. The selector
was modelled and set by nothing before this, so the rule matched nothing.

> **Still not built:** spatial (arrow-key) navigation. `SpatialNavigation` and
> `NavigationDirection` remain domain types with no runtime caller — arrow keys
> belong to the editing session, and taking them for navigation needs a rule
> about when a control is a *grid* that does not exist yet.

## Focus

```rust
<input on:focus="armHints" on:blur="validate"/>
```

`focus` and `blur` are reserved event names, emitted from the one place focus
changes. A move between two nodes fires **both, `blur` first**; a move with only
one end fires only that end — focusing the first control of a session fires
`focus` alone, and a press that lands on nothing clears focus and fires `blur`
alone. A press that does not move focus (re-pressing the focused node, or
pressing a **disabled** control, which is deliberately not a blur) fires
neither.

Both are dispatched **after** the move, so a handler asking what has focus reads
the answer its own event describes. By the same token:

> **Neither is a gate.** `prevent_default()` on `focus` or `blur` suppresses
> nothing, because focus has already moved when either is dispatched. No
> suppression was invented to make the story symmetrical.
>
> **The refusal is one event earlier.** `pointerdown` runs *before* the press
> moves focus, and refusing it suppresses the move — that is where a focus
> change is declined.

For an editable losing focus the order is **`change`, then `blur`, then the next
node's `focus`** — the order HTML fires them in, so a handler reading a field's
value on blur sees the committed one.

The binding resolves by walking the focused node's ancestors, first match wins —
unlike the DOM, where `focus` and `blur` do not bubble. The DOM focuses
*elements*; this runtime focuses whatever the press hit, and a press on a
field's glyphs hits the field's **text leaf**. Resolving on that node alone
would make `<input on:focus>` fire or not depending on whether the user clicked
the text or the padding beside it. The consequence is stated rather than hidden:
`focus` here behaves like the DOM's `focusin`, a panel hears a row inside it
take focus, and `event.target()` names the node the binding resolved on. There
is no separate `focusin` / `focusout`, because that is what these already are.

> **Only a pointer press moves focus today.** §2.5's focus tree, tab order and
> programmatic focus have no runtime path yet — see the note above. When one
> arrives it moves focus through the same door, so the pair fires without that
> path having to remember to fire it.

## Scrolling

`handle_scroll(delta_y, position)` routes a wheel notch to the innermost
`overflow: scroll` container under the point and returns which node took it, or
`None`. An app that windows its own list registers `App::on_scroll` for the
leftovers — so "which node took it" includes a node that took it by *refusing*
it; see below.

An `overflow: hidden` box clips and scrolls but paints no scrollbar; the wheel
passes through it to whatever is beneath.

### The wheel event

```rust
<canvas on:wheel="zoom"/>
```

`wheel` is a reserved event name dispatched to the **node under the point** —
the hit target, the same node a click there would land on — through §2.3's
capture / target / bubble, and **before the container moves**. That ordering is
the point: a wheel delivered after the scroll could report the notch but never
decline it.

Its payload is neither a pointer nor a key. A notch is a delta and a position,
and §2.4's pointer model describes a *contact*, so the envelope carries
`EventPayload::Wheel`:

```rust
rt.on_command(zoom, |e: &mut UiEvent| {
    if let Some(wheel) = e.wheel() {
        scale *= 1.0 + wheel.delta_y() / 500.0;   // logical px, + is downward
        e.prevent_default();                       // and the page stays still
    }
});
```

Synthesising a `PointerEventKind::Move` to carry it was refused: there is no
`Wheel` kind, so the notch would have arrived as a move that never happened, and
`PointerDelta` — "how far the pointer travelled" — would have meant something
else for that one event.

**Refusing it suppresses the scroll**, and the refusal is a *claim*.
`handle_scroll` reports which node took the notch, and `App::on_scroll` runs when
nothing did — so a refused wheel reports the **node that refused it**, never
`None`. The three answers are `Some(container)` scrolled it, `Some(node)` refused
it, `None` nobody took it. Anything else would scroll the host's own list out
from under a listener that had just said no.

The binding is resolved by walking the hit node's ancestors, so a listener on the
scrolling list hears a notch aimed at any row inside it without binding every
row.

Two limits, both deliberate:

- **The event is untrusted and unstamped.** `handle_scroll(delta_y, position)`
  takes two floats, so unlike `handle_pointer` and `handle_key` there is no
  self-describing event at the door and no converter to confer trust or a
  timestamp on. `EventTrust` fails closed rather than vouching for a notch the
  runtime cannot distinguish from a queued `InputEvent::Scroll`.
- **A fling or a pan is not a wheel.** Both move containers through the gesture
  recognizers and neither is dispatched here, so neither is refusable this way.

> **Cost.** A trackpad's two-finger scroll arrives as pixel deltas at nearly the
> rate its moves do, and the wheel path pays *more* per event than the move path
> because it hit-tests before it can resolve a binding. So it shares the raw
> pointer names' gate: nothing is mapped, hit-tested, walked or dispatched
> unless some node in the committed tree binds `wheel`.

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
- **Spatial navigation.** Arrow keys do not move focus. `SpatialNavigation` and
  `NavigationDirection` are modelled with no runtime caller: arrows belong to the
  editing session, and taking them for navigation needs a rule about when a
  control is a *grid* that this toolkit does not have.
- **`FocusTree` and `FocusRestore`.** Still no runtime caller. The Tab ring is
  computed from the retained tree on demand rather than from a registered focus
  tree, so `FocusTree`'s registration model answers a question nothing asks; and
  nothing saves or restores a focus position across a dialog yet.
- **The rest of the raw vocabulary.** The DOM's bubbling `pointerover` /
  `pointerout` are not reserved names, because a command routes either way here
  and they would be a second name for `pointerenter` / `pointerleave`. A
  `PointerEventKind::Enter` or `Leave` still dispatches no raw event *of its
  own*: the crossings it causes are reported through the hover transition like
  every other crossing, which is what keeps one boundary one event. There is no
  `scroll` event either — `wheel` fires before the scroll and can refuse it, and
  nothing reports that a scroll has happened.
- **A trusted wheel.** The notch a `wheel` handler receives is untrusted and
  unstamped, because `handle_scroll` takes two floats rather than a
  self-describing event. Closing it means a `WheelEvent` at the door and a
  converter to confer trust on it, the way `PointerConverter` does.
- **Window activation.** `WindowEvent::Focused` is deliberately ignored, and the
  concept it would need is the reason: `UiRuntime` models *node* focus, and the
  only door into it moves focus, fires `blur`/`focus` and commits the outgoing
  field's `change`. Sending a deactivated window through that door would clear a
  field's caret and commit it because the user alt-tabbed. Doing it right needs a
  window-activation state on the runtime and a `:window-inactive`-shaped selector
  reading it, so an inactive window can dim its selection and rest its caret
  without losing either. The one cost of ignoring it that would have bitten is
  already paid: a modifier released while the window was unfocused is corrected
  by `ModifiersChanged`.
- **A context menu.** `on:contextmenu` reports the request; nothing here draws or
  positions a menu, and `Menu` is a widget an application places itself.
