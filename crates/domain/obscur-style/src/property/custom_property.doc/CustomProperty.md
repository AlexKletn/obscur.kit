A CSS custom property declaration (`--name: value`).

The `name` is the interned `--`-prefixed identifier; the `value` is the
declared [`PropertyValue`], which may itself be an unresolved
[`PropertyValue::CustomRef`] until custom-property substitution runs.

Custom properties always inherit by default (§4.14).
