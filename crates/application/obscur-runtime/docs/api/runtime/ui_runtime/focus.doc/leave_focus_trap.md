Closes a trap whose element has left the tree, and restores what it displaced.

Focus is put back only when the trap actually held it — the node it names is
inside the closing scope, or has gone with it. Anything else means focus had
already moved somewhere the trap did not own, and a restore would be taking it
away from wherever it legitimately is.

The saved node is checked against the live tree before it is restored: the
control that opened a dialog can perfectly well have been removed by the same
re-render that removed the dialog, and focusing a dead id is the defect
[`drop_focus_if_removed`](UiRuntime::drop_focus_if_removed) exists to prevent.
