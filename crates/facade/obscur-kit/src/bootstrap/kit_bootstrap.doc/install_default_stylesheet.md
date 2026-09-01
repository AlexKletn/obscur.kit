Installs the built-in UA default stylesheet on `runtime` as its always-first
(lowest-origin) sheet, so bare built-in controls get the toolkit's base look
automatically.

This is purely additive and **overridable**: the UA sheet is matched below
every author/theme sheet the app later installs
([`set_stylesheets`](crate::runtime::UiRuntime::set_stylesheets) / the
`&[Stylesheet]` render paths), so any author rule of the same or higher
specificity wins over it (§4.14). Not calling this leaves the runtime with no
UA styling (byte-identical to before).

It also arms the runtime's custom-property grammar, if nothing has yet — see
[`install_custom_value_parser`](super::CssPipeline) for the funnel and the
reason it is drawn at "the bootstrap installed CSS" rather than at "the
bootstrap touched a runtime". A host that installs the UA sheet and then hands
`set_stylesheets` its own hand-compiled `:root { --brand: … }` would otherwise
read every one of those variables back as `unset`. An existing parser is left
alone.
