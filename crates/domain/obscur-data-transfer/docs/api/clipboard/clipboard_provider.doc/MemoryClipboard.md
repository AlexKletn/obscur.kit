A deterministic, in-process reference [`ClipboardProvider`](super::ClipboardProvider) (spec §6.20).

`MemoryClipboard` holds only the current clipboard contents — the last
written [`DataOffer`] — in memory, with no platform dependency. It is the
honest default for headless environments and tests, and documents the port's
contract concretely. System clipboard *history* is out of scope (spec §6.20);
this holds a single slot. At the storage level a cut is treated exactly like
a copy — the source-side deletion a [`Cut`](super::ClipboardWriteMode::Cut)
implies is the caller's concern, not the clipboard's.
