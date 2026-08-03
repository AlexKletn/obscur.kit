The pointer's hover chain newly contains this node — `on:pointerenter="…"`.

Emitted from the hover transition a pointer event produces, **not** from a
`PointerEventKind::Enter`. Every node
the chain newly holds gets one, so a container and the row inside it each hear
their own entry, and the node this event names is the node that was entered —
not the hit target the move landed on.

# What the pair means here, and what it does not

The DOM has two pairs: `pointerenter`/`pointerleave`, which do not bubble and
fire once per crossed element, and `pointerover`/`pointerout`, which do. This
kit dispatches a **command**, and a command routes — §2.3's capture / target /
bubble and then §2.6's chain — so "does not bubble" is not a property this
toolkit can give an event. Only the enter/leave pair is reserved, and it is
half of each DOM pair:

- Like `pointerenter`, the **binding** does not bubble. The command is resolved
  on the crossed node's own `on:pointerenter` and nowhere else — no ancestor
  walk. That is what makes one crossing one event: resolving through ancestors
  would dispatch a container's binding once per descendant that was entered,
  which is N deliveries for one boundary crossing.
- Unlike `pointerenter`, the **command** it names propagates like every other
  command in this toolkit. A listener registered on an ancestor with
  `on_command_at` still sees it in the bubble phase.

No `pointerover` / `pointerout` exists. Reserving a bubbling spelling that the
runtime would emit identically would be two names for one event.

# Order

Within one pointer event: every `pointerleave` first, innermost-first; then
every `pointerenter`, **outermost-first**. A container is entered before the row
inside it and left after it, so the two sequences nest — the same discipline a
stack unwinds by, and the same order the DOM fires its crossings in. A handler
that opens a menu on entering a container and highlights a row inside it can
therefore rely on the container having been entered first.

# `prevent_default` has nothing to suppress here

Deliberate, and the honest finding rather than an invented symmetry. A hover
transition is a report of where the pointer now is; the runtime performs no
default action in response to one. The chain is already seated by the time this
is dispatched, and the restyle that follows is not the crossing's default action
— it *is* the crossing, stated in CSS. Refusing it would mean asking for a
`:hover` that does not match a pointer that is over the node, which is not a
choice this event is in a position to offer. There is nothing left to decline,
so a refusal is inert.

# Touch

§2.4: touch does not synthesize persistent hover. A finger down enters the chain
and the lift leaves it, so a tap emits `pointerenter` and then `pointerleave`
within the one contact, and nothing stays entered afterwards. A pen genuinely
hovers, so it enters on approach and stays.

# Cost

A crossing is dispatched only when the chain actually changed, and — like the
raw pointer names — nothing is resolved or dispatched at all unless some node in
the committed tree binds this name.
