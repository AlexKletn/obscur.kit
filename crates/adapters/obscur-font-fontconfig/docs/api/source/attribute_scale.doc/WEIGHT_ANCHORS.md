fontconfig weight → CSS weight anchors, from `fontconfig.h`'s `FC_WEIGHT_*`
constants paired with the CSS weight each names.

fontconfig's weight axis is **not** CSS's: it runs `0..=215` and is
deliberately non-linear (`REGULAR` is 80, `MEDIUM` 100, but `BOLD` is 200 and
`BLACK` only 210 — the top of the scale is compressed). Reading a `%{weight}`
as if it were a CSS weight would call every regular face a hairline. The table
is sorted by the fontconfig value so [`css_weight`] can interpolate.
