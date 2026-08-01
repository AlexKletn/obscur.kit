An attribute's spelling as the author wrote it — `class`, or `on:click` for an
event-style one.

[`Attribute::name`] returns the *first* identifier, which for `on:click` is
`on`. Quoting `'on'` back at an author who wrote `on:click` is a message that
names a thing they cannot find, so the prefix is put back.
