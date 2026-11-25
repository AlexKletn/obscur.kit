The cascaded value for `property`, honouring any **logical** declaration
that maps onto it (spec §4.13).

A logical property and its physical counterpart are separate `PropertyId`s
that cascade independently, but CSS decides between them by ordinary cascade
order. Both winners are therefore compared here, by the priority the cascade
recorded — the later/stronger declaration wins, exactly as it would between
two `margin-left` rules.

Ties and missing priorities prefer the physical declaration: a value with no
recorded priority did not come from a cascade (a test, a runtime override),
and "the physical one" is the only honest answer when nothing states which
was declared later.
