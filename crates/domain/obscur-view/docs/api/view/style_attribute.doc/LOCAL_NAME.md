Reserved symbol for the element's **local name** — the tag a CSS type selector
matches (as [`ViewValue::Text`]).

This is the element's *identity*, not one of its attributes: it is what
`button { … }` matches, what `SemanticBuilder::role_for_local_name` derives a
§7.1 role from, and what `EditableField::is_multiline` reads to tell a
`<textarea>` from an `<input>`. No author writes it — both `.okui` lowering
paths append it from the tag itself, and each Rust widget sets its own — so it
is never in competition with anything the author authored.

That independence is the point of the symbol. It used to share [`TYPE`] with
HTML's own `type="…"` attribute, and the two meanings could not both be
carried: `<button type="submit">` was an element whose CSS *type* was `submit`,
so it matched no `button { … }` rule and silently lost its whole UA styling.
The tag says *what a thing is* and `type` says *which kind*; they are two
channels and now they are two symbols.
