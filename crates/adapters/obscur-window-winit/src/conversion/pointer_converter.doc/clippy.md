Assembles a trusted pointer event of `kind` from `device` at `position`
with no held buttons — the body [`event`](Self::event) and
[`touch`](Self::touch) share.

The device is a parameter rather than a constant because trust is decided
here, once, for everything the compositor delivered: a finger is exactly as
real a gesture as a click, and routing touch around this function to keep
trust mouse-only would have re-created the disjoint sets described below.

# Where trust enters the runtime

Every event built here descends from a `winit::event::WindowEvent` the
compositor delivered for an actual device, so this converter — and, in this
workspace, *only* this converter — marks its output
[`Trusted`](obscur_interaction::event::EventTrust::Trusted) (§2.3). The
other two producers of a `PointerEvent` are synthetic by definition
(§7.6's DevTools `Inject`, §7.4's headless harness) and leave the default
untrusted standing, so an application can tell a real click from a driven
one — but only for the kinds this converter can actually build. That is why
[`pressed`](Self::pressed) and [`released`](Self::released) exist: while the
builders were move/enter/leave only, the trusted set and the injectable set
(`PointerDown`/`PointerUp`) were **disjoint**, and the flag distinguished
nothing on the one gesture an attacker would forge. Key events still have no
converter here at all, so a real `KeyDown` remains untrusted.
