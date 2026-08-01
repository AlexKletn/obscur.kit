The replaceable port for resolving fonts (spec §5.18).

The domain holds a provider as `Box<dyn FontProvider>`, so the trait is
deliberately **object-safe** — no generic methods and no `Self`-returning
methods. An adapter implements it once per backend; the default provider
uses system fonts on native platforms and a bundled fallback set elsewhere,
and any adapter can replace it.
