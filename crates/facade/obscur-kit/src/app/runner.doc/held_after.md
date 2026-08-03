Folds one winit button transition into the held set and returns the result.

The set the event carries is the one held **after** the transition, which is
what `PointerEvent::buttons` means: a press includes the button that caused it,
a release excludes it.
