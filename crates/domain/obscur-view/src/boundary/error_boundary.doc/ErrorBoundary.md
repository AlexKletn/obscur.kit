A component's error boundary (spec §1.7).

This is the mechanism the runtime consults when it detects a contained
failure. It pairs a [`ErrorBoundaryPolicy`] with the current
[`ComponentFailure`] (if any) and an optional fallback [`View`] to show in
place of the failed subtree.

The core rule is *contain vs. propagate*: when the policy is enabled a
recorded failure is contained here — the subtree is replaced by the fallback
UI. When the policy is disabled the failure is still recorded (so it is
reported to diagnostics), but [`record`](Self::record) signals that this
boundary does not contain it, and the runtime must propagate it to the
nearest enclosing enabled boundary.

There is no reactive engine and no panic-catching here: the runtime feeds
failures in and reads the containment decision and fallback back out.
