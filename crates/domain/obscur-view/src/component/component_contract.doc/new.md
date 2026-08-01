Creates an empty-but-valid contract for `id`.

It exposes no props/events/slots/style-parts, requires no capabilities,
accepts no children ([`ChildContract::None`]), has no name, and is
attributed to [`DiagnosticSubsystem::View`]. Callers refine it with the
`with_*`/`allowing_children` builders.
