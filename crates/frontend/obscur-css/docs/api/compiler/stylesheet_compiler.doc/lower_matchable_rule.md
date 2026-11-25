Builds the runtime-matchable [`StyleRule`] for a parsed style rule: the
lowered selector list paired with a block of authored [`BlockDeclaration`]s.

The block is built once per rule (it is selector-independent). Motion
declarations are routed away as they are elsewhere, and only standard
longhands become block declarations; custom properties and unsupported
values are handled by the flat cascade path (their diagnostics are raised
there, so they are silently skipped here to avoid double reporting).
