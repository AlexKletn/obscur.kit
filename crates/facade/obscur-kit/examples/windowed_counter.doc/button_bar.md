Builds a real [`Button`] bound to `command` and rendered as a sizing bar.

The sizing `class` is placed on the **button's own element node** (not a
wrapping container) for a reason: the runtime resolves a click by bubbling
from the hit node up its *ancestors* to the first `EVENT_CLICK` binding. If
the button were wrapped in the sized container, the pointer would hit the
container and the button (a descendant) would never be on that ancestor
chain, so the click would resolve to nothing. Sizing the button node itself
makes it the hit target, so its own `EVENT_CLICK → command` binding fires.
