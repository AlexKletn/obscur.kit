A node's effective custom properties, and the `var()` substitution that reads
them (§4.14): [`effective_custom_properties`] and [`substitute`].

Custom properties always inherit, so a node's effective set is its parent's
set overridden by its own cascaded ones — and that set is the *whole* input to
`var()`. The two belong together because the second is meaningless without the
first: a substitution against anything narrower than the effective set
silently drops an inherited `--token`, which is the one thing custom
properties exist to carry down a tree.

Both run before any computed value, which is why neither takes a
[`LengthContext`](crate::property::LengthContext): the set has to be complete
before `font-size` — and therefore before every length base but the root's —
can be resolved.
