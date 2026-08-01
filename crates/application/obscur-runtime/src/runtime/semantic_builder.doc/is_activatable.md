Whether `node`'s own bindings make it activatable, mirroring the runtime's
`UiRuntime::activation_command_for`: a bound [`EVENT_CLICK`] activates,
and so does a bound [`EVENT_CHANGE`] — but only on a **non-editable**
control, since a checkbox's click *is* its change while a text field's
`change` fires on commit (blur / Enter), not on the click.
