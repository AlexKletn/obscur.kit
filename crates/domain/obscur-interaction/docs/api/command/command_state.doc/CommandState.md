The dynamic runtime state of a command (spec §6.7).

A [`Command`](crate::command::Command) is a static descriptor; its live state
changes as the application runs. `CommandState` carries that mutable part:
whether the command is currently `enabled`, whether it is a toggle and if so
`checked`, and whether it is `visible`. The default state is a live command:
enabled, not checkable, and visible.
