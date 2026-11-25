Routes `command` down §2.6's chain from the **focused** node and, when a
link claimed it, requests a [`FrameReason::Input`] frame — a handler
likely mutated app state, so the next `run_due_frame`/`render_now`
rebuilds from it. Returns whether a handler actually ran.

This is the chain's keyboard/semantic-command entry point: the origin is
[`focused`](Self::focused), exactly as §2.6 specifies. A *pointer*
activation instead originates at the node the user clicked, so the
internal activation path dispatches from *that* node — routing a click
from wherever focus happened to be would be wrong.

With no link claiming the command this is a no-op that returns `false`
and schedules no frame, so an unbound command (or one whose handler the
app never registered) can never panic or perturb runtime state.

A command **registered** in the [`command_registry`](Self::command_registry)
whose [`CommandState`] is not actionable (disabled or hidden) is inert:
no link is offered it and no frame is requested, whichever surface
asked — a keyboard shortcut, a menu item, or a pointer click all funnel
through here, so one state governs every surface (spec §6.7). A symbol
with no registered command is unconstrained, so applications routing raw
symbols without a descriptor are unaffected.

This is also the surface a **menu item or a toolbar button** dispatches
through: §2.6's input layers end in "semantic command", and a command
invoked from a menu has no device event behind it, so the event that
propagates reports [`EventOrigin::Synthetic`] and an empty payload rather
than a fabricated click.

[`FrameReason::Input`]: crate::frame::FrameReason::Input
[`EventOrigin::Synthetic`]: obscur_interaction::event::EventOrigin::Synthetic
