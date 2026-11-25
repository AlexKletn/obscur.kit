Resolves this node's `font-size` to absolute logical pixels (§4.14).

The cascaded value goes through the same substitution / CSS-wide-keyword /
inheritance path as every other property, and the resulting [`Length`] is
then folded against the *parent's* size — the base `em` and `%` mean on
this one property. An inherited value is already absolute (the parent's
pass folded it), so it survives that fold unchanged.
