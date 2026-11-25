An event binding on an IR node: an event name wired to the handler or
command that fires *up* when it occurs (spec §2.5).

The `event` is the triggering name (for example `"click"`); the `handler`
is the command or action name that flows up to the host when the event
fires. Both are plain names, so this type is fully hashable and derives
`Eq`/`Hash`.
