Sets the **readonly** state, returning the updated input (fluent builder).

A readonly field (the HTML `readonly` attribute, WHATWG HTML
§attr-input-readonly) is still focusable and its text still selectable /
copyable, but it cannot be edited (no insert/delete, no IME commit, no
paste). It is **independent of** [`disabled`](Self::disabled): a field can be
neither, one, or both.
