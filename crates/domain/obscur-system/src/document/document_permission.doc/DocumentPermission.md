The access rights and optional persistent permission token for a document
(spec §6.10 persistent permission token).

A document handle carries a permission describing what the host allows and,
optionally, an opaque **persistent permission token** the host can re-use to
re-acquire access to the same document in a later session (for example a
security-scoped bookmark on macOS or a portal document identifier on Linux).
A `None` token means access is ephemeral — valid only for the current session.

The token is opaque: this pure domain never interprets it, only stores and
hands it back. There is no filesystem path here (spec §6.10 — handles are not
paths).
