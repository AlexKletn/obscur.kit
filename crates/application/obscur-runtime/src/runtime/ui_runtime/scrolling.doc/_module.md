Per-element scroll containers (CSS Overflow Module Level 3): the host-facing
read and write of a container's `scroll_top`, the wheel gesture that routes to
one, and the §6.15 reveal that scrolls a focused field out from behind the
keyboard.

Every one of them ends in the same two calls — resolve the container from the
current layout with
[`scroll_containers`](crate::orchestration::scroll_containers), then move it
through [`ScrollOffsets`](crate::runtime::ScrollOffsets) — and that is why they
are one file rather than four. The container set is derived from the boxes on
every call rather than cached, so a node that stopped overflowing since the last
frame is simply not found and the write is a no-op; and the offset write refolds
the viewport-relative box projection in the same call, so a caller reading a
node's geometry between frames never sees it at the position it was painted at
before the scroll.

The wheel path adds two things the programmatic path does not need. The first is
the `wheel` event: a notch is dispatched to the node under the point before any
container moves, so a listener can refuse the scroll — which is the difference
between a wheel and `set_scroll_top`, since a host calling the latter is not
asking anyone's permission.

The second: `hidden`
is a scroll container in the CSS sense — it clips, and it can be scrolled by
[`set_scroll_top`](UiRuntime::set_scroll_top) — but the wheel must pass through
it, or a row that clips a long name eats the scroll meant for the list it sits
in. [`ScrollTargets`](crate::runtime::ScrollTargets) is what draws that
distinction, and it draws it for the reveal too.
