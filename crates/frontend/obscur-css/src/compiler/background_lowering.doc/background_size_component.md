Lowers one `background-size` component into a [`Length`]: `auto` →
[`Length::Auto`], a percentage / dimension → its length, a unitless `0` →
`0px`. Any other component (including `cover` / `contain`, which are only
valid alone) yields [`None`].
