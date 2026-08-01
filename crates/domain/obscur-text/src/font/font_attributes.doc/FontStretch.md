A font's width, from [`UltraCondensed`](FontStretch::UltraCondensed) to
[`UltraExpanded`](FontStretch::UltraExpanded) (spec §5.18).

Variants are declared narrowest-to-widest, so the derived [`Ord`] ranks them
by width and providers can find the nearest available width. The default is
[`FontStretch::Normal`].
