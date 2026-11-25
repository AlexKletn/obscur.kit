Two values — the ends of a span — bounded by an inclusive `[min, max]`
range.

The variant exists because a two-thumb control genuinely has two positions
and no single one. Every platform accessibility API carries exactly one
numeric value per node (ARIA's `aria-valuenow`, AT-SPI's `Value`, UIA's
`RangeValue`, AccessKit's `numeric_value`), so reporting such a control as
a [`Number`](Self::Number) would have to pick an end — and announce half
the range as if it were the whole control. Keeping both ends here lets the
adapter say what is true even when the platform's numeric slot cannot hold
it.
