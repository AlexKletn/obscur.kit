The replaceable port for opening targets via the shell (spec §6.11).

[`ShellProvider`] is the hexagonal port through which the domain asks the
platform shell to open a [`ShellTarget`](crate::shell::ShellTarget) — a file,
a URI, or a document token — and receives a
[`ShellResult`](crate::shell::ShellResult). App/installer responsibilities
(file associations, MIME/protocol registration, manifests, single-instance
coordination) are out of this port by design (spec §6.11).
