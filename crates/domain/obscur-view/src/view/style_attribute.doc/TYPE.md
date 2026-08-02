Reserved symbol for the authored `type="…"` attribute (as [`ViewValue::Text`])
— WHATWG HTML's own "which kind of control is this", the value an
`[type=submit]` attribute selector reads (spec §4.13).

It carries *only* what the author wrote. The element's local name — the tag a
`button { … }` type selector matches — is [`LOCAL_NAME`], a separate symbol, so
`<button type="submit">` is a `button` that also answers to `[type=submit]`.
