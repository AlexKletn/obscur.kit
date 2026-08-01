The `min` sizing function of a track (§11.4). `axis` is the content extent a
`Percent` bound resolves against.

Per §7.2.3 a standalone `fr` is formally `minmax(auto, fr)` — an `auto`
(content-based) minimum. Its base therefore grows to the items' **minimum
contribution**, which §12.5 accommodates with the min-content contribution
(measured under an offer of no space, [`Kind::Min`]) — not the loose/max-content
measure, so a reflowable item still shrinks to its longest word rather than
inflating the flexible track to its preferred size. This is what makes an `fr`
track blown past its fair share by content restart the fr algorithm (§12.7.1);
a track with no content min keeps a base of `0` and a plain `1fr` split. An
`fr` used as a `minmax` *min* is invalid and acts as `0`.
