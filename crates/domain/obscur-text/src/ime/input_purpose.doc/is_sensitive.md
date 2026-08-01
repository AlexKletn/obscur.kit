Returns whether this purpose implies sensitive-mode handling: true for
[`Password`](Self::Password) and [`Pin`](Self::Pin) (spec §5.12).

This is a hint that the field holds a secret, so IME learning,
suggestions, and clipboard exposure should be suppressed even when the
caller did not set an explicit sensitive flag.
