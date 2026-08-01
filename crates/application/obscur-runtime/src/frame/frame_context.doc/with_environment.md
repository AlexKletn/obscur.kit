Returns this context with the host-reported environment values attached —
the safe-area and on-screen-keyboard insets `env()` resolves against
(spec §6.15).

Defaulted rather than a [`new`](Self::new) parameter because a host that
reports nothing is the common case and its answer (all zeroes) is correct:
a screen with no cutout and no keyboard genuinely has no insets.
