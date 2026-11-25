The shared wrapping core: packs `runs` into lines and sizes each line box
from the CSS 2.1 §10.8.1 strut (`strut` + per-font `seg_metrics`).

`letter_spacing` / `word_spacing` are the resolved CSS Text 3 §8.1/§8.2
tracking values that were folded into the glyph advances upstream (by
[`apply_spacing`](crate::shaping::apply_spacing)); the core needs them to
*hang* (trim) the trailing tracking of each line's last cluster, which §8.1
says is "deferred to the line breaker". They are `0.0` on every historical
path, so the trim is then a no-op and the output is byte-identical.

`tab_width` is the resolved `tab-size` stop width (CSS Text 3 §4.3): after
packing, each preserved tab is re-expanded from the start of *its own line*
via [`PackedLine::retab`], because §4.3 measures a tab stop from the line
the tab is on and the pre-wrap
[`apply_tab_stops`](crate::shaping::apply_tab_stops) pass could only reset
the pen at a hard `\n`, never at a soft-wrap boundary. `0.0` (or text with
no `\t`) disables the pass, so every historical path is byte-identical, as
is a tab that stays on the first (line-0) line — its line start *is* the
paragraph origin the pre-wrap pass measured from.
