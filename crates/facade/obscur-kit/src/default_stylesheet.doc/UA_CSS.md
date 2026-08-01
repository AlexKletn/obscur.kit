The kit's **Wireframe** base look — a grayscale specification-sheet aesthetic
(ink `#2b2a27` on paper `#faf9f6`, hairline `1.5px` borders, tight `2px` radii,
a dashed focus ring, an inverted `:active`, a dashed muted `:disabled`, and a
filled `:checked`), keyed on each control's stable CSS type and restricted to
properties the engine renders. This is the DEFAULT look every bare control
gets; author/theme sheets sit above `SystemDefault` and override it freely.

Palette: ink `#2b2a27`, muted `#5a584f`, faint `#9b998f`, disabled-ink
`#b7b5ad`, surface `#faf9f6`, hover-fill `#e4e2db`, field-hover `#f4f2ec`,
disabled-fill `#f2f0ea`, rule `#d6d4cc`. Fonts are the host's (the source kit's
IBM Plex is not bundled), so only sizes/weights are set.
