The other door the CSS grammar comes through (CSS Variables 1 §3).

A custom property is declared as an arbitrary token sequence, so what
`--font-body: Inter, sans-serif` *means* is not decided until a `var()` puts
it on a property — at computed-value time, inside this runtime. Reading it
then takes the CSS grammar, which this tier may not depend on, so the grammar
is installed the way the `style` attribute's compiler is (see
[`inline_style`](super::inline_style)).
