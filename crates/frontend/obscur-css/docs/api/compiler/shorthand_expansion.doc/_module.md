Shorthand expansion: one authored declaration fanned out into the longhands
it sets (§4.13, §4.14).

Every function here answers the same question in a different grammar — given
a shorthand name and its component list, which longhands take which
components — and none of them lowers a value the longhand path cannot already
lower. The families share the CSS replication rules rather than machinery:
the 1/2/3/4-value box rule (`margin` / `padding` / `inset` / `border-width` /
`border-radius`), the two-value pair rule (`place-*` and the logical
`margin-inline` / `padding-block` / `inset-*` pairs), and the positional
forms (`flex`, `grid-column` / `grid-row`, `grid-area`). The `border` and
`outline` shorthands are order-*free* rather than positional and live in
[`super::border_shorthand`]; the value grammars themselves live with the
properties they belong to.
