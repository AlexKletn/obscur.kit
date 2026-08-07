Whether any leaf is a `%`, whose basis is a property-specific used value
(a percentage in `translate` is the element's own border box) that no
[`LengthContext`](crate::property::LengthContext) carries.

A caller that has no basis to offer uses this to REFUSE the expression up
front — with a diagnostic — rather than resolve it to a plausible-looking `0`.
