The key half of §7.2's *key + parameters* abstraction: a stable, opaque name
for a message, independent of any language's wording.

A key is a name, not a default translation. `"file.count"` is a key;
`"{n} files"` is an English message that happens to be a string. Keying by the
English text is the shortcut that makes English structurally privileged — it
forces every other language to be a lookup *from* English and breaks the
moment the English wording is edited.
