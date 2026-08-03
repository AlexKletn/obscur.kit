Replaces the runtime's idea of which modifiers are held.

The correction channel a platform adapter needs. The runtime otherwise folds the
chord from the modifier transitions it has seen, which is the only thing a host
that sends nothing but key events can offer — and it goes wrong the moment a
transition happens while the window is not focused. A user who alt-tabs away,
releases Alt, and comes back leaves the runtime holding an Alt that is not down.

winit reports this as `WindowEvent::ModifiersChanged`; every platform has an
equivalent, because every platform has the same problem.
