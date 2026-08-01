A single `@keyframes` reference declared by a style rule: the interned
animation name, its duration, and the playback controls the `animation-*`
longhands set (css-animations-1 §3.3–§3.9).

The name is interned in the stylesheet's shared [`CssSymbolInterner`], so it
compares equal to the matching [`CompiledKeyframes`](super::CompiledKeyframes)
name and can be resolved to that rule's tracks.
