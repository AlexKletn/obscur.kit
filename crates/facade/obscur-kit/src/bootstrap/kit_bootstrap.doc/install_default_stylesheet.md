Installs the built-in UA default stylesheet on `runtime` as its always-first
(lowest-origin) sheet, so bare built-in controls get the toolkit's base look
automatically.

This is purely additive and **overridable**: the UA sheet is matched below
every author/theme sheet the app later installs
([`set_stylesheets`](crate::runtime::UiRuntime::set_stylesheets) / the
`&[Stylesheet]` render paths), so any author rule of the same or higher
specificity wins over it (§4.14). Not calling this leaves the runtime with no
UA styling (byte-identical to before).
