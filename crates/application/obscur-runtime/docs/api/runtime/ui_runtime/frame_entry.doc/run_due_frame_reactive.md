Runs one reactive frame when a root view is installed and a frame is due.
For frames that may change application state it calls the
[`root_view`](Self::set_root_view) builder and reconciles the result using
the retained [`stylesheets`](Self::set_stylesheets). Returns the
[`FrameResult`], or [`None`] when no builder is installed or nothing is due.
A queued timeline continuation becomes due after a positive
[`Self::advance_motion`] tick.

This is the keystone that closes the event loop: a pointer click runs a
registered command handler, the handler mutates app state and requests a
frame, and *this* method — on the next due frame — rebuilds the view from
that mutated state and reconciles it into the retained tree, so the paint
reflects the new state. Unlike [`run_due_frame`](Self::run_due_frame), the
caller passes no view: the builder is the single source of truth.

An isolated runtime-owned timeline tick uses `FrameReason::Animation` and
reuses the retained tree. Explicit animation requests and frames coalesced
with other work keep the reactive builder path. The builder is available if
work discovered during dispatch or resource collection requires it.
A diagnostics-only request reuses the retained scene once the first frame
has been built.

Borrow handling: the builder is a `&mut` field and the render is `&mut
self`, so the builder and the held stylesheets are *taken out* for the
duration of the render and restored afterward, keeping the `FnMut` call and
the `run_frame` borrow disjoint.
