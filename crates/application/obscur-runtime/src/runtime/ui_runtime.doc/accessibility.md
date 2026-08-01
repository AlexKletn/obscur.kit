The §7.1 output path: the host-installed platform accessibility port, and
the focus that port was last told about.

One field rather than the two it replaces, because publishing reads both
in one operation and writes one of them — see [`AccessibilityPublisher`]
for why a push that fails to record what it published re-walks the whole
retained tree on every frame that follows it, silently.
