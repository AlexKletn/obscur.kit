The replaceable port for looking a message up (spec §7.2).

This is the hexagonal port §7.2 calls the *replaceable provider*: the domain
holds a `Box<dyn MessageProvider>` and never knows whether the patterns behind
it came from a compiled-in table, a `.ftl` file an adapter parsed, or a
translation service. Object-safe on purpose — no generics, no `Self`-returning
methods — so providers are stored polymorphically.

# Contract

- [`pattern`](Self::pattern) is asked for an **exact** locale, never a
  language "close enough". Walking the fallback chain is the
  [`Localizer`](crate::catalog::Localizer)'s job, and a provider that
  second-guessed it would fall back twice.
- A provider reports an honest [`availability`](Self::availability) and never
  panics.

The toolkit deliberately stops here. §7.2: *"The toolkit is not a full
application translation-management platform."* Extraction, merge, pluralisation
linting, translator workflow — all of that lives on the far side of this trait.
