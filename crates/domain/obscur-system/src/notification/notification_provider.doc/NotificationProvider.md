The replaceable port for presenting system notifications (spec §6.12).

An adapter implements this trait once per platform and the domain holds it as
`Box<dyn NotificationProvider>`. The trait is deliberately **object-safe** — no
generic or `Self`-returning methods — so providers can be stored
polymorphically.

Contract: a provider reports its honest [`permission`](Self::permission) and
never panics — a refusal is [`NotificationResult::Denied`] and an absent
facility is [`NotificationResult::Unsupported`] (spec §6.12).
