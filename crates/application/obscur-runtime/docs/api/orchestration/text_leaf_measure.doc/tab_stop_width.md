Resolves the `tab-size` tab-stop width in logical pixels for `content`
(CSS Text 3 §4.3), or `0.0` when `content` has no preserved tab.

A `<number>` tab-size is `factor` × the advance of a space shaped through the
same orchestrator/face the measure and compose passes use (so both agree); a
`<length>` tab-size needs no space and returns its fixed width. When there is
no tab the space is never shaped and `0.0` is returned, so the tab pass
early-returns and an unauthored / collapsing-mode node is unaffected.
