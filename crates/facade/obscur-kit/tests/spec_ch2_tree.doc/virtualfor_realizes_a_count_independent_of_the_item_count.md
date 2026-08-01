§2.11: "- `VirtualFor`;"

`<VirtualFor>` cannot be written in an `okui!` invocation in this file —
were the front end to refuse it, the *test file* would fail to compile and
report nothing. It is therefore driven through the `.okui` string path,
whose verdict is a value rather than a compile error, and the realized-count
assertion follows from the document that path produces.
