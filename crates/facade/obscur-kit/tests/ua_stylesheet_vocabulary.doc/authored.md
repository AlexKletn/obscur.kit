Compiles author CSS — the ordinary origin, above `SystemDefault`.

The sibling of `compile`, and the difference is the whole point of the tests
that use it: `compile` states `CascadeOrigin::SystemDefault` so a probe sheet
can stand in for the UA sheet, while this one leaves the compiler's default
origin alone, so what it produces is what an application's own stylesheet is
and outranks every UA rule by origin regardless of specificity.
