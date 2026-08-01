Descriptive metadata a host reports for a document or directory
(spec §6.10 metadata).

A file picker hands back a handle plus descriptive metadata: a display
`name` (spec §6.10 — a name for presentation, *not* a filesystem path), an
optional MIME type, byte size, and last-modified time, and whether the handle
refers to a directory. Every field beyond the name is optional because a
portal, cloud document, or virtual filesystem may not expose it — such
details are reported as `None` rather than fabricated.

This is pure data: `modified_unix_secs` is a plain Unix timestamp supplied by
the host, never read from a clock here.
