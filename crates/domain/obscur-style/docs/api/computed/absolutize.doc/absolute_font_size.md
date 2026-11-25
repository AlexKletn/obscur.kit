The absolute `font-size` of one node, in logical pixels (§4.14).

`font-size` is resolved **before** every other property because it is the base
its own node's `em` (and `ch`) resolve against — so it cannot be folded with
the context it defines. Its own relative units resolve against the *parent's*
size instead, which is what CSS specifies:

* `em` and `%` are relative to `parent_font_size` (`1.5em` and `150%` are the
  same declaration on this property);
* `rem` is relative to `root_font_size`, `vw`/`vh` to the viewport;
* `ch` is relative to the **parent's** `0` advance, supplied as
  `parent_ch_width` — see below.

`parent_font_size` is the root's implicit parent value (the `medium` initial)
for the root node, which is also what `rem` means on the root element.

# `font-size: 2ch` is not circular

It looks like it must be: `ch` is the advance of the `0` glyph in the font used
to render the element, and that font's size is the very thing being computed.
CSS Values 4 §6.1.2 cuts the knot explicitly, in the paragraph after the unit
table:

> When used in the value of any `font-*` property on the element they refer to,
> the font-relative lengths resolve against the computed metrics of the
> **parent** element — or against the computed metrics corresponding to the
> initial values of the font and line-height properties, if the element has no
> parent.

So `2ch` here means two of the *parent's* `0` advances, which are fully known
by the time this node is resolved — the same rule that already makes `em` and
`%` resolvable on this property. `parent_ch_width` is that measurement.

[`None`] means no measurement was available (no [`ChMetrics`] port, or a port
that could not measure). The fallback is then the parent's size — the same
answer an unresolvable `font-size` gets, and deliberately *not* `ch_base`'s
`0.5em` substitute, which would silently halve the text.

[`ChMetrics`]: super::ChMetrics
