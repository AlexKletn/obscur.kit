A single text document held open by the language server.

It pairs the document `uri`, the client's monotonically increasing `version`
and the full `text`. Full-text synchronization is used (the whole `text` is
replaced on every change), which is the simplest correct LSP sync model.
