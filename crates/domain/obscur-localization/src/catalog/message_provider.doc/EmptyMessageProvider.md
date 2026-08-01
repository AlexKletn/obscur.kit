A [`MessageProvider`] holding no messages.

The honest default for a [`Localizer`](crate::catalog::Localizer) nobody has
given a catalogue to: it reports [`SupportLevel::Unavailable`] and answers
`None` for everything, so a localizer over it renders every key as itself. That
is a documented reference implementation of the contract — visibly untranslated
beats invisibly English.
