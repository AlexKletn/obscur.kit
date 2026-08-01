Lowers a bare keyword, handling the CSS-wide keywords, `auto`, the intrinsic
sizing keywords, and the named colours the domain knows; anything else becomes
an interned keyword.

# Why the sizing keywords are lowered here and not per property

`min-content` / `max-content` / `fit-content` are values of `width`, `height`,
the `min-*`/`max-*` bounds and `flex-basis`, so the property-keyed
[`super::longhand_catalog::layout_keyword_mapper`] looks like the right home
for them. It is not: that path claims **every** bare keyword on a property it
maps, and it would
therefore claim `auto` — which must stay a [`Length::Auto`], not a keyword
symbol. Routing `width` through it would silently change what `width: auto`
computes to.

Lowering them alongside `auto` here keeps that seam untouched. It is
property-agnostic, exactly as the `auto` case already is: `color: max-content`
also produces the sizing symbol, and the colour path ignores it — the same
nothing it does with any keyword it cannot use.
