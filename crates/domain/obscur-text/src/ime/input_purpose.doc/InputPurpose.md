The semantic purpose of a text field (spec §5.12).

The purpose is a hint that platform IMEs, on-screen keyboards, and autofill
use to tailor input: a [`Password`](Self::Password) field suppresses
learning and suggestions, a [`Number`](Self::Number) field offers a numeric
keypad, an [`Email`](Self::Email) field surfaces `@`, and so on. It carries
no validation semantics of its own — it only describes intent.

[`is_sensitive`](Self::is_sensitive) reports whether the purpose implies
sensitive-mode handling (per §5.12 "sensitive/password modes are
respected").
