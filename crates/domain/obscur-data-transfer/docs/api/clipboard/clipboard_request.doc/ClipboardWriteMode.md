How a write places content on the clipboard (spec §6.20 COPY/CUT).

The two modes store identical bytes; they differ only in what the *source*
does afterward. [`Copy`](ClipboardWriteMode::Copy) leaves the source content
in place, while [`Cut`](ClipboardWriteMode::Cut) asks the source to delete
its content once the clipboard has been read (a move). Enforcing the deletion
is the source's concern — this value only records the intent.
