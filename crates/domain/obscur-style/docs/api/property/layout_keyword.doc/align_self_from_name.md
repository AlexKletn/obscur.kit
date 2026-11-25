Maps a `justify-self` / `align-self` keyword string to its reserved
[`InternedSymbol`].

The input is ASCII-lowercased before matching. This accepts everything
[`align_items_from_name`] does (`start` / `end` / `center` / `stretch`, plus
the flexbox `flex-start` / `flex-end` spellings) and additionally `auto`
(mapped to [`ALIGN_AUTO`]), the per-item value that inherits the container's
items default. Returns [`None`] for any keyword outside that set.
