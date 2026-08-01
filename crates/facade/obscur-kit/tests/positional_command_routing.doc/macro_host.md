The same two rows as [`SCREEN`], authored with the **macro** front end.

Audit 2-3: this function could not be written. `okui!` is a proc-macro over
Rust tokens, where `-` is punctuation, so `test-id="a"` was `expected ident`
— and the identifier this whole file is built on was authorable only in the
front end `docs/api/09-authoring-formats.md` calls the *fallback*. The
documented workaround (put the id on the authored component tag in a `.okui`
file and let the handler catch the command as an ancestor on §2.6 link 3) is a
different feature: it fires for every descendant, not for the node.
