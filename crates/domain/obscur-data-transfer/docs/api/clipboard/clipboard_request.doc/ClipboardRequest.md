A single clipboard command: copy/cut *to* the clipboard, paste *from* it, or
clear it (spec §6.20 data transfer — clipboard).

The clipboard and drag-and-drop share the data-transfer subsystem; this type
models the clipboard's three verbs as one value driven through the
[`ClipboardProvider`](super::ClipboardProvider) port. A
[`Write`](ClipboardRequest::Write) carries the [`TransferPayload`] to store
and its [`ClipboardWriteMode`](super::ClipboardWriteMode); a [`Read`](ClipboardRequest::Read) requests a
paste, optionally narrowing to specific [`MimeType`]s (an empty list means
"any/all available"); [`Clear`](ClipboardRequest::Clear) empties the
clipboard. The command is a pure, deterministic value carrying no I/O.
