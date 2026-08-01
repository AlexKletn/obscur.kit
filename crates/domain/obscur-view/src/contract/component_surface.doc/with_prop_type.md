Records the declared type category of prop `name` (fluent; spec §2.8).

`#[derive(Contract)]` emits one call per prop, so the `.okui` compiler can
reject a literal of the wrong kind. A prop with no recorded type — or one
recorded as [`PropTypeSurface::Other`] — is not type-checked.
