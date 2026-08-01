Enumerates the displays currently known to the backend (spec §6.6).

Returns an empty [`Vec`] when no displays are available; geometry the
platform does not expose is reported as `None` on each [`Display`] rather
than fabricated.
