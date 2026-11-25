A CSS custom property declaration (`--name: value`).

The `name` is the interned `--`-prefixed identifier. The `value` is one of two
things and nothing else: a [`PropertyValue::RawTokens`] holding the text it was
declared with, or — where the declaration was a lone `var()` —
a [`PropertyValue::CustomRef`] naming the property it defers to, which
substitution follows.

It is text rather than a lowered value because CSS Variables 1 §2 gives a
custom property's value no grammar at all until `var()` substitutes it into a
real property; the grammar is applied there, against the property it landed
on. A value lowered at declaration time is lowered against a grammar nobody
chose, and keeps only as much of the value as that grammar happened to accept.

Custom properties always inherit by default (§4.14).
