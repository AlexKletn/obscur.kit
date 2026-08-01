Offers the platform's text-entry facility to the focused element, and takes
it back when focus leaves it (spec §5.12; §6.15 on a touch device).

On a desktop this arms the IME. On a tablet it is what raises and drops the
**on-screen keyboard**, so getting it wrong is not a subtlety: a window that
claims text entry for its whole life keeps a keyboard over an app with
nothing to type into, which is what the calculator did on a real tablet.

The runtime owns focus, so it answers *whether* — this only mirrors the
answer onto the window, and only when it changed, because
`set_ime_allowed` is a round trip to the compositor and focus is
recomputed far more often than it moves.

Called after every window event rather than from the places focus can
change: focus moves on a click, a Tab, a key that dismisses a field, and
from an app's own command handler during a frame. Enumerating those is a
list that goes stale; asking once per event cannot.
