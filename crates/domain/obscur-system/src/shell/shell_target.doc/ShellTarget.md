What to open through the platform shell (spec §6.11).

A [`ShellProvider`](crate::shell::ShellProvider) opens a target and reports a
[`ShellResult`](crate::shell::ShellResult). A target is a pure value naming a
file, an arbitrary URI, or an opaque document token — never a live handle to
the platform. File-association, MIME/protocol registration, and single-instance
concerns are the app/installer's responsibility and stay out of this type
(spec §6.11).
