The `style` attribute, compiled and folded into the cascade at origin 5
(spec §4.14).

This crate cannot parse CSS — it does not depend on the front end that can, and
should not. So the compiler arrives as a PORT installed by whoever wired the
CSS pipeline, exactly as a resource decoder does. Without one, a `style`
attribute is inert rather than half-understood.
