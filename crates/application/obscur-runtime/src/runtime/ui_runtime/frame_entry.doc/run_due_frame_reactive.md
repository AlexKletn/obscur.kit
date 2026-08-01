Runs **one** reactive frame if the scheduler has a request pending: it calls
the runtime-held [`root_view`](Self::set_root_view) builder to REBUILD the
desired [`View`] from current application state, then runs the full
deterministic core (reconcile → style → layout → compose) over it with the
held [`stylesheets`](Self::set_stylesheets). Returns the [`FrameResult`], or
[`None`] when no builder is installed or nothing is due.

This is the keystone that closes the event loop: a pointer click runs a
registered command handler, the handler mutates app state and requests a
frame, and *this* method — on the next due frame — rebuilds the view from
that mutated state and reconciles it into the retained tree, so the paint
reflects the new state. Unlike [`run_due_frame`](Self::run_due_frame), the
caller passes no view: the builder is the single source of truth.

A reactive frame is always classified as a **structural**
[`FrameReason::UiCommit`] regardless of which coalesced request the
scheduler drained, because the builder may have produced a *different* tree
from the mutated state and that change must be reconciled — an `Input`
(hover/focus) or `Animation` reason would skip the reconcile phase and never
pick up the rebuilt view. Reconcile is an idempotent no-op when the rebuilt
view is structurally unchanged, so a hover-only reactive frame still costs
only the (cheap) diff.

Borrow handling: the builder is a `&mut` field and the render is `&mut
self`, so the builder and the held stylesheets are *taken out* for the
duration of the render and restored afterward, keeping the `FnMut` call and
the `run_frame` borrow disjoint.
