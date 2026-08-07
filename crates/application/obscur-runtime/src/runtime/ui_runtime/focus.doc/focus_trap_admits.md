Whether an open trap would allow focus to land here.

`true` when no trap is open. While one is, only its own members are admitted —
and `None` is not, because "focus nothing" is how a press on the page behind a
modal would otherwise blur the field inside it. The one caller that must still be
able to clear focus is [`drop_focus_if_removed`](UiRuntime::drop_focus_if_removed),
which goes straight to [`move_focus_to`](UiRuntime::move_focus_to): a node that
has left the tree cannot stay focused whatever the trap says.
