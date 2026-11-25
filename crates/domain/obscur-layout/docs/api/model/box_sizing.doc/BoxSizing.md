How an explicit `width` / `height` is interpreted against the box model
(CSS `box-sizing`, spec §2.3).

This governs only how an **explicit** size resolves into the element's
*content* extent (the region children are laid out within); an `Auto` size is
unaffected (it always fills the available content box). The distinction only
matters when the element has border + padding: with none, both modes are
identical.

* [`ContentBox`](Self::ContentBox) — the CSS default. An explicit `width` /
  `height` is the **content** box, so the element's border box is
  `authored + border + padding`.
* [`BorderBox`](Self::BorderBox) — an explicit `width` / `height` is the
  **border** box, so the content extent is `authored − border − padding`
  (floored at `0`) and the element's border box equals the authored size.
