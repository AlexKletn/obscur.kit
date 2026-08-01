A single-line field's `Enter`: a **commit** (WHATWG HTML §4.10.5.5 /
§4.10.21.3 implicit form submission). It fires `change` with the committed
value and rebaselines the session, inserts nothing, and is **not** consumed
— Enter's routing past the commit is byte-identical.
