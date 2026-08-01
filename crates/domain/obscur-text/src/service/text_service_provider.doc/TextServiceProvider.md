The replaceable port for optional language services (spec §5.13).

A provider may supply any subset of the §5.13 services — spellcheck, grammar,
autocorrect, language detection, and user-dictionary lookups — and reports
what it offers through [`supports`](Self::supports). Multiple providers may be
installed; the engine coordinating them is the caller's concern.

The trait is deliberately **object-safe** — no generic or `Self`-returning
methods — so adapters can be held as `Box<dyn TextServiceProvider>`.

The domain contract is a **pure function** from a
[`LanguageServiceRequest`] to a [`Vec<TextDiagnostic>`]: given a request,
[`analyze`](Self::analyze) returns the diagnostics for it. The §5.13
*asynchronous* and *incremental* delivery of those diagnostics — running the
engine off-thread, streaming partial results, coalescing versions — is the
adapter's responsibility and lives outside this pure-domain port.
