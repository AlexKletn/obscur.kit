Emits the Rust expression for a text node's content.

A binding is its **Rust path**, displayed: `<div>{user.name}</div>` becomes
`ViewNode::text(ToString::to_string(&(user.name)))`. It used to emit the path
as a string *literal*, so `{user.name}` rendered the nine characters
`user.name` — the document showing its own source. That was fixed for
attributes and left live here, on the more common case.

`Display` is the bound, where [`emit_value`] uses `Into<ViewValue>`, because
text content is a string and a body `{count}` must render `7` rather than fail
to compile. A type with no `Display` is **rustc's** error in the caller's
build — the same borrowed type system every other binding leans on. (The
emitted tokens are re-parsed from text, so that error lands on the whole
`okui!` invocation, not on the offending expression; only the §2.10 check,
which runs on the parsed input, is spanned that precisely.) Displaying by
reference, so reading a field for text never moves the caller's data.

The path is parenthesised because a "path" is only mostly a path: both front
ends fall back to a binding on the raw text for anything they do not classify
as a literal, so `{a + b}` arrives here as the binding `a + b`, and
`&a + b` is not what the author wrote.

A call emits the empty string for the reason [`emit_value`] emits `Null`:
§2.10 allows only "explicitly exported pure functions", and nothing yet decides
which those are.
