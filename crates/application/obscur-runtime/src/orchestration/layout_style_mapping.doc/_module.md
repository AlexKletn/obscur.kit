The [`ComputedStyle`] → [`LayoutStyle`] bridge: [`layout_style_from`].

One node's computed CSS, mapped into `obscur-layout`'s own vocabulary (spec
§3.10, §4.2). Layout consumes a [`LayoutStyle`] and knows nothing about
[`PropertyId`] or the cascade, so this is the single place where the two
vocabularies meet — and it is nearly pure: a computed style plus a resolution
context in, a [`LayoutStyle`] out, no tree and no side-tables.

The keyword readers below are private local functions with no independent
semantics: each compares one property's interned symbol against the reserved
[`layout_keyword`] handles and is called exactly once, by the assembly above
them. The *length* half is a separate entity, in
[`length_resolution`](super::length_resolution), because it has a second
consumer — `grid_templates` folds a fixed track's relative units through the
same context.
