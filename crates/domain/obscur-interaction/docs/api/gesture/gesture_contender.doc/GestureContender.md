A recognizer entered into a [`GestureArena`](super::GestureArena), bound to
a node with a priority (spec §6.16).

The contender pairs a [`RecognizerId`] with the [`NodeId`] it is attached to
and a `priority`. Higher priority is preferred when the arena resolves; the
runtime gives system gestures (semantic back, edge swipes) a high priority
so they win ties against app recognizers.
