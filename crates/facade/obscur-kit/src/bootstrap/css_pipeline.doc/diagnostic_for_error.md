Builds the `Style`-subsystem diagnostic for a CSS syntactic error. The
[`stable code`](DiagnosticCode) is derived from the error's *kind*, not its
(localizable) message or its source span, so it is identical across locales
and call sites (spec §1.6). The human-readable message carries the detail.
