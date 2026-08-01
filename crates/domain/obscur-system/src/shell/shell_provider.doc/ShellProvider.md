The replaceable port for opening targets through the platform shell
(spec §6.11).

An adapter implements this trait once per platform and the domain holds it as
`Box<dyn ShellProvider>`. The trait is deliberately **object-safe** — no
generic or `Self`-returning methods — so providers can be stored
polymorphically.

Contract: a provider reports honest, typed outcomes and **never panics** on a
target it cannot open — it answers [`ShellResult::Unsupported`] (or
[`ShellResult::Denied`]) instead.
