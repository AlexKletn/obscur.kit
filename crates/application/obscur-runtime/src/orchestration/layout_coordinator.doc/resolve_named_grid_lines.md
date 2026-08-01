Resolves every grid item's named `grid-column` / `grid-row` line reference
against its container's `grid-template-columns` / `grid-template-rows`
`[name]` line names, patching the item's [`LayoutStyle`] placement in
`styles` (spec §4.13).

Like named-area placement, a named line (`grid-column: sidebar-start`) is
declared on the item's **container**'s track list, so this runs after
[`layout_styles_from`](Self::layout_styles_from) and only overwrites the
named edges of items that reference one (looked up against the parent's
per-axis line-name map); every non-named layout is left byte-for-byte
identical.
