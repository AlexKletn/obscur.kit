A reference, no-op [`TextServiceProvider`] that supplies nothing — the honest
default (spec §5.13).

Per §5.13 no linguistic engine is required in the core package, so the default
provider offers no service: [`supports`](TextServiceProvider::supports) is
`false` for every kind, [`analyze`](TextServiceProvider::analyze) returns an
empty [`Vec`], and [`detect_language`](TextServiceProvider::detect_language)
returns `None`. Its "nothing" answers are correct behavior, not stubs waiting
to be filled in; a real adapter replaces it.
