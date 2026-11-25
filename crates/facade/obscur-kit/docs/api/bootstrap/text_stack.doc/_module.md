The font set a host hands the toolkit, and the handles the install gives back
(spec §5.18).

A self-shaping toolkit needs font **bytes**. It cannot take a CSS family name
and hope the platform draws it: the shaper reads `cmap`/`GSUB`/`GPOS` itself, so
"use the system font" is only meaningful once something has read a file off the
host's disk. That is why the byte-carrying [`FontSet`](super::FontSet) is the input to
[`KitBootstrap::install_text_stack`](super::KitBootstrap::install_text_stack).
An **empty** set is not the end of the road: the install appends a bundled
last-resort face so text still paints in an isolated environment — see
[`crate::bootstrap`](super)'s note on the §5.18 bundled fallback.
