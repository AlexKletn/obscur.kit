The replaceable port for presenting dialogs to the user (spec §6.9).

An adapter implements this trait once per presentation strategy (native,
portal-external, or kit-rendered) and the domain holds it as
`Box<dyn DialogProvider>`. The trait is deliberately **object-safe** — no
generic or `Self`-returning methods — so providers can be stored and chained
polymorphically.

Contract: a provider reports honest, typed outcomes and **never panics** on an
unsupported request — it answers [`DialogResult::Unsupported`] and reports
[`SupportLevel::Unavailable`] from [`Self::supports`] instead. See the module
docs for how [`Self::origin`] drives §6.9 chain loop detection.
