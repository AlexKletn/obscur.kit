The automatic-capitalization policy applied as the user types (spec §5.12).

This mirrors the on-screen-keyboard autocapitalization hints: whether to
capitalize nothing, the first letter of each sentence, the first letter of
each word, or every character. Its own [`Default`] is
[`None`](Self::None); [`InputHints`] overrides that with
[`Sentences`](Self::Sentences) for free-form fields.
