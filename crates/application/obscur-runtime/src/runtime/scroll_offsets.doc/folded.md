The scroll-chain-adjusted (viewport-relative) view of the last folded
boxes, which the public geometry accessor exposes for observers (spec
§4.4). Populated only while [`folded_active`](Self::folded_active) is set —
i.e. some scroll container overflows — so an unscrolled tree keeps
returning the content-space boxes by reference with no per-frame clone.
