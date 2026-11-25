Applies the CSS Text 3 white-space processing rules (§4.1.1) to `content`
**before** shaping, returning the processed string.

This is the pre-shape collapsing seam (the string-extraction point). Under a
collapsing mode ([`WhiteSpace::collapses_spaces`]): each run of collapsible
spaces/tabs (U+0020, U+0009) collapses to a single U+0020; a segment break
(`\n`, with `\r\n` / `\r` first normalised to one `\n`) is either preserved as
a forced break (when [`WhiteSpace::preserves_segment_breaks`], i.e. `pre-line`)
or removed and treated as a collapsible space (`normal` / `nowrap`); and
leading/trailing collapsible whitespace of the block — plus the collapsible
whitespace immediately around each preserved forced break — is trimmed. Under
a preserving mode (`pre` / `pre-wrap`) spaces, tabs, and segment breaks are
kept verbatim (only `\r` is normalised).

Full per-line hanging-whitespace trimming after wrap is a documented
refinement; this does the block-level and around-forced-break trim. The
processed string flows through itemization, shaping, and the paragraph cache
key, so a collapsed node is a distinct cache entry and the measure/compose
passes — which both run `content` through this same function — shape identical
text. A [`Cow::Borrowed`] is returned whenever the rules leave `content`
byte-for-byte unchanged (the common single-line, single-spaced case).
