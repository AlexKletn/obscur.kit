Returns `true` when the non-part simples of `selector`'s subject compound —
plus any ancestor/sibling combinator prefix — match `host`.

The `::part()` simple itself is a target, not a host constraint, so it is
skipped. A **type** simple is enforced against the host's CSS type when the
host exposes one; a mounted component host that carries no CSS type name (a
user component with no `type=` on its element) does not have the host-type
filter enforced — the component-host boundary and the export name already
gate the crossing, so this stays permissive rather than failing closed on a
host that simply never named a type. Class / id / attribute / pseudo-class
simples are matched exactly as the ordinary matcher would.
