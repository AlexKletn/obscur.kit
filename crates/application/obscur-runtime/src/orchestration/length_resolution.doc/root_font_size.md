The document root's computed `font-size` in logical pixels, for `rem`.

Reads the root node's `font-size` (`Length::Px`), defaulting to
[`DEFAULT_FONT_SIZE`] when the root is absent. A resolved `font-size` is always
an absolute pixel length — the style resolver folds it before any other
property — so a root written `2rem` reads back as `32px` here.

# This is now vestigial for `rem`

No `Rem` reaches this pass any more: the resolver absolutizes every computed
length, so the `root_font_size` threaded through `length_context` has nothing
left to resolve. It is kept because `length_context` still serves `%` and `ch`,
which are *not* absolutized, and because a hand-built [`ComputedStyles`] (a
test, a runtime override) can still carry a relative length this pass would
otherwise silently drop.

Note that `root` here is [`RetainedTree::root`](obscur_tree::tree::RetainedTree::root)
— the synthetic host node —
whereas the resolver's `rem` base is the root **element** beneath it (what
`:root` matches). The two agree in the only way that matters: nothing consults
this value for `rem` any more.
