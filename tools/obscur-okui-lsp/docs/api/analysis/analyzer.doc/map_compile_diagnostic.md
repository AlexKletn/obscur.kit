Maps a [`CompileDiagnostic`] to its byte offset, message and LSP severity.

Resolution, type and validation failures are errors; a missing `<For>` key is
the one deliberate warning (see below).

# Why a validation failure is an error

It was a warning, and `--check`'s gate only reads error severity, so the
check ran every rule, printed every verdict, and exited **zero** on all of
them. `<Button laebl="Save"/>` (§2.13), `<VirtualFor>` (§2.11), a malformed
`<slot>` (§2.12) and `{std::fs::read(p)}` (§2.10) each printed a warning and
passed CI. The two front ends that are not this one both treat exactly these
as fatal — the `okui!` macro raises a `syn::Error` at the author's token, and
`obscur-kit`'s `build.rs` panics on `CompileResult::has_errors()`, which
counts every diagnostic irrespective of the severity this function invents.
So the editor was the only place where a document that cannot build was green.

§2.13 settles it in its own words: "Unknown attributes, events and string
method names are **compile errors**." Warning was not a lenient reading of
the spec, it was a different one.
