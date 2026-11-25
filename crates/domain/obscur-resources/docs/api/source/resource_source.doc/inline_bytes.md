Returns the bytes already in hand, if any.

[`ResourceSource::Embedded`] and [`ResourceSource::Memory`] carry their
bytes directly, so this returns `Some`. The file, URL, document, and
generated kinds must be resolved before their bytes exist, so this returns
`None`.
