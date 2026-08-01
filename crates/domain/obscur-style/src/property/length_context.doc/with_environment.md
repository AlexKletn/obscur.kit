Returns a copy of this context with the host-reported `environment` set.

A builder rather than a [`new`](Self::new) parameter for the same reason
`ch_width` is one: only a caller in touch with the host can answer, and a
context built without one stays at the no-cutout, no-keyboard default that
every desktop genuinely has.
