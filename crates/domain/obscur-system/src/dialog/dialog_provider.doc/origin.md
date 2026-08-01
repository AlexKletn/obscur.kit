Returns a stable identifier for this provider (spec §6.9).

A composing caller records the origins already on a provider chain and
refuses to re-enter one, preventing a kit-rendered dialog implemented over
this provider from recursively re-entering it.
