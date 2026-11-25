The bundle of input hints for a text field (spec §5.12).

These hints steer platform IME and on-screen-keyboard behaviour without
dictating it: the [`purpose`](Self::purpose) names the field's semantics,
while the remaining flags toggle [`autocorrect`](Self::autocorrect),
[`spellcheck`](Self::spellcheck), automatic [`capitalization`](Self::capitalization),
whether the field is [`multiline`](Self::multiline), whether it is treated as
[`sensitive`](Self::sensitive), and an optional
[`preferred_language`](Self::preferred_language) (a BCP-47 tag).

# Defaults

[`InputHints::default`] / [`new`](Self::new) describe a free-form single-line
field: [`FreeForm`](InputPurpose::FreeForm) purpose, single-line, autocorrect
on, [`Sentences`](Capitalization::Sentences) capitalization, spellcheck on,
not sensitive, no preferred language. Note that the default capitalization is
[`Sentences`](Capitalization::Sentences) even though [`Capitalization`]'s own
[`Default`] is [`None`](Capitalization::None); `InputHints` sets its
capitalization default explicitly rather than deriving it.

[`effective_sensitive`](Self::effective_sensitive) folds the explicit
[`sensitive`](Self::sensitive) flag together with a sensitive
[`purpose`](Self::purpose), so a [`Password`](InputPurpose::Password) field is
sensitive even if the flag was never set (§5.12 sensitive/password modes are
respected).
