The access rights granted for a document handle (spec §6.10).

A file picker or portal grants either read-only or read-write access to the
document it returns; this enum records which. It carries no I/O and makes no
filesystem assumption — it is a pure statement of what the host permits.
