Creates a denied subscription record.

A denial carries no live subscription, so `id` is conventionally
[`SubscriptionId::default`] — the server mints live ids from 1, so a zero
id can never collide with one.
