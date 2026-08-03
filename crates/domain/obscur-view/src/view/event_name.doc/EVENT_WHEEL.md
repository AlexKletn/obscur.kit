A wheel notch was aimed at this node — `on:wheel="…"`.

Dispatched from `UiRuntime::handle_scroll` to the node under the point, through
§2.3's capture / target / bubble like any other event, and **before the scroll
it can refuse**.

# The payload is neither a pointer nor a key

A notch is a delta and a position, so it carries `EventPayload::Wheel` — read it
with `event.wheel()`, and `delta_y()` is the logical pixels the container would
be scrolled by, positive toward the content's end. Synthesising a
`PointerEventKind::Move` to carry it was refused: a move that never happened is
a report the model cannot then tell from a real one.

The event is **untrusted and unstamped**. `handle_scroll` takes two floats, so
there is no converter at the door to confer either — `UiEvent::from_wheel` says
why.

# `prevent_default` suppresses the scroll

The one default action a wheel has: refusing it means the container under the
point is not scrolled, and no frame is requested for a scroll that did not
happen. That is what makes a custom zoom, a horizontal carousel, or a canvas
that consumes the wheel itself expressible — each of them needs the notch and
needs the page under it to stay still.

A refusal is also a **claim**. `handle_scroll` returns the node that took the
notch, and a refused wheel returns the node whose binding refused it — so the
host's own fallback (`App::on_scroll`, for an application that windows its own
list) does not then run as though nothing had claimed the event. A listener that
refuses the scroll has taken it, not declined it.

# What it does not reach

The node it targets is the node **under the point** — the hit test, the same one
a click would find. It is not the scroll container that would have taken the
notch, though that container is usually an ancestor of it, and the ancestor walk
that resolves the binding is what lets a listener on the container hear a notch
aimed at a row inside it.

A refusal does not stop a **fling** already in flight, or a pan the finger is
driving. Neither is a wheel; both are the gesture recognizers' business, and
this event is never dispatched for either.

# Cost

A trackpad reports pixel deltas at very nearly the rate it reports moves.
Nothing is walked, hit-tested, allocated or dispatched unless some node in the
committed tree actually binds this name; the runtime keeps that answer as a bit
recomputed once per structural frame, in the same set the raw pointer names use.
