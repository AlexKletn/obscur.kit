Every attribute on an authored `<slot …/>` that is not its `name`, spelled as
the author wrote it.

An `on:` binding is spelled back whole (`on:click`) rather than as the bare
event: the message quotes the attribute, and an author looking for `on:click`
must find `on:click`. `DslAttr::Event` keeps only the part after the colon, so
the prefix is put back here — the same repair the file front end's
`authored_spelling` makes.
